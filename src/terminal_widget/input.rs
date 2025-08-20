use eframe::egui;

use crate::terminal_widget::TerminalWidget;

impl TerminalWidget {
    pub fn handle_input(&mut self, ctx: &egui::Context) -> Vec<u8> {
        let mut output = Vec::new();
        let mut text_to_copy = None;

        // If we're scrolled up, any input should bring us back to bottom
        let should_scroll_to_bottom = self.scroll_offset > 0;

        ctx.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Copy => {
                        if let Some((start, end)) = self.selection_start.zip(self.selection_end) {
                            let mut selected_text = String::new();

                            let (start_row, end_row) = (start.1.min(end.1), start.1.max(end.1));
                            let (start_col, end_col) = (start.0.min(end.0), start.0.max(end.0));

                            let visible_lines = self.get_visible_lines();
                            for r in start_row..=end_row {
                                for c in start_col..=end_col {
                                    if r < visible_lines.len() && c < visible_lines[r].len() {
                                        selected_text.push(visible_lines[r][c].character);
                                    }
                                }
                                if r < end_row {
                                    selected_text.push('\n');
                                }
                            }

                            text_to_copy = Some(selected_text);
                        }
                    }
                    egui::Event::Paste(paste) => {
                        let mut paste_text = paste.clone();
                        if self.bracket_paste_mode {
                            paste_text = format!("\x1b[200~{paste_text}\x1b[201~");
                        }

                        output.extend_from_slice(paste_text.as_bytes());
                    }
                    egui::Event::Key {
                        key, pressed: true, ..
                    } => {
                        match key {
                            // Don't process navigation keys that should only scroll
                            egui::Key::PageUp | egui::Key::PageDown => {
                                // These are handled in handle_scroll
                                continue;
                            }
                            egui::Key::Home | egui::Key::End if i.modifiers.ctrl => {
                                // These are handled in handle_scroll
                                continue;
                            }

                            // Arrow keys
                            egui::Key::ArrowUp => {
                                output.extend_from_slice(if self.decckm_mode {
                                    b"\x1bOA"
                                } else {
                                    b"\x1b[A"
                                });
                            }
                            egui::Key::ArrowDown => {
                                output.extend_from_slice(if self.decckm_mode {
                                    b"\x1bOB"
                                } else {
                                    b"\x1b[B"
                                });
                            }
                            egui::Key::ArrowLeft => {
                                output.extend_from_slice(if self.decckm_mode {
                                    b"\x1bOD"
                                } else {
                                    b"\x1b[D"
                                });
                            }
                            egui::Key::ArrowRight => {
                                output.extend_from_slice(if self.decckm_mode {
                                    b"\x1bOC"
                                } else {
                                    b"\x1b[C"
                                });
                            }

                            // Numpad keys (only special in DECCKM application mode)
                            egui::Key::Num0 if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOp")
                            }
                            egui::Key::Num1 if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOq")
                            }
                            egui::Key::Num2 if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOr")
                            }
                            egui::Key::Num3 if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOs")
                            }
                            egui::Key::Num4 if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOt")
                            }
                            egui::Key::Num5 if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOu")
                            }
                            egui::Key::Num6 if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOv")
                            }
                            egui::Key::Num7 if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOw")
                            }
                            egui::Key::Num8 if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOx")
                            }
                            egui::Key::Num9 if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOy")
                            }
                            egui::Key::Plus if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOl")
                            }
                            egui::Key::Minus if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOm")
                            }
                            // Why no asterisks? Huh? Process in text input instead
                            /*egui::Key::Asterisk if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOj")
                            }*/
                            egui::Key::Slash if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOo")
                            }
                            egui::Key::Period if self.decckm_mode => {
                                output.extend_from_slice(b"\x1bOn")
                            }

                            // Enter keys
                            egui::Key::Enter => {
                                if self.decckm_mode {
                                    output.extend_from_slice(b"\x1bOM");
                                } else {
                                    output.extend_from_slice(b"\r");
                                }
                            }

                            // Other keys
                            egui::Key::Backspace => {
                                output.extend_from_slice(b"\x08");
                            }
                            egui::Key::Tab => {
                                output.extend_from_slice(b"\t");
                            }
                            egui::Key::Escape => {
                                output.extend_from_slice(b"\x1b");
                            }
                            egui::Key::U if i.modifiers.ctrl => {
                                output.extend_from_slice(b"\x15");
                            }
                            _ => {}
                        }
                    }
                    egui::Event::Text(text) => {
                        for ch in text.chars() {
                            if ch == '*' && self.decckm_mode {
                                output.extend_from_slice(b"\x1bOj");
                            } else {
                                let mut buf = [0; 4];
                                output.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        // If any input was generated and we're scrolled up, scroll to bottom
        if !output.is_empty() && should_scroll_to_bottom {
            self.scroll_offset = 0;
        }

        // Copy text to clipboard if available
        if let Some(text) = text_to_copy {
            ctx.copy_text(text);
            self.selection_start = None;
            self.selection_end = None;
        }

        output
    }

    pub fn handle_scroll(&mut self, ui: &mut egui::Ui) {
        ui.input(|i| {
            let scroll_delta = i.smooth_scroll_delta.y;
            if scroll_delta.abs() > 0.0 {
                let lines_to_scroll = (scroll_delta / self.line_height).round() as i32;

                if lines_to_scroll > 0 {
                    // Scrolling down
                    let max_scroll = self.scrollback_buffer.len();
                    self.scroll_offset =
                        (self.scroll_offset + lines_to_scroll as usize).min(max_scroll);
                } else {
                    // Scrolling up
                    self.scroll_offset =
                        self.scroll_offset.saturating_sub(-lines_to_scroll as usize);
                }
            }

            // Handle Page Up/Page Down keys
            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    match key {
                        egui::Key::PageUp => {
                            let scroll_amount = self.buffer.height.saturating_sub(1);
                            let max_scroll = self.scrollback_buffer.len();
                            self.scroll_offset =
                                (self.scroll_offset + scroll_amount).min(max_scroll);
                        }
                        egui::Key::PageDown => {
                            let scroll_amount = self.buffer.height.saturating_sub(1);
                            self.scroll_offset = self.scroll_offset.saturating_sub(scroll_amount);
                        }
                        egui::Key::Home if modifiers.ctrl => {
                            // Ctrl+Home: Go to top of history
                            self.scroll_offset = self.scrollback_buffer.len();
                        }
                        egui::Key::End if modifiers.ctrl => {
                            // Ctrl+End: Go to bottom (current)
                            self.scroll_offset = 0;
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}
