#!/bin/bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
DIST_ROOT="$PROJECT_ROOT/dist"
EXTENSION_DIST="$DIST_ROOT/chrome-extension"
PACKAGE_PATH="$DIST_ROOT/hourglass-timer-extension.zip"
WASM_INPUT="$PROJECT_ROOT/target/wasm32-unknown-unknown/web-release/hourglass-timer.wasm"

for tool in wasm-bindgen wasm-opt zip; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        if [ "$tool" = "wasm-opt" ]; then
            echo "wasm-opt is required. Install Binaryen first (for example: brew install binaryen)." >&2
        else
            echo "$tool is required but was not found in PATH." >&2
        fi
        exit 1
    fi
done

echo "Building Hourglass Timer Chrome extension..."
cargo build \
    --manifest-path "$PROJECT_ROOT/Cargo.toml" \
    --target wasm32-unknown-unknown \
    --profile web-release \
    --no-default-features \
    --features chrome_extension

rm -rf "$EXTENSION_DIST"
mkdir -p "$EXTENSION_DIST/icons"

wasm-bindgen \
    --target web \
    --out-name hourglass-timer \
    --out-dir "$EXTENSION_DIST" \
    "$WASM_INPUT"

wasm-opt -Oz \
    "$EXTENSION_DIST/hourglass-timer_bg.wasm" \
    -o "$EXTENSION_DIST/hourglass-timer_bg.optimized.wasm"
mv "$EXTENSION_DIST/hourglass-timer_bg.optimized.wasm" \
    "$EXTENSION_DIST/hourglass-timer_bg.wasm"

cp "$PROJECT_ROOT/extension/manifest.json" "$EXTENSION_DIST/"
cp "$PROJECT_ROOT/extension/sidepanel.html" "$EXTENSION_DIST/"
cp "$PROJECT_ROOT/extension/sidepanel.css" "$EXTENSION_DIST/"
cp "$PROJECT_ROOT/extension/sidepanel.mjs" "$EXTENSION_DIST/"
cp "$PROJECT_ROOT/extension/panel-connection.mjs" "$EXTENSION_DIST/"
cp "$PROJECT_ROOT/extension/service-worker.mjs" "$EXTENSION_DIST/"
cp "$PROJECT_ROOT/extension/state.mjs" "$EXTENSION_DIST/"
cp "$PROJECT_ROOT/extension/icons/"*.png "$EXTENSION_DIST/icons/"

rm -f "$PACKAGE_PATH"
(
    cd "$EXTENSION_DIST"
    zip -q -r "$PACKAGE_PATH" .
)

RAW_WASM_BYTES="$(wc -c < "$EXTENSION_DIST/hourglass-timer_bg.wasm" | tr -d ' ')"
PACKAGE_BYTES="$(wc -c < "$PACKAGE_PATH" | tr -d ' ')"
echo "Extension directory: $EXTENSION_DIST"
echo "Installable ZIP:     $PACKAGE_PATH"
echo "Optimized WASM:      $RAW_WASM_BYTES bytes"
echo "Packaged ZIP:        $PACKAGE_BYTES bytes"
