<!-- wiki:sources: src/main.rs -->

# App (Root Plugin)

## Responsibility

The composition root. Defines `AppPlugin`, which wires Bevy's `DefaultPlugins` together with this project's three feature plugins, initializes the two shared resources, and spawns the camera. There is no business logic here — it is purely assembly.

## Where It Lives

[[src/main.rs|src/main.rs]]

## Key Files

| File | Purpose |
|------|---------|
| [[src/main.rs\|main.rs]] | `main()`, `AppPlugin`, `spawn_camera`, and the `mod` declarations for the whole crate. |

## What It Does

`main()` is a one-liner: `App::new().add_plugins(AppPlugin).run()`. It returns `AppExit`, the Bevy idiom for propagating a clean exit code.

`AppPlugin::build` does four things, in order:

1. **Adds `DefaultPlugins`**, overriding `WindowPlugin` so the primary window is titled `"Hourglass Timer"` and has `fit_canvas_to_parent: true` (important for the [[features/web-build|web build]] — the canvas tracks its container size).
2. **Initializes resources** via `init_resource::<HourglassConfig>()` and `init_resource::<TimerState>()` — both rely on their `Default` impls in [[modules/resources]].
3. **Adds the feature plugins**: [[modules/hourglass|`HourglassPlugin`]], [[modules/timer|`TimerPlugin`]], and [[modules/ui-layout|`UIPlugin`]] (which itself nests the four UI sub-plugins).
4. **Registers `spawn_camera`** as a `Startup` system.

`spawn_camera` spawns a single `Camera2d` named `"Camera"`. Everything in the app — the world-space hourglass *and* the screen-space UI — is rendered through this one 2D camera. Several systems query it via `Query<(&Camera, &GlobalTransform)>` to convert cursor positions to world coordinates (see [[flows/click-vs-drag]]).

## Module Declarations

`main.rs` declares the crate's module tree: `hourglass`, `resources` (public), `timer`, and `ui`. Only `resources` is `pub` because it is the shared vocabulary every other module imports.

## Features Supported

This module doesn't implement a user-facing feature directly; it enables all of them by composing the plugins that do. It most directly underpins [[features/countdown-timer]] (resource init + window setup).

## Dependencies

- `bevy` — `App`, `Plugin`, `DefaultPlugins`, `Camera2d`, `Window`.
- [[modules/resources]] — the two resources it initializes.
- [[modules/hourglass]], [[modules/timer]], [[modules/ui-layout]] — the plugins it adds.

## Used By

Nothing — it is the top of the tree. Bevy's runtime calls into it.

## Tests

No unit tests. Plugin composition is verified implicitly by the app building and running; there is no pure logic here to test. See [[references/test-coverage]].

## Related Pages

- [[architecture/overview]]
- [[flows/startup]]
- [[code-index/entry-points]]
