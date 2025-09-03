use eframe::egui::Color32;

use crate::terminal_widget::{TerminalWidget, color};

impl TerminalWidget {
    /// Process Select Graphic Rendition (SGR) parameters.
    /// `sequence` is the body part of a CSI sequence ending with 'm', without the trailing 'm'.
    /// Supports:
    /// - Reset (0)
    /// - Bold (1), Faint (2), Italic (3), Underline (4), Blink (5), Reverse (7), Hidden (8), Strikethrough (9)
    /// - Basic 30-37/40-47 colors + default 39/49
    /// - Bright 90-97/100-107 colors
    /// - 256-color and TrueColor via 38;5;idx / 48;5;idx and 38;2;r;g;b / 48;2;r;g;b
    pub fn process_sgr_sequence(&mut self, original_sequence: &str) {
        // Special case: ESC[m or ESC[0m
        if original_sequence.is_empty() || original_sequence == "0" {
            self.reset_sgr();
            return;
        }

        // We need to pre-scan for extended color specifications (38/48 with 2 or 5)
        // We'll parse token by token with an iterator so we can consume variable length params.
        let mut tokens = original_sequence.split(';').peekable();

        while let Some(token) = tokens.next() {
            if token.is_empty() {
                // Skip empty tokens (can happen with sequences like ";;m")
                continue;
            }

            match token {
                // Reset
                "0" => self.reset_sgr(),
                // Bold
                "1" => self.buffer.current_bold = true,
                // Faint (simulate by darkening fg)
                "2" => {
                    let c = self.buffer.current_fg_color;
                    self.buffer.current_fg_color = Color32::from_rgb(
                        (c.r() as u16 * 4 / 5) as u8,
                        (c.g() as u16 * 4 / 5) as u8,
                        (c.b() as u16 * 4 / 5) as u8,
                    );
                }
                // Italic
                "3" => self.buffer.current_italic = true,
                // Underline
                "4" => self.buffer.current_underline = true,
                // Blink
                "5" => self.buffer.current_blink = true,
                // Rapid Blink (treated same as regular blink)
                "6" => self.buffer.current_blink = true,
                // Reverse video
                "7" => {
                    std::mem::swap(
                        &mut self.buffer.current_fg_color,
                        &mut self.buffer.current_bg_color,
                    );
                }
                // Conceal / Hidden (proper flag-based implementation)
                "8" => {
                    self.buffer.current_hidden = true;
                }
                // Strikethrough
                "9" => self.buffer.current_strikethrough = true,
                // Primary font / Alternative font selections (10-19) ignored
                "10" | "11" | "12" | "13" | "14" | "15" | "16" | "17" | "18" | "19" => {}
                // Fraktur (20) ignored
                "20" => {}
                // Disable Bold/Faint
                "22" => {
                    self.buffer.current_bold = false;
                    // Note: faint is handled as darkened fg color, so we need to reset to original
                    // For now, we'll just clear bold. Proper faint handling would need color state stack.
                }
                // Disable Italic
                "23" => self.buffer.current_italic = false,
                // Disable Underline
                "24" => self.buffer.current_underline = false,
                // Disable Blink
                "25" => self.buffer.current_blink = false,
                // Disable Reverse
                "27" => {
                    // Note: Current reverse implementation swaps colors, but we cannot easily restore
                    // the original colors without maintaining a color state stack.
                    // This is a known limitation mentioned in the issue.
                    // For now, we swap again to reverse the effect (may not be perfectly accurate)
                    std::mem::swap(
                        &mut self.buffer.current_fg_color,
                        &mut self.buffer.current_bg_color,
                    );
                }
                // Reveal (disable hidden)
                "28" => self.buffer.current_hidden = false,
                // Disable Strikethrough
                "29" => self.buffer.current_strikethrough = false,

                // Foreground basic colors 30-37
                "30" => self.buffer.current_fg_color = Color32::BLACK,
                "31" => self.buffer.current_fg_color = Color32::RED,
                "32" => self.buffer.current_fg_color = Color32::GREEN,
                "33" => self.buffer.current_fg_color = Color32::YELLOW,
                "34" => self.buffer.current_fg_color = Color32::BLUE,
                "35" => self.buffer.current_fg_color = Color32::MAGENTA,
                "36" => self.buffer.current_fg_color = Color32::CYAN,
                "37" => self.buffer.current_fg_color = Color32::WHITE,
                // Default foreground
                "39" => self.buffer.current_fg_color = Color32::WHITE,

                // Background basic colors 40-47
                "40" => self.buffer.current_bg_color = Color32::BLACK,
                "41" => self.buffer.current_bg_color = Color32::RED,
                "42" => self.buffer.current_bg_color = Color32::GREEN,
                "43" => self.buffer.current_bg_color = Color32::YELLOW,
                "44" => self.buffer.current_bg_color = Color32::BLUE,
                "45" => self.buffer.current_bg_color = Color32::MAGENTA,
                "46" => self.buffer.current_bg_color = Color32::CYAN,
                "47" => self.buffer.current_bg_color = Color32::WHITE,
                // Default background
                "49" => self.buffer.current_bg_color = Color32::TRANSPARENT,

                // Bright foreground 90-97
                "90" => self.buffer.current_fg_color = color::to_bright(Color32::BLACK),
                "91" => self.buffer.current_fg_color = color::to_bright(Color32::RED),
                "92" => self.buffer.current_fg_color = color::to_bright(Color32::GREEN),
                "93" => self.buffer.current_fg_color = color::to_bright(Color32::YELLOW),
                "94" => self.buffer.current_fg_color = color::to_bright(Color32::BLUE),
                "95" => self.buffer.current_fg_color = color::to_bright(Color32::MAGENTA),
                "96" => self.buffer.current_fg_color = color::to_bright(Color32::CYAN),
                "97" => self.buffer.current_fg_color = color::to_bright(Color32::WHITE),

                // Bright background 100-107
                "100" => self.buffer.current_bg_color = color::to_bright(Color32::BLACK),
                "101" => self.buffer.current_bg_color = color::to_bright(Color32::RED),
                "102" => self.buffer.current_bg_color = color::to_bright(Color32::GREEN),
                "103" => self.buffer.current_bg_color = color::to_bright(Color32::YELLOW),
                "104" => self.buffer.current_bg_color = color::to_bright(Color32::BLUE),
                "105" => self.buffer.current_bg_color = color::to_bright(Color32::MAGENTA),
                "106" => self.buffer.current_bg_color = color::to_bright(Color32::CYAN),
                "107" => self.buffer.current_bg_color = color::to_bright(Color32::WHITE),

                // Extended color foreground/background 38/48
                "38" | "48" => {
                    // Expect either ;5;idx or ;2;r;g;b
                    let is_fg = token == "38";
                    let Some(mode) = tokens.next() else {
                        break;
                    };
                    match mode {
                        "5" => {
                            if let Some(idx_str) = tokens.next()
                                && let Ok(idx) = idx_str.parse::<u8>()
                            {
                                let col = color::process_256_color_palette(idx);
                                if is_fg {
                                    self.buffer.current_fg_color = col;
                                } else {
                                    self.buffer.current_bg_color = col;
                                }
                            }
                        }
                        "2" => {
                            let r = tokens
                                .next()
                                .and_then(|s| s.parse::<u8>().ok())
                                .unwrap_or(0);
                            let g = tokens
                                .next()
                                .and_then(|s| s.parse::<u8>().ok())
                                .unwrap_or(0);
                            let b = tokens
                                .next()
                                .and_then(|s| s.parse::<u8>().ok())
                                .unwrap_or(0);
                            let col = Color32::from_rgb(r, g, b);
                            if is_fg {
                                self.buffer.current_fg_color = col;
                            } else {
                                self.buffer.current_bg_color = col;
                            }
                        }
                        other => {
                            warn!("Unsupported extended color mode: {other}");
                        }
                    }
                }

                // Ignore unknown but log
                other => {
                    warn!("Unsupported SGR parameter: {other}");
                }
            }
        }
    }

    fn reset_sgr(&mut self) {
        self.buffer.current_fg_color = Color32::WHITE;
        self.buffer.current_bg_color = Color32::TRANSPARENT;
        self.buffer.current_bold = false;
        self.buffer.current_underline = false;
        self.buffer.current_italic = false;
        self.buffer.current_blink = false;
        self.buffer.current_strikethrough = false;
        self.buffer.current_hidden = false;
    }
}
