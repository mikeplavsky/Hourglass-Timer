# Wiki Changelog

## 2026-06-06

- Documented the **flip-on-change** feature (commit `beaf967`), which the prior sync had only covered in the test/patterns pages. A color or shape change now also flips the hourglass; because the change rebuilds the entity, the flip is deferred via a new `PendingFlip` resource and applied to the rebuilt entity by `apply_pending_flip`, with `handle_timer_start` suppressing its first-start flip while one is pending.
  - New `### PendingFlip` section in [[modules/resources]]; new `## Flip-on-change orchestration` section in [[modules/hourglass]] (+ `apply_pending_flip` in the systems table, updated `handle_timer_start`).
  - New `## Flipping the rebuilt hourglass` section in [[flows/appearance-recreation]] with its own sequence diagram; cross-linked from [[flows/click-vs-drag]].
  - Updated [[modules/color-panel]], [[modules/shape-panel]], [[features/color-selection]], [[features/shape-selection]], [[features/hourglass-interaction]], [[features/overview]] side-effect sections (color/shape change → restart + flip).
  - Added the `PendingFlip` resource node + flip edges to the [[architecture/overview]] component diagram and a new design-decision bullet.
- Refreshed [[modules/pause-overlay]]'s visibility-condition code block to show the extracted `pause_overlay_should_show` helper (the inline `if` was replaced in `0b73d65`).
- Advanced `.wiki-state.json` `last_sync_commit` from `1a17fc4` to `a165b49` (it had not been bumped during the 2026-05-31 partial sync).

## 2026-05-31

- Test suite grew from 43 → **77** tests (new headless-`App` system tests + 4 extracted pure helpers: `within_click_radius`, `exceeds_drag_threshold`, `pause_overlay_should_show`, `shape_button_scale`). Line coverage 28% → 44%.
- Rewrote `references/test-coverage.md` (thesis, per-file counts, feature matrix, gaps, pinned-behavior list) and added the coverage command.
- Added a "Headless `App` system tests" pattern to `patterns.md` and refreshed the pure-helpers list.
- Corrected the `## Tests` sections in `modules/hourglass`, `modules/timer-panel`, `modules/shape-panel`, `modules/pause-overlay` (no longer "no unit tests").

## 2026-05-30

- Initial wiki generation (full bootstrap) at commit `1a17fc4`.
- Created config + index:
  - `wiki.config.md`, `HOME.md`, `log.md`, `patterns.md`
  - `code-index/entry-points.md`, `code-index/important-files.md`
- Created architecture + onboarding:
  - `architecture/overview.md` (component diagram)
  - `onboarding/first-day-guide.md`
- Created 9 module pages: `app`, `resources`, `timer`, `hourglass`, `ui-layout`, `color-panel`, `shape-panel`, `timer-panel`, `pause-overlay`.
- Created 7 feature pages: `countdown-timer`, `timer-duration-controls`, `hourglass-interaction`, `color-selection`, `shape-selection`, `shape-morphing`, `web-build`; plus `features/overview.md`.
- Created 4 flow pages (each with a Mermaid diagram): `startup`, `countdown-tick`, `click-vs-drag`, `appearance-recreation`.
- Created `references/test-coverage.md` — feature × test matrix answering "how well do tests cover each feature" (43 unit tests; pure logic well covered, ECS systems manual-only).
- Updated `.gitignore` to exclude `.obsidian/`.
