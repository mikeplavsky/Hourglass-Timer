# Wiki Changelog

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
