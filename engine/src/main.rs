//! The normal Windows distribution uses the CEF sandbox bootstrap executable.
#[cfg(all(target_os = "windows", not(feature = "sandbox")))]
fn main() -> Result<(), String> {
    let args = cef::args::Args::new();
    let command_line = args.as_cmd_line().ok_or("CEF command-line initialization failed")?;
    clearlane_engine::runtime::run_main(args.as_main_args(), &command_line, std::ptr::null_mut())
}

#[cfg(all(target_os = "windows", feature = "sandbox"))]
fn main() {
    eprintln!("Use the bundled clearlane-engine.exe produced by scripts/build-engine.ps1 (sandbox bootstrap).");
    std::process::exit(1);
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("This engine proof is Windows-only. Cross-platform support is not part of this experiment.");
    std::process::exit(1);
}
