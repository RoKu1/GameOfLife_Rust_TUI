# Game of Life TUI — Upgrade and Stabilisation Plan

## Goal

Turn the current prototype into a reliable, approachable Conway's Game of Life terminal app. Running `cargo run` should open a setup TUI where the user can start from the current workspace (the default blank/editable grid), a curated pattern, or a random board. The simulation runs continuously until paused, returned to setup, or quit; it does not ask for a generation limit. The simulation view should offer predictable controls, visible progress, and a safe way back to setup.

This file records the implementation plan and original findings. Implementation has begun in the application source, so some earlier assessment items below describe the original prototype and may no longer reflect current code.

## Current-state assessment

- The README promises a TUI with random, step, autoplay, stop, and quit controls, but there is no setup screen, generation limit, pattern/workspace concept, or way to edit cells.
- `tui` 0.19 is the old `tui-rs` crate; its maintained successor is `ratatui`. The direct dependencies are pinned to old compatible lines: `crossterm` 0.25, `rand` 0.8, `log` 0.4.16, and `simplelog` 0.12.0. At the time of assessment, crates.io reports Ratatui 0.30.2, Crossterm 0.29.0, Rand 0.9.2, Log 0.4.29, and Simplelog 0.12.2 as current. Verify all versions again immediately before implementation.
- `Cargo.toml` says edition 2021, contains an unused/misleading Yew comment, and has `author` where Cargo expects `authors` (Cargo warns that `package.author` is unused).
- `grid.rs` mixes simulation, topology, and drawing; uses public non-idiomatic names; repeatedly allocates/clones neighbor vectors; and its next-generation updates happen during the scan. Since earlier cells are mutated before later cells are counted, the result is scan-order dependent rather than a correct synchronous Conway generation.
- Neighbor indices can wrap across rows because they are constructed as linear offsets without checking x/y boundaries. The intended boundary policy is not stated. Prefer bounded, non-wrapping finite grids initially, unless the product explicitly chooses toroidal wrapping.
- The test module only exercises a random simulation, asserts no behavior, and prints large output. It will not catch the evolution or edge bugs.
- `main.rs` does not guarantee terminal restoration on every error/panic path, uses `expect`, swallows/prints run errors after cleanup, and the auto-run interval disagrees with the README (200 ms in code, 100 ms in docs). The simulation API takes an unnecessary active boolean.
- The GitHub workflow uses `actions/checkout@v3`, tests/builds without formatting, lint, or a pinned toolchain, and no `rust-toolchain.toml` is present.
- The local `cargo test --all-targets` attempt is currently blocked before project tests by the host Apple linker / Command Line Tools SDK mismatch (`malformed file`, unsupported `arm64e.x1-*` architecture). This appears environmental, not a Rust source diagnostic; fix/upgrade the local Xcode Command Line Tools or validate on CI after code changes.
- README has no further reference links for cellular automata or patterns beyond the demo GIF. A ConwayLife wiki fetch was attempted but returned HTTP 403; pattern selections below are standard named Life patterns and should be checked against authoritative pattern references when implementation begins.

## Product decisions to lock before implementation

1. **Simulation boundary — decided:** use a finite grid; positions beyond its edges are dead. Neighbour counting must not wrap across rows or edges.
2. **Generation count — decided:** do not ask the user to enter a generation limit. The default experience runs continuously until paused/stopped/quit; a later optional bounded run mode may use an internal safe maximum, but it must not block the requested unlimited mode.
3. **“Workspace” meaning — decided:** the current workspace is the user's editable in-memory starting board for the run. Treat “mark/select current state” as selecting this board as the seed; no custom save/load format is required in the first release.
4. **Persistence — decided:** ship a built-in static pattern library only. Defer user pattern files, recent sessions, and persistence of custom workspaces.
5. **Grid dimensions and viewport — decided:** keep a fixed logical board; the viewport may clip/pan to show it in different terminal sizes. Keep model coordinates independent of terminal drawing.

## Proposed user flow

1. `cargo run` enters the terminal safely and opens a **New simulation** menu; no extra flags are required for the normal developer workflow.
2. Setup fields: start source (**Current workspace**, initially blank/editable, or **Pattern library**); pattern selection when relevant; optional speed; **Start** and **Quit** actions. Do not expose a generation-count input. Keyboard navigation, clear focus/selection, and help text are required.
3. Starting a pattern copies its immutable seed into the working board. Pattern names and short descriptions appear in the selector.
4. Simulation screen shows the board, selected starting pattern/workspace, generation counter, and running/paused state. The default mode runs indefinitely until the user pauses/stops/quits. Controls: pause/resume, single-step, reset to seed, clear, choose another seed/return to setup, and quit.
5. On exit or any application error, restore raw mode, alternate screen, mouse capture, and cursor visibility before returning control to the shell.

## Curated starter library (initial candidates)

Prefer a compact, deterministic catalogue of recognizable, small patterns with tests for their known behaviour:

- **Still lifes:** Block, Beehive, Loaf, Boat.
- **Oscillators:** Blinker, Toad, Beacon, Pulsar.
- **Spaceships:** Glider, Lightweight Spaceship (LWSS).
- **Methuselah / interesting long-lived seeds:** R-pentomino, Diehard, Acorn.

The initial catalogue can ship with 8–10 patterns: Block, Beehive, Blinker, Toad, Beacon, Pulsar, Glider, LWSS, R-pentomino, and Acorn. Store each as a named coordinate list or compact ASCII fixture with explicit dimensions/origin; avoid hardcoding coordinates in UI rendering. Include metadata and a short plain-language description. Verify well-known lifespan/period/displacement claims against a reliable reference before displaying them.

## Technical upgrade plan

### 1. Establish a clean baseline

- Preserve the existing uncommitted `.DS_Store` change; do not discard user work. Remove tracked OS/editor artifacts from version control deliberately in a separate hygiene change, and add ignore rules.
- Fix manifest metadata (`authors`, repository/description/license metadata if known) and remove the stale Yew comment.
- Add a small deterministic test baseline before changing the UI: canonical patterns and expected successor states, edge behavior, and finite-run stopping.
- Capture expected CLI usage and the chosen boundary/workspace semantics in README.

### 2. Upgrade Rust and dependency set

- Install/select the latest stable Rust toolchain at implementation time; add `rust-toolchain.toml` (stable channel, minimal profile) so contributors and CI use the same predictable baseline. Do not pin to a nightly toolchain without a concrete need.
- Raise edition to 2024 only after checking the current stable compiler and applying migration diagnostics; otherwise use the latest stable Rust with edition 2021 as a smaller-risk first step, then make the edition a separate change.
- Replace deprecated `tui` with maintained `ratatui`; migrate imports/widgets and confirm canvas support or use a custom widget that maps cells to terminal coordinates. Ratatui's latest release may require a recent Rust baseline; validate MSRV and features before selecting it.
- Upgrade Crossterm to the newest compatible release and use a single, deliberate version shared with Ratatui to avoid duplicate terminal backends.
- Upgrade Rand; use the modern `rand::rng()` API if applicable and allow an injectable/seedable RNG for deterministic tests and reproducible random boards.
- Keep `log`/`simplelog` only if there is a real logging requirement. Logging to stdout/stderr corrupts a TUI; either remove unused logging dependencies or direct logs to a file with an opt-in flag. Upgrade to the current compatible releases if retained.
- Run `cargo update`, inspect `cargo tree`, remove unused transitive/direct crates, and keep `Cargo.lock` checked in for this executable.
- Add a dependency update workflow/tool (e.g. Dependabot) only if maintainers want automated PRs; upgrades must still run the full validation matrix.

### 3. Repair and isolate the simulation model

- Replace `grid`/`Cell` implementation with an idiomatic `Board`/`CellState` model and explicit width/height. Keep rendering out of the model.
- Define topology and boundary behavior once. Compute each next state into a separate buffer, then swap buffers so every generation is simultaneous and deterministic.
- Use bounded coordinate iteration and count live neighbors without per-cell heap allocations. Select a representation based on measured needs: flat `Vec<CellState>` is a simple, clear baseline; optimize only after profiling.
- Add model operations: clear, randomize (seedable), toggle/set cell, load seed, advance one generation, run up to N generations, count live cells, and detect an empty/stable state if useful.
- Add focused tests: neighbor counts at corners/edges/interior; classic blinker two-step return; block stability; glider displacement; no scan-order dependence; random seed reproducibility; dimensions validation; and continuous-run/pause semantics.

### 4. Rebuild the TUI as explicit app states

- Use Ratatui with separated modules, for example `app`, `ui`, `input`, `simulation`, `patterns`, and terminal setup/cleanup.
- Model screens as states (`Setup`, `PatternPicker`, `EditingWorkspace`, `Running`, `Help`, `ConfirmQuit` as warranted) rather than nested key handling and booleans.
- Use a responsive layout with a board viewport and a status/control panel. Handle resize and small terminal dimensions without panics; show a friendly “terminal too small” message.
- Add keyboard navigation (arrows/j/k, Enter, Esc, Space) and retain simple documented shortcuts. Mouse support is optional; do not enable capture unless it is used.
- Ensure playback speed and continuous-run handling are app-state driven; simulation ticks must not block input/rendering.
- Use unlimited run by default, with no user-entered generation count. Keep rendering/input responsive during long sessions and make pause/stop immediately available.
- Make current workspace editable with a cursor and a toggle action; allow setup to mark/use the edited current board as the seed. Add reset/clear and easy return to pattern selection.
- Add readable styling, live-cell count, generation number, status, and visible hotkey hints; avoid tiny hard-coded layout percentages that waste space.

### 5. Terminal safety and errors

- Create a terminal guard/RAII wrapper so raw mode and alternate screen are restored on normal return and error paths. Handle panic restoration if supported by the chosen terminal setup.
- Propagate errors with context (an error crate is optional; keep the dependency list lean) instead of `expect`/debug-printing and then returning success.
- Only enable mouse capture if mouse editing is implemented. Always restore exactly the modes enabled.
- Test terminal-independent app transitions and rendering with Ratatui's test backend; reserve manual checks for real terminal behavior.

### 6. CI, quality, and documentation

- Update GitHub Actions to current supported action versions and stable toolchain; run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, and `cargo build --release`.
- Add a platform matrix at least for Linux and macOS; include Windows if Crossterm support is a project goal. Keep CI failures actionable and avoid relying on local-only environment assumptions.
- Update README with installation/run instructions, setup flow, supported keys, generation-limit semantics, boundary behavior, curated pattern names, troubleshooting, and the demo GIF.
- Add links to Rust installation/toolchain docs, Ratatui docs/migration, Crossterm docs, Conway's Life rules, and a reputable pattern reference once checked and reachable. Do not link to unverified sources.

## Suggested milestones and acceptance criteria

### Milestone A — Make core behavior correct

- Deterministic simulation model has an explicit topology and passes canonical unit tests.
- No per-cell mutation leaks into a generation currently being calculated.
- Reproducible board seeds are supported in tests.

### Milestone B — Modernize dependencies/toolchain

- Old `tui` dependency is removed and replaced with a compatible current Ratatui release.
- Direct dependencies are current at the time of merge, Cargo.lock is regenerated, and clean CI passes.
- CI and local instructions agree on the Rust stable toolchain.

### Milestone C — Deliver setup and pattern-selection experience

- Launch enters a setup TUI; user can choose either current workspace or a named built-in seed. No generation-count field is shown.
- Starting any catalogue entry loads the expected board exactly; invalid setup selections are explained and recoverable.
- Workspace editing can toggle cells and Start uses the marked board.

### Milestone D — Reliable run, controls, and release readiness

- Run indefinitely by default; pause/resume, single-step, reset, stop, return-to-setup, and quit work consistently. No generation-limit prompt appears.
- Generation number and running/completed state are visible and correct.
- Terminal is restored after normal quit and reported errors; resize/small-screen handling does not panic.
- README and CI describe/verify the actual behavior; tests, fmt, clippy, and release build pass.

## Recommended implementation order

1. Product decisions above are confirmed: finite dead-edge grid, unlimited default run with no user count input, editable in-memory current workspace, built-in pattern library, and fixed logical board.
2. Add tests and replace the incorrect simulation update algorithm before UI work.
3. Upgrade Rust/dependencies and CI in a reviewable commit; resolve the local Apple SDK toolchain issue separately from code failures.
4. Implement TUI state model and safe terminal guard.
5. Implement setup form, editable current workspace, and built-in pattern picker.
6. Implement continuous runs, run controls, generation/status display, resize behavior, and help.
7. Update README, demo, and run full quality checks on Linux/macOS (and Windows if supported).

## Implementation status (2026-10-10)

- Replaced the old `tui` dependency with Ratatui and upgraded Crossterm/Rand; added a stable toolchain file and release optimizations.
- Rebuilt the simulation with a flat fixed board and synchronous generation buffers, finite dead edges, centered built-in pattern seeds, and model-level tests.
- Added setup, pattern and random selection, current-workspace editing, continuous simulation, step/pause/reset/speed controls, and terminal cleanup handling.
- Updated README and CI. Local formatting and diff whitespace checks pass. Test/check/clippy execution is currently blocked by the host macOS SDK/linker (Command Line Tools SDK 27 contains `arm64e.x1-*` target entries rejected by the installed linker); resolve the local Xcode Command Line Tools before relying on local build results. CI should provide a clean-platform compile/test signal.
- The checked-in stable toolchain resolves to Rust 1.99.0 in this checkout; ordinary `cargo` and `rustc` commands honor it.
- Follow-up review found and fixed two behavior issues: starting from the workspace editor now always uses the edited board (including when editing a running pattern/random board), and starting/resuming playback resets its tick clock so it does not unexpectedly advance immediately.
- The terminal guard now owns the terminal before hiding the cursor, so its drop cleanup also runs if cursor hiding fails.
- The TUI now gives a clear size requirement (80 × 13) before laying out controls; README documents it.
- Follow-up review fixed the simulation `q` shortcut to quit as documented (Esc still returns to setup), added the selected seed name to the board title, and removed duplicate target ignore rules. The manifest now includes the repository URL.
- Validation after the Command Line Tools installation: plain `cargo run` now works with a project-local Cargo `SDKROOT` default pointing to the generic `MacOSX.sdk` symlink (26.5 on the current host). `cargo fmt --all -- --check`, `cargo test --all-targets` (12 passed), Clippy with `-D warnings`, `cargo build --release`, and launch/quit smoke tests all pass without an environment prefix. The runtime smoke test also exposed and fixed a busy redraw loop in setup/paused mode; idle polling is now capped at 10 redraws per second, while active simulation keeps its selected tick rate. The selected SDK 27.0 still fails in Apple's linker on `arm64e.x1-*` stub entries before project compilation; the workaround is documented in README. Remaining checks are confirming the CI matrix is green and manually exercising editor-to-run/reset and small-terminal resize flows.

## Launch simplicity and performance expectations

- `cargo run` should be the simple documented launch command: it starts the TUI with sensible defaults and needs no arguments. Do not require a separate setup command or mandatory flags.
- Provide a production/performance run path using Cargo's optimized profile, for example `cargo run --release`. Document both: `cargo run` for development and `cargo run --release` for the smoothest/highest-throughput experience. Keep both paths supported.
- Measure before adding complexity. The initial performance baseline should profile simulation stepping separately from terminal rendering on a representative fixed board. Use release-mode measurements; debug builds are expected to be slower.
- Keep the board model in a compact flat buffer and compute each generation with minimal allocation. Render only the visible viewport and avoid redrawing unchanged decoration where Ratatui's normal diffing already handles it.
- Prefer stable, portable Rust optimizations and optimized release settings over machine-specific compiler flags. If profiling justifies it, tune `[profile.release]` (e.g. thin LTO and one codegen unit) and record the trade-off in build time/binary size; do not promise gains without measurement.
- Add a lightweight repeatable benchmark for generation throughput if the implementation makes performance a primary goal. Compare debug and release runs, and optionally include Criterion only if a benchmark suite provides enough value to justify the dependency.

## External references to verify during implementation

- Rust toolchain: <https://rustup.rs/> and <https://doc.rust-lang.org/book/ch01-01-installation.html>
- Rust edition migration: <https://doc.rust-lang.org/edition-guide/>
- Ratatui documentation: <https://ratatui.rs/> and <https://docs.rs/ratatui/latest/ratatui/>
- Crossterm documentation: <https://docs.rs/crossterm/latest/crossterm/>
- Conway's Game of Life rule overview: <https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life>
- ConwayLife pattern reference (access was blocked in this assessment; validate links/content from a normal browser before relying on it): <https://www.conwaylife.com/wiki/>
