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
- Check formatting: `cargo fmt --check` -- currently shows formatting issues that need fixing
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

### Source Files
- `src/main.rs` - Application entry point and GTK initialization
- `src/app.rs` - Main application struct with terminal and explorer widgets
- `src/config.rs` - Configuration loading and management
- `src/terminal_widget/` - Terminal emulator implementation
  - `input.rs` - Input handling and key mapping
  - `parser.rs` - Terminal sequence parsing
  - `parser_csi.rs` - CSI sequence handling
  - `parser_osc.rs` - OSC sequence handling  
  - `parser_vt100.rs` - VT100 compatibility
  - `render.rs` - Terminal rendering
  - `color.rs` - Color management
- `src/terminal_buffer.rs` - Terminal buffer management
- `src/terminal_cell.rs` - Individual terminal cell representation
- `src/explorer_widget.rs` - File explorer widget
- `src/utils.rs` - Utility functions including font loading and file operations

### Key Dependencies
- `eframe` - GUI framework
- `egui` - Immediate mode GUI library
- `portable-pty` - Cross-platform PTY implementation
- `gtk` - GTK system integration
- `font-kit` - Font discovery and loading
- `anyhow` - Error handling
- `serde` + `toml` - Configuration parsing

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
- Minimum Rust version: Uses 2024 edition (requires recent Rust)
- rustfmt and clippy are available and should be used

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

### Adding New Features
- Terminal-related features: Modify files in `src/terminal_widget/`
- UI features: Modify `src/app.rs` for main application logic
- Configuration: Update `src/config.rs` and the Config struct
- File operations: Extend `src/explorer_widget.rs` or `src/utils.rs`

### Debugging Issues
- Enable debug logging: Set `RUST_LOG=debug` environment variable
- Check GTK initialization if application won't start
- Font loading issues are handled in `src/utils.rs::load_system_font`
- PTY issues are in `src/app.rs::start_pty`

### Code Quality Checks
Run these before submitting changes:
```bash
cargo fmt                    # Fix formatting
cargo clippy                 # Check linting (1 min)  
cargo build                  # Verify compilation (2 min)
cargo build --release        # Verify release build (4 min)
```

## Known Issues

- Application requires GUI environment and cannot run headless
- Current code has formatting issues that must be fixed with `cargo fmt`
- No unit tests exist currently
- Configuration file is optional - app uses defaults if not found

## File Outputs

### Repository Root Structure
```
.
├── .git/
├── .gitignore           # Contains `/target`
├── Cargo.toml          # Project configuration
├── Cargo.lock          # Dependency lock file
└── src/                # Source code directory
```

### Cargo.toml Key Information
- Package name: `explotty`
- Edition: `2024`
- Key dependencies: eframe, egui_extras, gtk, portable-pty, font-kit
- Features: `debug-outline` available for debugging

### Key Application Features
- GUI terminal emulator with PTY support
- File explorer widget integrated in same window
- Font customization via configuration
- GTK integration for system functionality
- VT100/ANSI terminal sequence support