# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

### Native Development
- `cargo run` - Run the application natively with dev features
- `cargo build` - Build for native target
- `cargo clippy` - Run linter (uses custom clippy.toml config)
- `cargo fmt` - Format code

### Web/WASM Build
- `./build_wasm.sh` - Build WASM version (automatically installs dependencies)
- `cd wasm && python -m http.server 8080` - Serve WASM build locally

### Features and Profiles
- Default dev build uses `dev_native` feature with hot reloading
- WASM builds use minimal Bevy features for web compatibility
- Release builds optimize for performance with LTO and codegen-units=1

## Architecture Overview

This is a Bevy-based interactive hourglass timer application with both native and web (WASM) support.

### Core Plugin Structure
- **AppPlugin**: Main entry point, initializes resources and spawns camera
- **HourglassPlugin**: Manages the visual hourglass using bevy_hourglass crate
- **TimerPlugin**: Handles timer logic and state management  
- **UIPlugin**: Coordinates all UI panels and layout

### Key Resources
- **HourglassConfig**: Stores appearance settings (colors, shapes, animation modes)
- **TimerState**: Manages timer duration, running state, and elapsed time

### UI Architecture
The UI uses a structured layout with marker components:
- **TopControlsMarker**: Contains color selection panel
- **ShapeRowMarker**: Shape selection controls
- **BottomTimerMarker**: Collapsible timer controls panel
- **TimerPanelVisible**: Resource controlling timer panel visibility

### Module Organization
- `hourglass.rs`: Visual hourglass rendering and interaction
- `timer.rs`: Timer logic and state management
- `resources.rs`: Shared data structures and configurations
- `ui/`: UI components split into focused modules (color_panel, timer_panel, etc.)

### Special Considerations
- Project was developed entirely through LLM prompting ("vibe coded")
- Uses custom Bevy features for both native (with Wayland support) and web builds
- Drag interactions have threshold-based detection to distinguish clicks from drags
- Color modes include static colors, random generation, and continuous cycling