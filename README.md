# Hourglass Timer

An interactive hourglass timer built with Rust and the [Bevy game engine](https://bevyengine.org/).

**Website:** [https://edouardpoitras.github.io/hourglass-timer/](https://edouardpoitras.github.io/hourglass-timer/)

> **Note**: This project was entirely "vibe coded" - developed through LLM prompting solely. There be dragons in the code.

## Getting Started

### Running Natively

1. **Clone the repository**:
   ```bash
   git clone https://github.com/edouardpoitras/hourglass-timer.git
   cd hourglass-timer
   ```

2. **Run the application**:
   ```bash
   cargo run
   ```

### Web Version (WASM)

1. **Build for web**:
   ```bash
   chmod +x build_wasm.sh
   ./build_wasm.sh
   ```

2. **Serve the files**:
   ```bash
   cd wasm
   python -m http.server 8080
   ```

3. **Open in browser**:
   Navigate to `http://localhost:8080`

### Chrome Side Panel Extension

1. Install the build prerequisites: `wasm-bindgen`, Binaryen (`wasm-opt`), and `zip`.
2. Run `./build_extension.sh`.
3. Open `chrome://extensions`, enable Developer mode, choose **Load unpacked**, and select `dist/chrome-extension`.
4. Click the Hourglass Timer toolbar icon to open the side panel.

The build also creates `dist/hourglass-timer-extension.zip`. The extension stores one shared timer locally in Chrome and uses alarms and notifications when the panel is closed. It does not request access to websites or tabs.

## How to Use

1. **Set Your Time**:
   - Click "Timer Controls" to reveal the control panel
   - Use the +/- buttons to adjust your desired duration
   - Or start with the default 3 minutes

2. **Customize Appearance**:
   - Click color swatches for static colors
   - Try the colorful grid button for random colors
   - Click the rainbow stripes for continuous color cycling
   - Select different hourglass shapes or enable morphing mode

3. **Start Timer**:
   - Click the hourglass to start
   - Or use the "Start" button in the timer controls

4. **Control Playback**:
   - Click to pause/resume
   - Drag the hourglass to flip and reset
   - Use control buttons for precise start/pause/reset

## License

MIT OR Apache-2.0
