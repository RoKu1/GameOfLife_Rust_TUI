# Game of Life — Rust TUI

A keyboard-driven terminal interface for Conway's Game of Life, written in Rust with Ratatui and Crossterm.

![Build](https://github.com/RoKu1/GameOfLife_Rust_TUI/actions/workflows/rust.yml/badge.svg?branch=main)

## Run

Install Rust with [rustup](https://rustup.rs/), clone the repository, and run:

```sh
cargo run
```

For a smooth, optimized simulation, use the release profile:

```sh
cargo run --release
```

Both commands open the TUI directly. `cargo run` is convenient while developing; release mode enables LTO and full optimizations for everyday use. The logical board is fixed at 100 × 100 cells. The simulation runs continuously until paused or stopped; there is no generation-count prompt.

### macOS linker / SDK errors

If compilation fails with `ld: ... malformed file` and an “unknown architecture” entry such as `arm64e.x1-macos` in `MacOSX*.sdk/usr/lib/*.tbd`, the problem is the selected Apple SDK/linker toolchain, not this project's Rust source. Check the active developer directory and SDK with:

```sh
xcode-select -p
xcrun --sdk macosx --show-sdk-path
clang --version
```

Install or update Xcode Command Line Tools (or full Xcode), then select that installation with `sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer` for full Xcode, or `sudo xcode-select --switch /Library/Developer/CommandLineTools` for Command Line Tools. Re-open the terminal and retry `cargo run`.

This repository configures Cargo to use the Command Line Tools' generic `MacOSX.sdk` symlink when `SDKROOT` is not already set. On the affected machine it points to SDK 26.5, avoiding the malformed SDK 27.0 stubs, so the normal `cargo run` command works. The setting is project-local and does not modify system SDK files. If that symlink is unavailable or points to another incompatible SDK, set `SDKROOT` to a compatible installed SDK for the command, for example:

```sh
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk cargo run
```

The same prefix works with `cargo test --all-targets`, `cargo clippy`, and `cargo build --release`. Avoid editing SDK `.tbd` files or forcing another architecture as a workaround.

## Start a simulation

At launch, choose one of:

- **Current workspace** — starts with the current in-memory board. Press **E** to edit it first.
- **Pattern library** — select a built-in pattern with **←/→**.
- **Random board** — generates a fresh random starting state.

The pattern library includes the original classics plus ten new additions: Gosper glider gun, Pentadecathlon, Tumbler, Middleweight and Heavyweight spaceships, Diehard, Queen bee shuttle, Puffer 1, Pufferfish, and Kok's galaxy. Browse them with **←/→** before starting a simulation. The gun emits gliders, spaceships and puffers move across the board, the shuttle and oscillators continually reshape, and Diehard eventually burns out.

### Setup controls

| Key | Action |
| --- | --- |
| `↑` / `↓` or `k` / `j` | Choose a starting source |
| `←` / `→` or `h` / `l` | Browse patterns |
| `e` | Edit the current workspace |
| `Enter` | Start |
| `q` / `Esc` | Quit |

### Workspace editor

| Key | Action |
| --- | --- |
| Arrow keys or `h` / `j` / `k` / `l` | Move the cursor |
| `Space` or `x` | Toggle a cell |
| `c` | Clear the workspace |
| `Enter` | Start from this state |
| `Esc` | Return to setup |

### Simulation controls

| Key | Action |
| --- | --- |
| `Space` or `p` | Pause / resume |
| `s` | Advance one generation and pause |
| `r` | Reset to the selected starting state |
| `e` | Pause and edit the current board |
| `+` / `-` | Increase / decrease simulation speed |
| `Esc` | Return to setup |
| `q` | Quit |

The TUI needs a terminal at least 80 × 13 cells. The board has finite edges: cells beyond the 100 × 100 board are dead, and patterns do not wrap around. If the terminal is smaller than the board, the view shows a viewport around the cursor while editing and the board center during simulation.

## Development checks

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

CI runs these checks on Linux, macOS, and Windows using the latest stable Rust toolchain. The release build uses thin LTO and one codegen unit to favor runtime performance; debug builds intentionally prioritize iteration time.

## Learn more

- [Rust installation](https://rustup.rs/) · [Rust edition guide](https://doc.rust-lang.org/edition-guide/)
- [Ratatui](https://ratatui.rs/) · [Crossterm](https://docs.rs/crossterm/latest/crossterm/)
- [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life)
- [ConwayLife Wiki](https://www.conwaylife.com/wiki/)

## Demo

![Game of Life TUI demo](./working_tui_gol.gif)
