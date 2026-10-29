# Chromium runtime proof (not yet the ClearLane browser)

## Why this branch exists
The two previous frontend mockups failed the product-quality goal. Their visual styling is not a foundation to keep patching. Before creating another browser shell, prove the supported CEF runtime, sandbox bundle and rendering lifecycle using the upstream cef-rs model.

This proof intentionally has **no fabricated tabs, navigation toolbar or ad counters**. It creates one genuine Chromium page via CEF Views/Alloy on Windows, and uses the official Windows sandbox bootstrap. It is **not** a usable everyday browser or an approved UI design.

## Implementation boundary
- `engine/` contains the real Chromium process/CEF runtime proof only.
- No CEF types appear in the planned browser domain layer.
- Do not merge this as product complete. Only promote it once compiled, launched and manually validated in Windows.
- Prefer integrating the eventual Rust UI host after the CEF runtime is real and the host/child-window ownership strategy is proven.
- Do not choose an egui, GPUI, Slint or CEF Views final shell based on screenshots or a framework reputation. Compare input, IME, popup stacking, DPI, accessiblity, graphics composition and memory overhead using a minimal practical case.

## Research (October 2026)
- CEF API 156: https://docs.rs/crate/cef/156.2.0+156.0.14
- cef-rs and official bundle tool: https://github.com/tauri-apps/cef-rs
- `cef-dll-sys::get_cef_dir()` uses `CEF_PATH` or the build script's `OUT_DIR`. Installing `bundle-cef-app` separately gives it a different `OUT_DIR`; set a shared versioned `CEF_PATH` for both builds. Both scripts now do so. The CEF binary is downloaded during the Rust build.
- CEF message loop integration: https://chromiumembedded.github.io/cef/general_usage.html
- CEF multi-threaded UI loop and external message pumps have different threading/CPU constraints. This prototype stays with the standard CEF event loop until we can test host-owned integration safely.
- Avoid the unproven GPU CEF-to-wgpu welding project for production until sandbox and input support are established: https://docs.rs/welding/latest/welding/

## Acceptance to move beyond this proof
1. CI compiles and packages a real sandboxed executable (not merely `cargo check`).
2. A Windows run displays **one real HTTPS site**, with functional mouse, keyboard, resizing, links, and video playback test.
3. Close/relaunch does not hang or leave unexpected processes.
4. Default sandbox stays enabled; never use `--no-sandbox`, `--single-process`, or disabled site isolation to make it work.
5. Capture logs and errors instead of restarting/rewriting blindly.
6. Only then build one integrated browser shell with real navigation and tabs, and compare layout/UX against current browsers before presentation.

## Windows setup
Requires Rust (stable, MSVC toolchain), Visual Studio C++ build tools and PowerShell.

```powershell
git fetch origin
git switch feat/chromium-browser-foundation
git pull --ff-only
powershell -ExecutionPolicy Bypass -File .\scripts\build-engine.ps1
.\target\engine-proof\clearlane_engine.exe --url=https://example.com/
```

### Windows local build path fix (MSVC C1083)

If CEF fails with `fatal error C1083: Cannot open compiler generated file: '': Invalid argument`, the checkout path may be too deeply nested for MSVC/CMake's generated object files. The user-reported path under `D:\\Programming\\...\\ClearLane\\engine\\target\\release\\build\\...` exceeded the traditional 260-character path limit for some generated objects.

`scripts/build-engine.ps1` now sets a short `CARGO_TARGET_DIR` automatically (for example, `D:\\clearlane-build`). Keep the project in its existing directory; **do not move it, delete the downloaded CEF SDK, enable unsafe browser flags, or change the Windows registry**.

If you already set `CARGO_TARGET_DIR`, use a similarly short location or clear that environment variable for this shell. The script prints the chosen path before compiling. The finished bundle remains at `target/engine-proof/clearlane_engine.exe` inside the repository. The helper also checks native-command exit codes immediately, so an unsuccessful CEF compile cannot be mistaken for a runnable bundle.

If C1083 still occurs **after the short-path fix**, capture the first failing object path and full compiler error. Investigate MSVC parallel compilation, antivirus/file locks and temporary directories as separate hypotheses rather than arbitrarily editing CEF flags.

The PowerShell script defaults `CEF_PATH` to `%LOCALAPPDATA%/ClearLane/cef-sdk` (override with your own `CEF_PATH`). The first Rust compile downloads the matching CEF distribution; the separately installed bundle tool reuses exactly that distribution. First setup can be large. Do not remove or overwrite this cache while the build runs. Alternatively download the `clearlane-cef-engine-proof` CI artifact.

No competitive speed or memory claims are made from this proof.
