# Wiki Configuration

## Scope
Include all source files by default.

## Exclusions
- Tests (unless specifically requested) — note: this project's unit tests live inline
  in `#[cfg(test)]` modules and ARE documented via [[references/test-coverage]].
- Build artifacts and generated files (`target/`, `wasm/*.js`, `wasm/*.wasm`)
- Dependency directories

## Custom Pages
Add entries here to request specific documentation pages.

## Style
- Concise pages (under 300 lines)
- Obsidian-style links: `[[path/to/page]]`
- Mermaid diagrams for flows and dependencies
- Explain behavior and architecture, not just code structure
