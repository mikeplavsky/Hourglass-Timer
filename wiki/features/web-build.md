<!-- wiki:sources: build_wasm.sh, Cargo.toml -->

# Web Build (WASM)

## What It Does

Ships the same app to the browser by compiling to WebAssembly. The live site runs this build. The native and web targets share all source code; only the Bevy feature set and build flags differ.

## User Journey (developer)

1. Run `./build_wasm.sh`.
2. The script installs `wasm-bindgen-cli` and the `wasm32-unknown-unknown` target if missing, builds the crate for WASM, and generates JS bindings into `wasm/`.
3. Serve it: `cd wasm && python -m http.server 8080`, open `http://localhost:8080`.

## Implementation

| Component | File | Role |
|-----------|------|------|
| build script | [[build_wasm.sh\|build_wasm.sh]] | Toolchain checks, `cargo build --target wasm32-unknown-unknown`, `wasm-bindgen`. |
| dependencies | [[Cargo.toml\|Cargo.toml]] | WASM-specific Bevy feature set + `getrandom` js backend. |

## How the targets differ

`Cargo.toml` defines a `[target.wasm32-unknown-unknown.dependencies]` block that pins Bevy to a **minimal, no-default-features feature set** (`bevy_render`, `bevy_ui`, `bevy_sprite`, `bevy_winit`, `webgl2`, etc.) suitable for the browser, plus `getrandom` with the `js` backend (the browser's RNG source). The native build instead uses the `dev_native` feature with hot reloading and Wayland support.

`build_wasm.sh` compiles with `--no-default-features` and explicitly lists those same Bevy features, then runs `wasm-bindgen --target web` to emit the `.js` glue and `.wasm` next to a hosting page in `wasm/`. The generated `*.js`/`*.wasm` are gitignored.

The window is configured with `fit_canvas_to_parent: true` in [[modules/app|`AppPlugin`]], which is what lets the canvas resize to its container in the browser.

## Build profiles

`Cargo.toml` defines a `web-release` profile (inherits `release`, `opt-level = "s"` for size, strips debug info) used by `bevy run web`, alongside `release` (LTO thin, codegen-units 1) and a fast-iteration `ci` profile.

## Architecture Decisions

- **One codebase, two feature sets** — no `#[cfg]` branching in the app logic; the difference is entirely in `Cargo.toml` feature selection and build flags.
- **`getrandom` js backend** is required because the [[modules/color-panel|random color]] / [[modules/shape-panel|random shape]] features use `rand`, which needs a browser entropy source on WASM.

## Open Questions

- The `Cargo.toml` comments warn that a future `getrandom` v0.3+ transitive dependency would break web builds (it needs the `wasm_js` backend + an extra rustflag). Currently pinned to v0.2 with the `js` feature.
