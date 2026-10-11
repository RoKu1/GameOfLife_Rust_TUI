# Browser / GitHub Pages Plan

## Answer

Yes. This project can run from a static GitHub Pages URL. The important architectural change is to separate the **portable Game of Life engine** from the current terminal UI:

```text
shared Rust engine
  ├── native binary: Ratatui + Crossterm TUI (`cargo run`)
  └── web target: WebAssembly engine + browser renderer (GitHub Pages)
```

GitHub Pages only serves static files, so the browser build must contain the WASM module and JavaScript/HTML assets. No server is required.

## Recommendation

### Phase 1: WASM + Canvas 2D

Use the existing Rust `Board` and pattern model as a library compiled to WebAssembly. Build a small browser frontend using TypeScript and Canvas 2D.

This is the best first browser target because:

- A fixed 100 × 100 board is only 10,000 cells; WebGPU is unnecessary for this workload.
- Canvas 2D is widely supported, simple to deploy, and easy to make responsive.
- The engine remains Rust and can be tested identically on native and web.
- The UI can offer smooth animation with `requestAnimationFrame`, pause/resume, stepping, reset, pattern selection, randomization, speed control, and workspace editing.
- WebGPU can be added later behind a renderer interface without changing the simulation rules or pattern library.

### Phase 2: Optional WebGPU renderer

Add WebGPU only if profiling shows Canvas 2D is insufficient or if the project intentionally wants a GPU-rendering demonstration. WebGPU is viable on a static site, but it adds:

- asynchronous adapter/device initialization;
- browser and hardware compatibility handling;
- shader code and a second rendering path;
- a fallback requirement for browsers without WebGPU;
- more deployment/debugging complexity.

For this board size, WebGPU would improve the visual technology more than actual user-visible simulation performance.

## Target repository structure

Refactor toward a Cargo workspace or a library-plus-binary layout:

```text
src/
  lib.rs                 # shared engine exports
  engine.rs              # Board, stepping, finite-edge rules
  patterns.rs            # built-in pattern definitions
  main.rs                # native Ratatui application
web/
  index.html
  src/main.ts            # browser UI and canvas renderer
  package.json
  vite.config.ts
  public/                # static assets, if needed
.github/workflows/
  rust.yml               # native quality checks
  pages.yml              # build and deploy GitHub Pages
```

Keep the engine free of `std::io`, terminal APIs, Ratatui, and Crossterm. Native-only dependencies should be moved behind the binary target or target-specific dependencies. The web target should not compile `main.rs` or any terminal code.

## WASM API design

Expose a small stable interface rather than binding the entire Rust object graph:

- `new_board(width, height)`
- `clear()`
- `set_cell(x, y, alive)` / `toggle_cell(x, y)`
- `load_pattern(pattern_id)`
- `randomize(seed)` or `randomize()`
- `step()`
- `generation()`
- `live_count()`
- `cells()` as a compact byte/typed-array view (`Uint8Array`), not a large JSON object
- `patterns()` returning IDs/names/descriptions

Prefer `u8` or a packed bit representation for the browser-facing cell buffer. The current `Vec<bool>` is compact internally but is not an ideal JavaScript/WASM memory contract. A straightforward first version can copy 10,000 bytes per frame; optimize to a packed buffer only if measurement warrants it.

Use `wasm-bindgen` for the JS boundary. Keep the actual rules in ordinary Rust methods so the existing native unit tests continue to cover the important behavior. Add WASM-target tests only for bindings and browser-specific behavior.

## Browser UI behavior

Match the TUI’s core behavior while adapting controls to the browser:

1. Landing page opens with the setup panel.
2. Choose Current Workspace, Pattern Library, or Random Board.
3. Edit cells by clicking/tapping or dragging on the board.
4. Start an unlimited simulation; pause/resume, single-step, reset, clear, and change speed.
5. Display generation, live-cell count, selected pattern, and finite-edge rule.
6. Resize the canvas while keeping the logical 100 × 100 board unchanged.
7. Support keyboard shortcuts as a bonus, but do not make them the only interaction method.

The web version should not try to emulate a terminal. It should share the model and behavior, not the Ratatui layout.

## Build choices

### Recommended: Vite + wasm-bindgen

- Rust builds the WASM package with `wasm-pack` or `wasm-bindgen-cli`.
- TypeScript/Vite owns the browser UI and asset bundling.
- Vite emits a static `dist/` directory suitable for GitHub Pages.
- Use a relative/base path configuration because the Pages URL is normally:
  `https://<user>.github.io/<repository>/`.

This approach provides the clearest separation between native Rust and browser presentation.

### Alternative: Trunk

Trunk can produce a static WASM site from a Rust web binary and avoids a separate Node frontend. It is attractive if the UI should also be written in Rust using a framework such as Leptos. It is less attractive here because the current TUI code is not reusable as browser UI, and introducing a Rust web UI framework would be a larger rewrite than a small TypeScript canvas frontend.

Do not use `ratatui` or `crossterm` in the browser build. They target terminal interaction, not normal browser layout/canvas input.

## GitHub Pages deployment

Add a Pages workflow that:

1. Checks out the repository.
2. Installs the pinned stable Rust toolchain and WASM target.
3. Installs the web package manager/dependencies with a lockfile.
4. Builds the Rust WASM package.
5. Builds the frontend with the repository base path.
6. Uploads the generated `web/dist` artifact.
7. Deploys through GitHub Pages actions.

Use the official Pages artifact/deploy actions and configure the repository’s Pages source as **GitHub Actions**. The workflow should run on pushes to `main` and optionally on manual dispatch.

The generated site must include a fallback for direct navigation, though a single-page app can initially use hash-free navigation with only one page. Avoid runtime API calls so the site remains fully static and works offline after initial load.

## Rust and dependency changes

- Move current `Board`, patterns, dimensions, and rules into `src/lib.rs` or engine modules.
- Keep the native binary’s Ratatui/Crossterm dependencies target-specific where practical.
- Add `wasm-bindgen` only to the web-facing crate/module.
- Configure `rand`/`getrandom` for WASM browser randomness, or use an explicit seed and browser-provided randomness at the boundary. Keep deterministic seeded randomization for tests.
- Add `wasm32-unknown-unknown` to local/CI validation.
- Keep `rust-toolchain.toml` as the stable channel so both native and WASM builds use the same Rust version.
- Keep the optimized native release profile. Add a web release build with size optimization after correctness is established; use `wasm-opt` only after measuring output and build complexity.

## Performance plan

1. Measure Rust stepping throughput independently from rendering.
2. Measure Canvas 2D rendering at the selected animation speed.
3. Render only the logical board/cells and avoid rebuilding expensive DOM nodes per cell.
4. Use one canvas and draw rectangles or an `ImageData` buffer.
5. Use `requestAnimationFrame` for rendering and a fixed simulation timestep so browser frame rate does not alter Game of Life rules.
6. Add WebGPU only if profiling identifies rendering as a real bottleneck or the visual goal requires it.

The 100 × 100 board should be fast with a byte buffer and Canvas 2D. More important than GPU acceleration is avoiding per-cell DOM elements and unnecessary WASM/JavaScript conversions.

## Testing and acceptance criteria

### Shared engine

- Existing native tests pass for block, blinker, finite edges, synchronous updates, and pattern loading.
- The same engine API drives native and web simulations.
- Seeded random boards are reproducible.

### Web build

- `wasm-pack build`/web build succeeds in CI.
- The generated site is purely static and contains no backend dependency.
- Browser smoke test loads the WASM module, starts a pattern, advances generations, and updates generation/live-cell counters.
- Canvas editing works with mouse and touch.
- Direct `cargo run` still opens the native TUI unchanged.

### Deployment

- A push to `main` publishes the site to the repository’s GitHub Pages URL.
- Repository base paths work correctly; assets do not assume `/` as the site root.
- A Pages build failure is visible in GitHub Actions.

## Proposed implementation milestones

1. **Extract engine:** move simulation and patterns into a reusable library; preserve native TUI behavior.
2. **Add WASM bindings:** expose board operations and a typed cell buffer; add WASM build locally and in CI.
3. **Build browser MVP:** Vite/TypeScript setup screen, canvas board, controls, patterns, workspace editing.
4. **Deploy static site:** add GitHub Pages workflow and repository-base-path configuration.
5. **Polish:** responsive layout, touch input, accessibility, loading/error states, favicon/readme link, and browser smoke tests.
6. **Evaluate WebGPU:** profile first; add a fallback-capable renderer only if the measured or product need justifies it.

## Implementation status (2026-10-11)

- The shared Rust library and `WebBoard` WASM bindings are implemented.
- A Canvas 2D/Vite browser frontend is implemented under `web/` with pattern selection, workspace editing, random boards, continuous playback, stepping, reset, and speed controls.
- GitHub Pages deployment is implemented in `.github/workflows/pages.yml`; the Rust CI workflow also checks the WASM library target.
- Local WASM compilation and the production browser bundle pass with `cargo check --target wasm32-unknown-unknown --lib` and `cd web && npm run build`. The final bundle is emitted to `web/dist`.
- WebGPU remains intentionally deferred because the 100 × 100 board does not need GPU rendering. Canvas 2D is the first renderer.

## Decision

Proceed with **shared Rust engine + wasm-bindgen + TypeScript/Vite + Canvas 2D** for the first browser release. Keep WebGPU as an optional renderer behind the same board/rendering interface, not as a prerequisite for the static site.

## References

- Rust WASM: <https://www.rust-lang.org/what/wasm>
- wasm-bindgen guide: <https://rustwasm.github.io/docs/wasm-bindgen/>
- wasm-pack: <https://rustwasm.github.io/wasm-pack/>
- GitHub Pages custom workflows: <https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages>
- WebGPU API: <https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API>
- Canvas 2D API: <https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D>
