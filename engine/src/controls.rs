//! CEF Views-only navigation fixture. This is an engine input test, not the
//! final ClearLane browser shell or styling system.
use cef::*;

#[derive(Clone)]
enum NavigationAction {
    Back,
    Forward,
    Reload,
}

wrap_button_delegate! {
    struct NavigationButton {
        browser_view: BrowserView,
        action: NavigationAction,
    }

    impl ViewDelegate {}

    impl ButtonDelegate {
        fn on_button_pressed(&self, _button: Option<&mut Button>) {
            let Some(browser) = self.browser_view.browser() else {
                return;
            };

            match self.action {
                NavigationAction::Back if browser.can_go_back() != 0 => browser.go_back(),
                NavigationAction::Forward if browser.can_go_forward() != 0 => browser.go_forward(),
                NavigationAction::Reload => browser.reload(),
                _ => {}
            }
        }
    }
}

wrap_textfield_delegate! {
    struct NavigationAddress {
        browser_view: BrowserView,
    }

    impl ViewDelegate {}

    impl TextfieldDelegate {
        fn on_key_event(
            &self,
            textfield: Option<&mut Textfield>,
            event: Option<&KeyEvent>,
        ) -> i32 {
            let (Some(textfield), Some(event)) = (textfield, event) else {
                return 0;
            };
            if event.type_ != KeyEventType::RAWKEYDOWN || event.windows_key_code != 13 {
                return 0;
            }

            let input = CefString::from(&textfield.text()).to_string();
            let Some(url) = normalize_address(&input) else {
                return 1;
            };
            if let Some(browser) = self.browser_view.browser()
                && let Some(frame) = browser.main_frame()
            {
                frame.load_url(Some(&CefString::from(url.as_str())));
                textfield.set_text(Some(&CefString::from(url.as_str())));
                return 1;
            }
            0
        }
    }
}

/// Only HTTP(S) addresses belong in this engine proof. Search queries and
/// privileged/internal schemes are left to the future browser omnibox.
fn normalize_address(input: &str) -> Option<String> {
    let value = input.trim();
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return None;
    }
    let lowered = value.to_ascii_lowercase();
    if lowered.starts_with("https://") || lowered.starts_with("http://") {
        let host = value.split_once("://")?.1;
        return (!host.is_empty() && !host.starts_with('/')).then(|| value.to_owned());
    }
    if value.contains("://") {
        return None;
    }
    if value.contains('/') || value.contains('.') || value.eq_ignore_ascii_case("localhost") {
        return Some(format!("https://{value}"));
    }
    None
}

/// Attach the real CEF browser and a small native Views toolbar. Both are
/// laid out by CEF (DIP-aware), rather than by manual Windows HWND geometry.
pub fn attach(window: &mut Window, browser_view: &BrowserView, initial_url: &str) {
    let settings = BoxLayoutSettings {
        size: std::mem::size_of::<BoxLayoutSettings>(),
        horizontal: 0,
        ..Default::default()
    };
    let Some(window_layout) = window.set_to_box_layout(Some(&settings)) else {
        eprintln!("Could not initialize CEF window layout");
        return;
    };

    let Some(toolbar) = panel_create(None) else {
        eprintln!("Could not create CEF navigation toolbar");
        return;
    };
    let row = BoxLayoutSettings {
        size: std::mem::size_of::<BoxLayoutSettings>(),
        horizontal: 1,
        between_child_spacing: 4,
        inside_border_horizontal_spacing: 8,
        inside_border_vertical_spacing: 6,
        ..Default::default()
    };
    let Some(row_layout) = toolbar.set_to_box_layout(Some(&row)) else {
        eprintln!("Could not initialize CEF toolbar layout");
        return;
    };

    for (label, action) in [
        ("Back", NavigationAction::Back),
        ("Forward", NavigationAction::Forward),
        ("Reload", NavigationAction::Reload),
    ] {
        let mut delegate = NavigationButton::new(browser_view.clone(), action);
        let caption = CefString::from(label);
        if let Some(button) = label_button_create(Some(&mut delegate), Some(&caption)) {
            let mut view = View::from(&button);
            toolbar.add_child_view(Some(&mut view));
        }
    }

    let mut address_delegate = NavigationAddress::new(browser_view.clone());
    if let Some(textfield) = textfield_create(Some(&mut address_delegate)) {
        textfield.set_text(Some(&CefString::from(initial_url)));
        textfield.set_placeholder_text(Some(&CefString::from("Enter a website URL")));
        let mut view = View::from(&textfield);
        toolbar.add_child_view(Some(&mut view));
        row_layout.set_flex_for_view(Some(&mut view), 1);
    } else {
        eprintln!("Could not create CEF address textfield");
    }

    let mut toolbar_view = View::from(&toolbar);
    window.add_child_view(Some(&mut toolbar_view));

    let mut web_view = View::from(browser_view);
    window.add_child_view(Some(&mut web_view));
    window_layout.set_flex_for_view(Some(&mut web_view), 1);
    window.layout();
}

#[cfg(test)]
mod tests {
    use super::normalize_address;

    #[test]
    fn addresses_are_normalized_without_accepting_internal_schemes() {
        assert_eq!(
            normalize_address("  example.com  "),
            Some("https://example.com".to_owned())
        );
        assert_eq!(
            normalize_address("https://www.rust-lang.org/"),
            Some("https://www.rust-lang.org/".to_owned())
        );
        assert_eq!(
            normalize_address("http://localhost:3000/test"),
            Some("http://localhost:3000/test".to_owned())
        );
        assert_eq!(normalize_address("javascript:alert(1)"), None);
        assert_eq!(normalize_address("file:///C:/secret"), None);
        assert_eq!(normalize_address("   "), None);
        assert_eq!(normalize_address("a search phrase"), None);
    }
}
