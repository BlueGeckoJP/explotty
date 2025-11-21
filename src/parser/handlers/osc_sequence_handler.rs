use eframe::egui;

use crate::parser::{handler_context::HandlerContext, sequence_handler::SequenceHandler};

pub struct OscSequenceHandler;

impl SequenceHandler for OscSequenceHandler {
    fn handle(&self, ctx: &mut HandlerContext, sequence: &str) {
        match sequence {
            s if s.starts_with("0;") => {
                // Set title (OSC 0)
                let title = s.trim_start_matches("0;").trim_end_matches('\x07');
                if !title.is_empty() {
                    // Send the title to the terminal
                    ctx.ctx
                        .send_viewport_cmd(egui::ViewportCommand::Title(title.to_string()));
                }
            }
            _ => {
                warn!("Unhandled OSC sequence: {sequence}");
            }
        }
    }
}
