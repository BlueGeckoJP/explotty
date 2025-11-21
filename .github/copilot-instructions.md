# Explotty Terminal Emulator

Explotty is a Rust-based GUI terminal emulator built with eframe (egui), GTK, and portable-pty. It features both a terminal widget and a file explorer widget in a single application window.

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## Working Effectively

### Bootstrap and Build
- Install system dependencies: `sudo apt-get update && sudo apt-get install -y libgtk-3-dev libglib2.0-dev libgdk-pixbuf2.0-dev libpango1.0-dev libatk1.0-dev libcairo-gobject2 libepoxy-dev`
- Build debug version: `cargo build` -- takes 2 minutes. NEVER CANCEL. Set timeout to 5+ minutes.
- Build release version: `cargo build --release` -- takes 4 minutes. NEVER CANCEL. Set timeout to 10+ minutes.
- Run tests: `cargo test` -- takes under 2 seconds (no tests currently exist).

### Code Quality and Validation
- Check formatting: `cargo fmt --check`
- Fix formatting: `cargo fmt`
- Run linting: `cargo clippy` -- takes about 1 minute. NEVER CANCEL. Set timeout to 3+ minutes.
- Always run `cargo fmt` and `cargo clippy` before completing work or CI may fail.

### Running the Application
- The application is a GUI terminal emulator that requires a display server
- Cannot run in headless environments - GTK initialization will fail
- Run with: `cargo run` (debug) or `cargo run --release` (release)
- The application opens an 800x600 window with terminal and file explorer widgets

## Configuration

The application supports TOML configuration files in these locations (checked in order):
- `~/.config/explotty.toml`
- `~/.explotty.toml`

Configuration options:
- `ui_font_family`: Optional UI font family name
- `terminal_font_family`: Optional terminal font family name
- `terminal_fallback_font_families`: Optional array of fallback font families for terminal

## Project Structure

### Root Module Files
- `src/main.rs` - Application entry point, GTK initialization, and CONFIG static setup
- `src/app.rs` - Main application struct (App) with terminal and explorer widgets, PTY management
- `src/config.rs` - Configuration loading and management
- `src/terminal_widget.rs` - Terminal widget implementation and terminal state
- `src/terminal_buffer.rs` - Terminal buffer management
- `src/terminal_cell.rs` - Individual terminal cell representation with styling
- `src/explorer_widget.rs` - File explorer widget
- `src/utils.rs` - Utility functions including font loading and file operations
- `src/logging.rs` - Logging output and input for debugging
- `src/parser.rs` - Terminal sequence parser module exports

### Terminal Widget Submodule (`src/terminal_widget/`)
- `color.rs` - Color management and ANSI color support
- `input.rs` - Input handling and key mapping
- `render.rs` - Terminal rendering and layout

### Parser Submodule (`src/parser/`)
- `dispatcher.rs` - Sequence dispatch logic
- `handler_context.rs` - Context for handling terminal sequences
- `handlers.rs` - Handler registry and routing
- `sequence_handler.rs` - Base trait for sequence handlers
- `sequence_token.rs` - Token representation for sequences
- `sequence_tokenizer.rs` - Tokenization of terminal sequences
- `handlers/` - Specific handler implementations
  - `csi_sequence_handler.rs` - CSI (Control Sequence Introducer) handling
  - `dcs_sequence_handler.rs` - DCS (Device Control String) handling
  - `osc_sequence_handler.rs` - OSC (Operating System Command) handling
  - `sgr_sequence_handler.rs` - SGR (Select Graphic Rendition) handling
  - `vt100_sequence_handler.rs` - VT100 compatibility sequences

### Key Dependencies
- `eframe` (0.32) - GUI framework
- `egui_extras` (0.32) - Extended egui components
- `portable-pty` (0.9) - Cross-platform PTY implementation
- `gtk` (0.18) - GTK system integration
- `font-kit` (0.14) - Font discovery and loading
- `anyhow` - Error handling
- `serde` + `toml` - Configuration parsing
- `log` + `env_logger` - Logging infrastructure
- `unicode-width` - Terminal character width calculation
- `open` (5.3) - Open system files/URLs
- `resvg` (0.45) - SVG rendering
- `gio` (0.21) - GLib I/O library
- `chrono` (0.4) - Date/time utilities

## Build Requirements

### System Dependencies (Linux)
CRITICAL: Must install GTK development libraries before building:
```bash
sudo apt-get update && sudo apt-get install -y \
  libgtk-3-dev \
  libglib2.0-dev \
  libgdk-pixbuf2.0-dev \
  libpango1.0-dev \
  libatk1.0-dev \
  libcairo-gobject2 \
  libepoxy-dev
```

### Rust Toolchain
- Edition: 2024 (requires recent Rust)
- rustfmt and clippy are available and should be used
- Features available: `debug-outline`, `debug-logging` (for debugging)

## Timing Expectations

**NEVER CANCEL BUILD OR LINT COMMANDS** - Always allow full completion:

- Debug build: ~2 minutes (set timeout to 5+ minutes)
- Release build: ~4 minutes (set timeout to 10+ minutes)
- Clippy linting: ~1 minute (set timeout to 3+ minutes)
- Tests: <2 seconds (no tests currently exist)
- Formatting check: <5 seconds

## Validation Requirements

When making changes:
1. **ALWAYS** run `cargo fmt` to fix formatting
2. **ALWAYS** run `cargo clippy` to check for linting issues
3. Build the project with `cargo build` to ensure compilation
4. The application cannot be functionally tested in headless environments
5. Configuration changes can be tested by creating test config files

## Common Tasks

### Adding Terminal Sequence Handlers
1. Create new handler file in `src/parser/handlers/` implementing `SequenceHandler` trait
2. Register handler in `src/parser/handlers.rs`
3. Add routing logic in `src/parser/dispatcher.rs`
4. Test with appropriate terminal sequences

### Adding New Features
- Terminal rendering: Modify `src/terminal_widget/render.rs`
- Input handling: Extend `src/terminal_widget/input.rs`
- Terminal state/buffer: Update `src/terminal_buffer.rs` and `src/terminal_cell.rs`
- UI features: Modify `src/app.rs` for main application logic
- Configuration: Update `src/config.rs`
- File operations: Extend `src/explorer_widget.rs` or `src/utils.rs`

### Debugging Issues
- Enable debug logging: Set `RUST_LOG=debug` environment variable
- Use `debug-logging` feature: `cargo run --features debug-logging`
- Check GTK initialization if application won't start
- Font loading issues are handled in `src/utils.rs::load_system_font`
- PTY issues are in `src/app.rs::start_pty`
- Terminal sequence parsing: Check `src/parser/sequence_tokenizer.rs` for tokenization
- Handler dispatch: Review `src/parser/dispatcher.rs` for routing logic

### Code Quality Checks
Run these before submitting changes:
```bash
cargo fmt                    # Fix formatting
cargo clippy                 # Check linting (1 min)
cargo build                  # Verify compilation (2 min)
cargo build --release        # Verify release build (4 min)
```

## Architecture Notes

### Terminal Processing Pipeline
1. **Input**: User keyboard input → `src/terminal_widget/input.rs`
2. **PTY Output**: Shell output received in `src/app.rs` → buffered
3. **Parsing**: Terminal sequences parsed by `src/parser/sequence_tokenizer.rs`
4. **Dispatch**: Tokens routed through `src/parser/dispatcher.rs` to appropriate handlers
5. **Handling**: Handlers update terminal state in `src/terminal_buffer.rs`
6. **Rendering**: `src/terminal_widget/render.rs` draws current buffer state

### Configuration System
- Static CONFIG loaded at startup in `src/main.rs`
- Accessible globally via `crate::CONFIG`
- Defaults used if no config file found
- Supports font customization per-component

## Known Issues

- Application requires GUI environment and cannot run headless
- No unit tests exist currently
- Configuration file is optional - app uses defaults if not found

## File Outputs

### Repository Root Structure
```
.
├── .git/
├── .gitignore              # Contains `/target`
├── Cargo.toml             # Project configuration
├── Cargo.lock             # Dependency lock file
└── src/                   # Source code directory
```

### Cargo.toml Key Information
- Package name: `explotty`
- Edition: `2024`
- Key dependencies: eframe, egui_extras, gtk, portable-pty, font-kit, log, env_logger
- Features: `debug-outline`, `debug-logging` available for debugging

### Key Application Features
- GUI terminal emulator with PTY support (bash shell)
- File explorer widget integrated in same window
- Modular terminal sequence parser with pluggable handlers
- Font customization via configuration
- GTK integration for system functionality
- Comprehensive ANSI/VT100 sequence support (CSI, DCS, OSC, SGR)
- Multi-threaded PTY I/O with buffering
- Logging infrastructure for debugging