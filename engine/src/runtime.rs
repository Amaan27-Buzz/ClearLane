//! Minimal real Chromium/CEF vertical proof, intentionally isolated from the
//! product UI until window ownership and integration have been validated.
use cef::*;
use std::{cell::RefCell, path::PathBuf};

wrap_window_delegate! {
    struct ProofWindow {
        browser_view: RefCell<Option<BrowserView>>,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 1240, height: 780 }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            let view = self.browser_view.borrow();
            let (Some(window), Some(browser_view)) = (window, view.as_ref()) else {
                return;
            };
            let mut child = View::from(browser_view);
            window.add_child_view(Some(&mut child));
            window.set_title(Some(&CefString::from("ClearLane — Chromium engine proof")));
            window.show();
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            *self.browser_view.borrow_mut() = None;
        }

        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            self.browser_view
                .borrow()
                .as_ref()
                .and_then(BrowserView::browser)
                .and_then(|browser| browser.host())
                .map_or(1, |host| host.try_close_browser())
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

wrap_browser_view_delegate! {
    struct ProofBrowserViewDelegate {}

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::ALLOY
        }
    }
}

wrap_client! {
    struct ProofClient {}

    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(ProofDisplayHandler::new())
        }
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(ProofLifeSpanHandler::new())
        }
    }
}

wrap_display_handler! {
    struct ProofDisplayHandler {}

    impl DisplayHandler {
        fn on_title_change(&self, browser: Option<&mut Browser>, title: Option<&CefString>) {
            if let Some(mut browser) = browser.cloned()
                && let Some(view) = browser_view_get_for_browser(Some(&mut browser))
                && let Some(window) = view.window() {
                window.set_title(title);
            }
        }
    }
}

wrap_life_span_handler! {
    struct ProofLifeSpanHandler {}

    impl LifeSpanHandler {
        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            // Exactly one browser is created in this proof.
            quit_message_loop();
        }
    }
}

wrap_app! {
    struct ProofApp { start_url: String }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(ProofBrowserProcessHandler::new(self.start_url.clone()))
        }
    }
}

wrap_browser_process_handler! {
    struct ProofBrowserProcessHandler { start_url: String }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            debug_assert_ne!(currently_on(ThreadId::UI), 0);
            let url = CefString::from(self.start_url.as_str());
            let settings = BrowserSettings::default();
            let mut client = ProofClient::new();
            let mut browser_delegate = ProofBrowserViewDelegate::new();
            let view = browser_view_create(
                Some(&mut client),
                Some(&url),
                Some(&settings),
                None,
                None,
                Some(&mut browser_delegate),
            );
            if view.is_none() {
                eprintln!("CEF did not create a BrowserView");
                quit_message_loop();
                return;
            }
            let mut window_delegate = ProofWindow::new(RefCell::new(view));
            window_create_top_level(Some(&mut window_delegate));
        }
    }
}

fn cache_dir() -> Result<PathBuf, String> {
    let local = std::env::var_os("LOCALAPPDATA")
        .ok_or("LOCALAPPDATA is unavailable on this Windows installation")?;
    let path = PathBuf::from(local).join("ClearLane").join("engine-proof");
    std::fs::create_dir_all(&path).map_err(|e| format!("CEF cache directory: {e}"))?;
    Ok(path)
}

fn start_url() -> String {
    // No privileged schemes are accepted from the command line.
    std::env::args()
        .find_map(|arg| arg.strip_prefix("--url=").map(str::to_owned))
        .and_then(|value| {
            let lower = value.to_ascii_lowercase();
            (lower.starts_with("https://") || lower.starts_with("http://")).then_some(value)
        })
        .unwrap_or_else(|| "https://example.com/".to_string())
}

pub fn run_main(
    main_args: &MainArgs,
    cmd_line: &CommandLine,
    sandbox_info: *mut u8,
) -> Result<(), String> {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let is_subprocess = cmd_line.has_switch(Some(&CefString::from("type"))) != 0;
    let outcome = execute_process(Some(main_args), None, sandbox_info);
    if is_subprocess {
        return if outcome >= 0 {
            Ok(())
        } else {
            Err(format!("CEF subprocess returned {outcome}"))
        };
    }
    if outcome != -1 {
        return Err(format!("unexpected CEF browser-process result: {outcome}"));
    }

    let cache = cache_dir()?;
    let cache = CefString::from(cache.to_string_lossy().as_ref());
    let settings = Settings {
        no_sandbox: i32::from(!cfg!(feature = "sandbox")),
        root_cache_path: cache.clone(),
        cache_path: cache,
        ..Default::default()
    };
    let mut app = ProofApp::new(start_url());
    if initialize(
        Some(main_args),
        Some(&settings),
        Some(&mut app),
        sandbox_info,
    ) != 1
    {
        return Err("cef_initialize failed".into());
    }
    run_message_loop();
    shutdown();
    Ok(())
}
