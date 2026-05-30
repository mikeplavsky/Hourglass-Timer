# Hourglass Timer

An interactive countdown timer rendered as a visual hourglass, built in Rust on the [Bevy](https://bevyengine.org/) game engine (v0.16) using the [`bevy_hourglass`](https://crates.io/crates/bevy_hourglass) crate. Set a duration, start it, and watch sand flow from the top bulb to the bottom; customize the sand color and hourglass shape along the way. The same code runs natively and in the browser as WebAssembly. New here? Start with the [[onboarding/first-day-guide]].

## Architecture

The app is a composition of Bevy ECS plugins assembled by [[modules/app|`AppPlugin`]]. Plugins never call each other — they communicate entirely through two shared resources defined in [[modules/resources]]: `HourglassConfig` (appearance) and `TimerState` (countdown). The countdown logic in [[modules/timer]] only mutates state, while [[modules/hourglass]] mirrors that state into the visual hourglass and rebuilds it when appearance changes. The UI is split across [[modules/ui-layout|layout scaffold]], [[modules/color-panel|color]], [[modules/shape-panel|shape]], [[modules/timer-panel|timer]], and [[modules/pause-overlay|pause overlay]] sub-plugins. The full picture, with a component diagram, is in [[architecture/overview]]; the recurring designs are catalogued in [[patterns]].

## Features

The core is a [[features/countdown-timer|countdown timer]] you drive by [[features/hourglass-interaction|clicking or dragging the hourglass]] or via the [[features/timer-duration-controls|control panel]]. Around it sits appearance customization: [[features/color-selection|color selection]] (static, random, or cycling rainbow), [[features/shape-selection|shape selection]] across four presets, and continuous [[features/shape-morphing|shape morphing]]. The whole app also ships as a [[features/web-build|WASM web build]]. See [[features/overview]] for the inventory.

## Quick Reference

- [[code-index/entry-points]] — Where execution begins
- [[code-index/important-files]] — Key files at a glance
- [[onboarding/first-day-guide]] — Start here if you're new
- [[architecture/overview]] — System design and component diagram
- [[references/test-coverage]] — How well tests cover each feature
- [[patterns]] — Recurring design patterns

## Modules

[[modules/app]] is the composition root; [[modules/resources]] holds the shared state every module reads. [[modules/timer]] advances the countdown and [[modules/hourglass]] (the largest module) handles all rendering, shape presets, morphing, and click/drag input. The UI is [[modules/ui-layout]] (flexbox scaffold + markers) plus four panels: [[modules/color-panel]], [[modules/shape-panel]], [[modules/timer-panel]], and [[modules/pause-overlay]].

## Flows

- [[flows/startup]] — `main()` → built, interactive screen
- [[flows/countdown-tick]] — per-frame decrement → sand level → display
- [[flows/click-vs-drag]] — resolving a single interaction into pause/play vs. flip/reset
- [[flows/appearance-recreation]] — how color/shape changes rebuild the hourglass
