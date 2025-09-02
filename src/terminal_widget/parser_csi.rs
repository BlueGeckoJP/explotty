use eframe::egui::Color32;

use crate::{
    terminal_buffer::TerminalBuffer,
    terminal_cell::TerminalCell,
    terminal_widget::{TerminalWidget, color},
};

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
                let original_sequence = sequence;
                let mut sequence = sequence.trim_end_matches('m').to_string();

                // Process 38 and 48 for foreground and background colors first
                match &sequence {
                    // Handle 24-bit color foreground color
                    s if s.contains("38;2;") => {
                        let delimiter = "38;2;";
                        if let Some(start_pos) = s.find(delimiter) {
                            // Extract the part after delimiter
                            let after_delimiter = &s[start_pos + delimiter.len()..];
                            let parts: Vec<&str> = after_delimiter.split(';').take(3).collect();

                            // Remove delimiter and the parts from the sequence
                            let mut to_remove = delimiter.to_string();
                            for (i, part) in parts.iter().enumerate() {
                                to_remove.push_str(part);
                                if i < parts.len() - 1
                                    || after_delimiter.split(';').nth(3).is_some()
                                {
                                    to_remove.push(';');
                                }
                            }

                            let new_sequence = sequence.replace(&to_remove, "");

                            // Convert the RGB values to Color32
                            let rgb = parts
                                .iter()
                                .map(|x| x.parse::<u8>().unwrap_or(0))
                                .collect::<Vec<u8>>();

                            self.buffer.current_fg_color = Color32::from_rgb(
                                rgb.first().cloned().unwrap_or(0),
                                rgb.get(1).cloned().unwrap_or(0),
                                rgb.get(2).cloned().unwrap_or(0),
                            );

                            sequence = new_sequence;
                        }
                    }
                    s if s.contains("38;5;") => {
                        let delimiter = "38;5;";
                        if let Some(start_pos) = s.find(delimiter) {
                            // Extract the part after delimiter
                            let after_delimiter = &s[start_pos + delimiter.len()..];
                            if let Some(color_index_str) = after_delimiter.split(';').next() {
                                if let Ok(color_index) = color_index_str.parse::<u8>() {
                                    self.buffer.current_fg_color =
                                        color::process_256_color_palette(color_index);
                                }

                                // Remove the color index from the sequence
                                sequence =
                                    sequence.replace(&format!("{delimiter}{color_index_str}"), "");
                            }
                        }
                    }
                    s if s.contains("48;2;") => {
                        let delimiter = "48;2;";
                        if let Some(start_pos) = s.find(delimiter) {
                            // Extract the part after delimiter
                            let after_delimiter = &s[start_pos + delimiter.len()..];
                            let parts: Vec<&str> = after_delimiter.split(';').take(3).collect();

                            // Remove delimiter and the parts from the sequence
                            let mut to_remove = delimiter.to_string();
                            for (i, part) in parts.iter().enumerate() {
                                to_remove.push_str(part);
                                if i < parts.len() - 1
                                    || after_delimiter.split(';').nth(3).is_some()
                                {
                                    to_remove.push(';');
                                }
                            }

                            let new_sequence = sequence.replace(&to_remove, "");

                            // Convert the RGB values to Color32
                            let rgb = parts
                                .iter()
                                .map(|x| x.parse::<u8>().unwrap_or(0))
                                .collect::<Vec<u8>>();

                            self.buffer.current_bg_color = Color32::from_rgb(
                                rgb.first().cloned().unwrap_or(0),
                                rgb.get(1).cloned().unwrap_or(0),
                                rgb.get(2).cloned().unwrap_or(0),
                            );

                            sequence = new_sequence;
                        }
                    }
                    s if s.contains("48;5;") => {
                        let delimiter = "48;5;";
                        if let Some(start_pos) = s.find(delimiter) {
                            // Extract the part after delimiter
                            let after_delimiter = &s[start_pos + delimiter.len()..];
                            if let Some(color_index_str) = after_delimiter.split(';').next() {
                                if let Ok(color_index) = color_index_str.parse::<u8>() {
                                    self.buffer.current_bg_color =
                                        color::process_256_color_palette(color_index);
                                }

                                // Remove the color index from the sequence
                                sequence =
                                    sequence.replace(&format!("{delimiter}{color_index_str}"), "");
                            }
                        }
                    }

                    _ => {}
                }

                // \e[m process
                if original_sequence == "m" {
                    self.buffer.current_fg_color = Color32::WHITE;
                    self.buffer.current_bg_color = Color32::TRANSPARENT;
                    self.buffer.current_bold = false;
                    self.buffer.current_underline = false;
                    self.buffer.current_italic = false;
                    return;
                }

                let params: Vec<&str> = sequence.split(';').collect();
                for param in params {
                    match param {
                        "0" | "00" => {
                            // Reset all attributes
                            self.buffer.current_fg_color = Color32::WHITE;
                            self.buffer.current_bg_color = Color32::TRANSPARENT;
                            self.buffer.current_bold = false;
                            self.buffer.current_underline = false;
                            self.buffer.current_italic = false;
                        }
                        "1" | "01" => self.buffer.current_bold = true,
                        "2" | "02" => {
                            let current_color = self.buffer.current_fg_color;
                            self.buffer.current_fg_color = Color32::from_rgb(
                                (current_color.r() as u16 * 4 / 5).min(255) as u8,
                                (current_color.g() as u16 * 4 / 5).min(255) as u8,
                                (current_color.b() as u16 * 4 / 5).min(255) as u8,
                            );
                        }
                        "3" | "03" => self.buffer.current_italic = true,
                        "4" | "04" => self.buffer.current_underline = true,
                        // "5" | "05" => Blink
                        // "7" | "07" => Reverse
                        // "8" | "08" => Hidden
                        // "9" | "09" => Strikethrough
                        "30" => {
                            self.buffer.current_fg_color = Color32::BLACK;
                        }
                        "31" => {
                            self.buffer.current_fg_color = Color32::RED;
                        }
                        "32" => {
                            self.buffer.current_fg_color = Color32::GREEN;
                        }
                        "33" => {
                            self.buffer.current_fg_color = Color32::YELLOW;
                        }
                        "34" => {
                            self.buffer.current_fg_color = Color32::BLUE;
                        }
                        "35" => {
                            self.buffer.current_fg_color = Color32::MAGENTA;
                        }
                        "36" => {
                            self.buffer.current_fg_color = Color32::CYAN;
                        }
                        "37" => {
                            self.buffer.current_fg_color = Color32::WHITE;
                        }
                        "39" => {
                            self.buffer.current_fg_color = Color32::WHITE;
                        }
                        "40" => {
                            self.buffer.current_bg_color = Color32::BLACK;
                        }
                        "41" => {
                            self.buffer.current_bg_color = Color32::RED;
                        }
                        "42" => {
                            self.buffer.current_bg_color = Color32::GREEN;
                        }
                        "43" => {
                            self.buffer.current_bg_color = Color32::YELLOW;
                        }
                        "44" => {
                            self.buffer.current_bg_color = Color32::BLUE;
                        }
                        "45" => {
                            self.buffer.current_bg_color = Color32::MAGENTA;
                        }
                        "46" => {
                            self.buffer.current_bg_color = Color32::CYAN;
                        }
                        "47" => {
                            self.buffer.current_bg_color = Color32::WHITE;
                        }
                        "49" => {
                            self.buffer.current_bg_color = Color32::TRANSPARENT;
                        }

                        "90" => {
                            self.buffer.current_fg_color = color::to_bright(Color32::BLACK);
                        }
                        "91" => {
                            self.buffer.current_fg_color = color::to_bright(Color32::RED);
                        }
                        "92" => {
                            self.buffer.current_fg_color = color::to_bright(Color32::GREEN);
                        }
                        "93" => {
                            self.buffer.current_fg_color = color::to_bright(Color32::YELLOW);
                        }
                        "94" => {
                            self.buffer.current_fg_color = color::to_bright(Color32::BLUE);
                        }
                        "95" => {
                            self.buffer.current_fg_color = color::to_bright(Color32::MAGENTA);
                        }
                        "96" => {
                            self.buffer.current_fg_color = color::to_bright(Color32::CYAN);
                        }
                        "97" => {
                            self.buffer.current_fg_color = color::to_bright(Color32::WHITE);
                        }
                        "100" => {
                            self.buffer.current_bg_color = color::to_bright(Color32::BLACK);
                        }
                        "101" => {
                            self.buffer.current_bg_color = color::to_bright(Color32::RED);
                        }
                        "102" => {
                            self.buffer.current_bg_color = color::to_bright(Color32::GREEN);
                        }
                        "103" => {
                            self.buffer.current_bg_color = color::to_bright(Color32::YELLOW);
                        }
                        "104" => {
                            self.buffer.current_bg_color = color::to_bright(Color32::BLUE);
                        }
                        "105" => {
                            self.buffer.current_bg_color = color::to_bright(Color32::MAGENTA);
                        }
                        "106" => {
                            self.buffer.current_bg_color = color::to_bright(Color32::CYAN);
                        }
                        "107" => {
                            self.buffer.current_bg_color = color::to_bright(Color32::WHITE);
                        }

                        "" => {}
                        _ => {
                            warn!("Unsupported SGR parameter: {param}");
                        }
                    }
                }
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
