use cef::{self, sys};

/// Entry point used by the CEF Windows sandbox bootstrap executable.
/// Called by the official CEF bootstrap; never call from an ordinary Rust main().
#[unsafe(no_mangle)]
unsafe extern "C" fn RunWinMain(
    instance: sys::HINSTANCE,
    _command_line: *const u8,
    _command_show: i32,
    sandbox_info: *mut u8,
) -> i32 {
    let _ = cef::api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = cef::MainArgs { instance };
    let args = cef::args::Args::from(args);
    let Some(command_line) = args.as_cmd_line() else {
        return 1;
    };
    match crate::runtime::run_main(args.as_main_args(), &command_line, sandbox_info) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("ClearLane CEF startup failed: {message}");
            1
        }
    }
}
