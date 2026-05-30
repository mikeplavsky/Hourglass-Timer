# First Day Guide

## What Is This?

Hourglass Timer is a small interactive countdown timer drawn as a visual hourglass, built in Rust on the [Bevy](https://bevyengine.org/) game engine. You set a duration, hit start, and watch sand flow from the top bulb to the bottom as time runs out. You can recolor the sand, switch between four hourglass shapes (or morph through them), and run it natively or in a browser. It was originally "vibe coded" (see [[README.md|README.md]]) and has since grown a unit-test suite around its core logic.

## Get It Running

From the project root (commands per [[CLAUDE.md|CLAUDE.md]] and project memory):

```bash
# Native, with dev hot-reload
cargo run

# Build (release, no dev features)
cargo build --release --no-default-features

# Web / WASM
./build_wasm.sh
cd wasm && python -m http.server 8080   # open http://localhost:8080

# Tests
cargo test --no-default-features         # run all
cargo clippy                             # lint
cargo fmt                                # format
```

## Where to Start Reading

1. Start with [[HOME]] for the big picture.
2. Read [[architecture/overview]] for how the plugins and resources connect (has a component diagram).
3. Browse [[features/overview]] to see what the app does.
4. Pick a feature and follow its links into the code — e.g. [[features/countdown-timer]] → [[modules/timer]] → [[flows/countdown-tick]].

## Key Concepts

You need five ideas to be productive here:

1. **Bevy ECS plugins** — the app is `AppPlugin` composing feature plugins ([[modules/app]]). Each plugin registers *systems* that run on `Startup`, `PostStartup`, or every frame (`Update`).
2. **Two shared resources** — `HourglassConfig` (appearance) and `TimerState` (countdown) in [[modules/resources]]. **All cross-module communication goes through these**, not through direct calls. See [[patterns#Resource-mediated communication]].
3. **Logic vs. visual split** — the countdown only mutates `TimerState`; a separate system mirrors it into the visual `Hourglass`. See [[flows/countdown-tick]].
4. **Recreate-on-change** — changing shape/color despawns and rebuilds the hourglass entity. This is the most surprising mechanism; read [[flows/appearance-recreation]] early.
5. **`bevy_hourglass`** — the external crate that does the actual hourglass mesh, sand, and flip animation. This app *configures* it. See [[modules/hourglass]].

## Common Tasks

| Task | Where to Look |
|------|--------------|
| Add a new hourglass shape | [[modules/resources]] (`HourglassShape` enum) + `get_main_shape_config`/`get_mini_shape_config` in [[modules/hourglass]] |
| Add a color swatch | `COLOR_PALETTE` in [[modules/resources]] |
| Change countdown behavior | [[modules/timer]] (`tick_countdown`) |
| Adjust the click vs. drag threshold | `DragState::drag_threshold` in [[modules/hourglass]] |
| Tweak the morph speed | `cycle_time` in `update_morphing_shape`, [[modules/hourglass]] |
| Understand the test situation | [[references/test-coverage]] |
| Change the web build feature set | [[Cargo.toml\|Cargo.toml]] + [[build_wasm.sh\|build_wasm.sh]], see [[features/web-build]] |

## A Word of Caution

Per the README this was developed through LLM prompting — "there be dragons." A few quirks are documented and even pinned by tests (e.g. `format_time` doesn't guard negatives; the morph throttle comment disagrees with its constant). The biggest untested area is the **side effect that picking a color/shape restarts the timer** — keep it in mind when editing the panels. See [[references/test-coverage#Biggest coverage gaps]].
