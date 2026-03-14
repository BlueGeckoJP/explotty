use crate::parser::{handler_context::HandlerContext, sequence_handler::SequenceHandler};

pub struct EscSequenceHandler;

impl SequenceHandler for EscSequenceHandler {
    fn handle(&self, ctx: &mut HandlerContext, sequence: &str) {
        match sequence {
            "7" => {
                // Save Cursor (DECSC)
                ctx.buffer.saved_cursor_x = ctx.buffer.cursor_x;
                ctx.buffer.saved_cursor_y = ctx.buffer.cursor_y;
            }
            "8" => {
                // Restore Cursor (DECRC)
                let x = ctx.buffer.saved_cursor_x;
                let y = ctx.buffer.saved_cursor_y;
                ctx.buffer.move_cursor(x, y);
            }
            "D" => {
                // Index (IND)
                ctx.buffer.cursor_y += 1;
                if ctx.buffer.cursor_y > ctx.buffer.scroll_region_bottom {
                    ctx.buffer.scroll_up();
                    ctx.buffer.cursor_y = ctx.buffer.scroll_region_bottom;
                }
            }
            "E" => {
                // Next Line (NEL)
                ctx.buffer.cursor_x = 0;
                ctx.buffer.cursor_y += 1;
                if ctx.buffer.cursor_y > ctx.buffer.scroll_region_bottom {
                    ctx.buffer.scroll_up();
                    ctx.buffer.cursor_y = ctx.buffer.scroll_region_bottom;
                }
            }
            "M" => {
                // Reverse Index (RI)
                if ctx.buffer.cursor_y > ctx.buffer.scroll_region_top {
                    ctx.buffer.cursor_y -= 1;
                } else {
                    ctx.buffer.scroll_down();
                }
            }
            "c" => {
                // Full Reset (RIS)
                ctx.buffer.clear_screen();
                ctx.scrollback_buffer.clear();
                // Reset colors and other styles
                ctx.buffer.current_fg_color = eframe::egui::Color32::WHITE;
                ctx.buffer.current_bg_color = eframe::egui::Color32::TRANSPARENT;
                ctx.buffer.current_bold = false;
                ctx.buffer.current_underline = false;
                ctx.buffer.current_italic = false;
                ctx.buffer.current_blink = false;
                ctx.buffer.current_strikethrough = false;
                ctx.buffer.current_hidden = false;
            }
            "=" => {
                // Application Keypad (DECKPAM)
                warn!("Application Keypad (DECKPAM) enabled");
            }
            ">" => {
                // Normal Keypad (DECKPNM)
                warn!("Normal Keypad (DECKPNM) enabled");
            }
            "H" => {
                // Horizontal Tab Set (HTS)
                warn!("Horizontal Tab Set (HTS) not fully implemented");
            }
            "Z" => {
                // Return Terminal ID (DECID)
                let response = "\x1b[?1;2c";
                let output_buffer = crate::app::OUTPUT_BUFFER.get();
                if let Some(output_buffer) = output_buffer {
                    let mut output = output_buffer.lock();
                    output.extend_from_slice(response.as_bytes());
                }
            }
            seq if seq.starts_with('(') || seq.starts_with(')') || seq.starts_with('*') || seq.starts_with('+') => {
                // Designate Character Set
                warn!("Designate Character Set sequence: ESC {}", seq);
            }
            seq if seq.starts_with('#') => {
                // DEC Screen Alignment Test (DECALN), etc
                warn!("Screen alignment test sequence: ESC {}", seq);
            }
            _ => {
                warn!("Unhandled ESC sequence: ESC {}", sequence);
            }
        }
    }
}
