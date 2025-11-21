use eframe::egui::Color32;

use crate::parser::{handler_context::HandlerContext, sequence_handler::SequenceHandler};
use crate::terminal_widget::color;

pub struct SgrSequenceHandler;

impl SgrSequenceHandler {
    fn reset_sgr(ctx: &mut HandlerContext) {
        ctx.buffer.current_fg_color = Color32::WHITE;
        ctx.buffer.current_bg_color = Color32::TRANSPARENT;
        ctx.buffer.current_bold = false;
        ctx.buffer.current_underline = false;
        ctx.buffer.current_italic = false;
        ctx.buffer.current_blink = false;
        ctx.buffer.current_strikethrough = false;
        ctx.buffer.current_hidden = false;
    }
}

impl SequenceHandler for SgrSequenceHandler {
    fn handle(&self, ctx: &mut HandlerContext, sequence: &str) {
        // Special case: ESC[m or ESC[0m
        if sequence.is_empty() || sequence == "0" {
            Self::reset_sgr(ctx);
            return;
        }

        // We need to pre-scan for extended color specifications (38/48 with 2 or 5)
        // We'll parse token by token with an iterator so we can consume variable length params.
        let mut tokens = sequence.split(';').peekable();

        while let Some(token) = tokens.next() {
            if token.is_empty() {
                // Skip empty tokens (can happen with sequences like ";;m")
                continue;
            }

            match token.trim_end_matches('m') {
                // Reset
                "0" | "" => Self::reset_sgr(ctx),
                // Bold
                "1" | "01" => ctx.buffer.current_bold = true,
                // Faint (simulate by darkening fg)
                "2" => {
                    let c = ctx.buffer.current_fg_color;
                    ctx.buffer.current_fg_color = Color32::from_rgb(
                        (c.r() as u16 * 4 / 5) as u8,
                        (c.g() as u16 * 4 / 5) as u8,
                        (c.b() as u16 * 4 / 5) as u8,
                    );
                }
                // Italic
                "3" => ctx.buffer.current_italic = true,
                // Underline
                "4" => ctx.buffer.current_underline = true,
                // Blink
                "5" => ctx.buffer.current_blink = true,
                // Rapid Blink (treated same as regular blink)
                "6" => ctx.buffer.current_blink = true,
                // Reverse video
                "7" => {
                    std::mem::swap(
                        &mut ctx.buffer.current_fg_color,
                        &mut ctx.buffer.current_bg_color,
                    );
                }
                // Conceal / Hidden (proper flag-based implementation)
                "8" => {
                    ctx.buffer.current_hidden = true;
                }
                // Strikethrough
                "9" => ctx.buffer.current_strikethrough = true,
                // Primary font / Alternative font selections (10-19) ignored
                //"10" | "11" | "12" | "13" | "14" | "15" | "16" | "17" | "18" | "19" => {}
                // Fraktur (20) ignored
                "20" => {}
                // Disable Bold/Faint
                "22" => {
                    ctx.buffer.current_bold = false;
                    // Note: faint is handled as darkened fg color, so we need to reset to original
                    // For now, we'll just clear bold. Proper faint handling would need color state stack.
                }
                // Disable Italic
                "23" => ctx.buffer.current_italic = false,
                // Disable Underline
                "24" => ctx.buffer.current_underline = false,
                // Disable Blink
                "25" => ctx.buffer.current_blink = false,
                // Disable Reverse
                "27" => {
                    // Note: Current reverse implementation swaps colors, but we cannot easily restore
                    // the original colors without maintaining a color state stack.
                    // This is a known limitation mentioned in the issue.
                    // For now, we swap again to reverse the effect (may not be perfectly accurate)
                    std::mem::swap(
                        &mut ctx.buffer.current_fg_color,
                        &mut ctx.buffer.current_bg_color,
                    );
                }
                // Reveal (disable hidden)
                "28" => ctx.buffer.current_hidden = false,
                // Disable Strikethrough
                "29" => ctx.buffer.current_strikethrough = false,

                // Foreground basic colors 30-37
                "30" => ctx.buffer.current_fg_color = Color32::BLACK,
                "31" => ctx.buffer.current_fg_color = Color32::RED,
                "32" => ctx.buffer.current_fg_color = Color32::GREEN,
                "33" => ctx.buffer.current_fg_color = Color32::YELLOW,
                "34" => ctx.buffer.current_fg_color = Color32::BLUE,
                "35" => ctx.buffer.current_fg_color = Color32::MAGENTA,
                "36" => ctx.buffer.current_fg_color = Color32::CYAN,
                "37" => ctx.buffer.current_fg_color = Color32::WHITE,
                // Default foreground
                "39" => ctx.buffer.current_fg_color = Color32::WHITE,
                // Background basic colors 40-47
                "40" => ctx.buffer.current_bg_color = Color32::BLACK,
                "41" => ctx.buffer.current_bg_color = Color32::RED,
                "42" => ctx.buffer.current_bg_color = Color32::GREEN,
                "43" => ctx.buffer.current_bg_color = Color32::YELLOW,
                "44" => ctx.buffer.current_bg_color = Color32::BLUE,
                "45" => ctx.buffer.current_bg_color = Color32::MAGENTA,
                "46" => ctx.buffer.current_bg_color = Color32::CYAN,
                "47" => ctx.buffer.current_bg_color = Color32::WHITE,
                // Default background
                "49" => ctx.buffer.current_bg_color = Color32::TRANSPARENT,

                // Bright foreground 90-97
                "90" => ctx.buffer.current_fg_color = color::to_bright(Color32::BLACK),
                "91" => ctx.buffer.current_fg_color = color::to_bright(Color32::RED),
                "92" => ctx.buffer.current_fg_color = color::to_bright(Color32::GREEN),
                "93" => ctx.buffer.current_fg_color = color::to_bright(Color32::YELLOW),
                "94" => ctx.buffer.current_fg_color = color::to_bright(Color32::BLUE),
                "95" => ctx.buffer.current_fg_color = color::to_bright(Color32::MAGENTA),
                "96" => ctx.buffer.current_fg_color = color::to_bright(Color32::CYAN),
                "97" => ctx.buffer.current_fg_color = color::to_bright(Color32::WHITE),

                // Bright background 100-107
                "100" => ctx.buffer.current_bg_color = color::to_bright(Color32::BLACK),
                "101" => ctx.buffer.current_bg_color = color::to_bright(Color32::RED),
                "102" => ctx.buffer.current_bg_color = color::to_bright(Color32::GREEN),
                "103" => ctx.buffer.current_bg_color = color::to_bright(Color32::YELLOW),
                "104" => ctx.buffer.current_bg_color = color::to_bright(Color32::BLUE),
                "105" => ctx.buffer.current_bg_color = color::to_bright(Color32::MAGENTA),
                "106" => ctx.buffer.current_bg_color = color::to_bright(Color32::CYAN),
                "107" => ctx.buffer.current_bg_color = color::to_bright(Color32::WHITE),

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
                                    ctx.buffer.current_fg_color = col;
                                } else {
                                    ctx.buffer.current_bg_color = col;
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
                                ctx.buffer.current_fg_color = col;
                            } else {
                                ctx.buffer.current_bg_color = col;
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
}
