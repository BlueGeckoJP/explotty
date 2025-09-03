use crate::{terminal_cell::TerminalCell, terminal_widget::TerminalWidget};

impl TerminalWidget {
    pub fn process_csi_sequence(&mut self, sequence: &str) {
        debug!("Processing CSI sequence: {sequence}");

        if self.process_vt100(sequence) {
            return;
        }

        // Process the CSI sequence
        match sequence {
            // Cursor Control - Cursor Movement
            ch if ch.ends_with('A') => {
                // Cursor Up
                let num = sequence.trim_end_matches('A').parse::<usize>().unwrap_or(1);
                self.buffer.move_cursor(
                    self.buffer.cursor_x,
                    self.buffer.cursor_y.saturating_sub(num),
                );
            }
            ch if ch.ends_with('B') => {
                // Cursor Down
                let num = sequence.trim_end_matches('B').parse::<usize>().unwrap_or(1);
                self.buffer.move_cursor(
                    self.buffer.cursor_x,
                    self.buffer.cursor_y.saturating_add(num),
                );
            }
            ch if ch.ends_with('C') => {
                // Cursor Right
                let num = sequence.trim_end_matches('C').parse::<usize>().unwrap_or(1);
                self.buffer.move_cursor(
                    self.buffer.cursor_x.saturating_add(num),
                    self.buffer.cursor_y,
                );
            }
            ch if ch.ends_with('D') => {
                // Cursor Left
                let num = sequence.trim_end_matches('D').parse::<usize>().unwrap_or(1);
                self.buffer.move_cursor(
                    self.buffer.cursor_x.saturating_sub(num),
                    self.buffer.cursor_y,
                );
            }
            ch if ch.ends_with('E') => {
                // Cursor Next Line
                let num = sequence.trim_end_matches('E').parse::<usize>().unwrap_or(1);
                self.buffer
                    .move_cursor(0, self.buffer.cursor_y.saturating_add(num));
            }
            ch if ch.ends_with('F') => {
                // Cursor Previous Line
                let num = sequence.trim_end_matches('F').parse::<usize>().unwrap_or(1);
                self.buffer
                    .move_cursor(0, self.buffer.cursor_y.saturating_sub(num));
            }
            ch if ch.ends_with('G') => {
                // Cursor Horizontal Absolute
                let num = sequence.trim_end_matches('G').parse::<usize>().unwrap_or(1);
                self.buffer
                    .move_cursor(num.saturating_sub(1), self.buffer.cursor_y);
            }
            ch if ch.ends_with('H') || ch.ends_with('f') => {
                // Cursor Position (CSI H or CSI f)
                let parts: Vec<&str> = sequence.trim_end_matches(['H', 'f']).split(';').collect();
                let row = parts
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                let col = parts
                    .get(1)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                self.buffer
                    .move_cursor(col.saturating_sub(1), row.saturating_sub(1));
            }

            // Cursor Control - History of Cursor Position
            ch if ch.ends_with('s') => {
                // Save Cursor Position
                self.buffer.saved_cursor_x = self.buffer.cursor_x;
                self.buffer.saved_cursor_y = self.buffer.cursor_y;
            }
            ch if ch.ends_with('u') => {
                // Restore Cursor Position
                self.buffer
                    .move_cursor(self.buffer.saved_cursor_x, self.buffer.saved_cursor_y);
            }

            // Cursor Control - Report Cursor Position
            ch if ch.ends_with("6n") => {
                let x = self.buffer.cursor_x + 1; // Convert to 1-based index
                let y = self.buffer.cursor_y + 1; // Convert to 1-based index
                let response = format!("\x1b[{y};{x}R");

                {
                    // Send the response back to the terminal
                    let output_buffer = crate::app::OUTPUT_BUFFER.get();
                    if let Some(output_buffer) = output_buffer {
                        let mut output = output_buffer.lock();
                        output.extend_from_slice(response.as_bytes());
                    } else {
                        warn!("Output buffer not initialized");
                    }
                }
            }

            // Erase in Display/Line - Erase in Display
            ch if ch.ends_with('J') => {
                let num = sequence.trim_end_matches('J').parse::<usize>().unwrap_or(0);
                let (cx, cy) = (self.buffer.cursor_x, self.buffer.cursor_y);
                match num {
                    0 => {
                        // Erase from cursor to end of screen
                        // Erase from cursor to end of line
                        self.buffer.clear_range(
                            Some((cx, cy)),
                            Some((self.buffer.width.saturating_sub(1), cy)),
                        );
                        // Erase all lines below
                        if cy + 1 < self.buffer.height {
                            self.buffer.clear_range(Some((0, cy + 1)), None);
                        }
                    }
                    1 => {
                        // Erase from beginning of screen to cursor
                        // Erase all lines above
                        if cy > 0 {
                            self.buffer.clear_range(
                                None,
                                Some((self.buffer.width.saturating_sub(1), cy - 1)),
                            );
                        }
                        self.buffer.clear_range(Some((0, cy)), Some((cx, cy)));
                    }
                    2 => self.buffer.clear_screen(),
                    3 => {
                        // Clear entire screen including scrollback
                        self.buffer.clear_screen();
                        self.scrollback_buffer.clear();
                    }
                    _ => {
                        warn!("Unsupported erase in display parameter: {num}");
                    }
                }
            }

            // Erase in Display/Line - Erase in Line
            ch if ch.ends_with('K') => {
                let num = sequence.trim_end_matches('K').parse::<usize>().unwrap_or(0);
                let (cx, cy) = (self.buffer.cursor_x, self.buffer.cursor_y);
                match num {
                    0 => {
                        // Erase from cursor to end of line
                        self.buffer.clear_range(
                            Some((cx, cy)),
                            Some((self.buffer.width.saturating_sub(1), cy)),
                        );
                    }
                    1 => {
                        // Erase from start of line to cursor
                        self.buffer.clear_range(Some((0, cy)), Some((cx, cy)));
                    }
                    2 => {
                        // Erase entire line
                        self.buffer.clear_range(
                            Some((0, cy)),
                            Some((self.buffer.width.saturating_sub(1), cy)),
                        );
                    }
                    _ => {}
                }
            }

            // Select Graphic Rendition (SGR)
            ch if ch.ends_with('m') => {
                let body = sequence.trim_end_matches('m');
                self.process_sgr_sequence(body);
            }

            // Scroll Control - Scroll Up
            // ch if ch.ends_with('S') => {}

            // Scroll Control - Scroll Down
            // ch if ch.ends_with('T') => {}

            // Insert/delete lines/characters
            // ch if ch.ends_with('L') => {} // Insert lines
            // ch if ch.ends_with('M') => {} // Delete lines
            ch if ch.ends_with('P') => {
                // Delete characters
                let num = sequence.trim_end_matches('P').parse::<usize>().unwrap_or(1);
                if self.buffer.cursor_x < self.buffer.width {
                    for _ in 0..num {
                        if self.buffer.cursor_x < self.buffer.width {
                            self.buffer.cells[self.buffer.cursor_y].remove(self.buffer.cursor_x);
                            self.buffer.cells[self.buffer.cursor_y].push(TerminalCell::default());
                        }
                    }
                }
            }
            // ch if ch.ends_with('X') => {} // Erase characters
            // ch if ch.ends_with('@') => {} // Insert characters

            // Set Mode/Reset Mode
            // Not implemented yet

            // CSI n d (Vertical Line Position Absolute - VPA)
            ch if ch.ends_with('d') => {
                let row = sequence.trim_end_matches('d').parse::<usize>().unwrap_or(1);
                self.buffer
                    .move_cursor(self.buffer.cursor_x, row.saturating_sub(1));
            }

            // Other CSI sequences
            _ => {
                warn!("Unhandled CSI sequence: {sequence}");
            }
        }
    }
}
