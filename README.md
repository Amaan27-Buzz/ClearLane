# ClearLane

A fast, lightweight, user-first Chromium browser built around a Rust-native core, native ad/tracker blocking, and an agent-ready foundation.

> **Current stage:** Slice 1.5, Gate A — native shell prototype ready for Windows visual review. Chromium rendering, real Shields, and performance gates are not implemented yet. The failed first browser experiment remains reference-only.

## Goals

- Be an excellent everyday browser before becoming an AI product.
- Keep the common browsing path simple while retaining deep, optional features.
- Use Brave-style native ad/tracker blocking.
- Treat speed, memory use, startup time, idle cost, and tab lifecycle as measurable product features.
- Later become a low-token browser runtime for Codex, Claude Code, MCP, and Playwright-style agents.

## Build plan

ClearLane still uses a small number of substantial implementation stages. Slice 1.5 exists only because Slice 1 failed its manual gate and requires a deliberate reset.

| Slice | Outcome | Status |
| --- | --- | --- |
| 1. Browser Alpha — first attempt | Runnable Rust/CEF browser experiment | **Failed manual gate; reference only** |
| 1.5. Browser Alpha Reset | Shell prototype, then CEF, Shields and real efficiency gates | **Gate A prototype built; visual approval pending** |
| 2. Everyday Browser | Full human browser: customization, Spaces/groups, split view, history, bookmarks, downloads, profiles/incognito, permissions, lifecycle and performance hardening | Planned |
| 3. Agent Runtime | CDP/Playwright compatibility, structured page state/deltas, MCP interface and token/latency benchmarks | Planned |

**Superagent remains out of scope until the browser itself is good enough to use daily.**

## Engineering rules

- Simple modules before services or frameworks.
- Reuse proven libraries instead of rebuilding solved systems.
- Chromium/CEF types stay behind one integration boundary.
- No speculative abstractions, microservices, generic event buses, or unnecessary crates.
- Research upstream documentation, maintained examples, issues, and alternatives before committing to a difficult integration.
- When an approach repeatedly fails, isolate the problem and reconsider the approach instead of brute-forcing patches.
- No performance claim without reproducible competitor measurements.
- UI quality is a product gate, not post-CI polish.
- UI decisions come from explicit requirements, reference study, and visual/manual validation — not generic SaaS/AI styling.

## Try the current shell (not yet a browser)

From Windows PowerShell with Rust installed:

```powershell
git clone --branch slice-1-5-browser-alpha-reset https://github.com/Amaan27-Buzz/ClearLane.git
cd ClearLane
powershell -ExecutionPolicy Bypass -File .\scripts\run-shell.ps1
```

Or use the `clearlane-gate-a-shell` artifact from [Windows CI](https://github.com/Amaan27-Buzz/ClearLane/actions/runs/40711777434). Navigation, webpage content and Shields in this build are demo-only, not connected to Chromium. See [draft PR #1](https://github.com/Amaan27-Buzz/ClearLane/pull/1).

## Docs

- [Product and UX](docs/PRODUCT.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Slice 1.5 reset](docs/SLICE_1_5.md)
- [Roadmap and acceptance gates](docs/ROADMAP.md)
- [Performance discipline](docs/PERFORMANCE.md)
- [Agent instructions](AGENTS.md)

## License

ClearLane is licensed under the [Mozilla Public License 2.0](LICENSE).
