use eframe::egui;

use crate::terminal_widget::TerminalWidget;

impl TerminalWidget {
    pub fn process_osc_sequence(&mut self, ctx: &egui::Context, sequence: &str) {
        debug!("Processing OSC sequence: {sequence}");

        // Process the OSC sequence
        match sequence {
            s if s.starts_with("0;") => {
                // Set title (OSC 0)
                let title = s.trim_start_matches("0;").trim_end_matches('\x07');
                if !title.is_empty() {
                    // Send the title to the terminal
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.to_string()));
                }
            }
            _ => {
                warn!("Unhandled OSC sequence: {sequence}");
            }
        }
    }
}
