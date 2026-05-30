# Wiki Changelog

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
