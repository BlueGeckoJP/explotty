use eframe::egui;

use crate::terminal_widget::TerminalWidget;

impl TerminalWidget {
    pub fn process_output(&mut self, ctx: &egui::Context, data: &[u8]) {
        self.pty_buffer.extend_from_slice(data);

        let mut cursor = 0;
        while cursor < self.pty_buffer.len() {
            let start_cursor = cursor;
            let remaining_bytes = &self.pty_buffer[cursor..].to_vec();

            match remaining_bytes[0] {
                b'\r' => {
                    self.buffer.carriage_return();
                    cursor += 1;
                }
                b'\n' => {
                    // Save the current top line to scrollback before scrolling
                    if self.buffer.cursor_y >= self.buffer.height - 1 {
                        let top_line = self.buffer.cells[0].clone();
                        self.add_line_to_scrollback(top_line);
                    }
                    self.buffer.new_line(self.new_line_mode);
                    cursor += 1;
                }
                b'\t' => {
                    for _ in 0..4 {
                        self.buffer.put_char(' ');
                    }
                    cursor += 1;
                }
                b'\x08' => {
                    self.buffer.backspace();
                    cursor += 1;
                }
                b'\x1b' => {
                    if remaining_bytes.len() < 2 {
                        break;
                    }

                    if remaining_bytes[1] == b'[' {
                        let mut end_of_seq = 0;
                        for (i, &byte) in remaining_bytes.iter().enumerate().skip(2) {
                            if byte.is_ascii_lowercase() || byte.is_ascii_uppercase() {
                                end_of_seq = i;
                                break;
                            }
                        }

                        if end_of_seq == 0 {
                            break;
                        }

                        let sequence_body = &remaining_bytes[2..=end_of_seq];
                        if let Ok(s) = std::str::from_utf8(sequence_body) {
                            self.process_csi_sequence(s);
                        }
                        cursor += end_of_seq + 1;
                    } else if remaining_bytes[1] == b']' {
                        let mut end_of_seq = 0;
                        let mut terminator_len = 0;

                        // Find the end of the OSC sequence
                        let mut i = 2;
                        while i < remaining_bytes.len() {
                            // BEL
                            if remaining_bytes[i] == b'\x07' {
                                end_of_seq = i;
                                terminator_len = 1;
                                break;
                            }
                            // ESC \
                            if remaining_bytes[i] == b'\x1b'
                                && i + 1 < remaining_bytes.len()
                                && remaining_bytes[i + 1] == b'\\'
                            {
                                end_of_seq = i;
                                terminator_len = 2;
                                break;
                            }
                            i += 1;
                        }

                        if end_of_seq == 0 {
                            break;
                        }

                        let sequence_body = &remaining_bytes[2..end_of_seq];
                        if let Ok(s) = std::str::from_utf8(sequence_body) {
                            self.process_osc_sequence(ctx, s);
                        }
                        cursor += end_of_seq + terminator_len;
                    } else {
                        cursor += 2;
                    }
                }
                ch if ch < 32 || ch == 127 => {
                    cursor += 1;
                }
                _ => match std::str::from_utf8(remaining_bytes) {
                    Ok(s) => {
                        if let Some(ch) = s.chars().next() {
                            self.buffer.put_char(ch);
                            cursor += ch.len_utf8();
                        }
                    }
                    Err(e) => {
                        let valid_len = e.valid_up_to();
                        if valid_len > 0 {
                            let valid_str = unsafe {
                                std::str::from_utf8_unchecked(&remaining_bytes[..valid_len])
                            };
                            for ch in valid_str.chars() {
                                self.buffer.put_char(ch);
                            }
                            cursor += valid_len;
                        } else {
                            break;
                        }
                    }
                },
            }

            if cursor == start_cursor {
                warn!("Terminal parser did not advance. Forcing advance to prevent freeze.");
                cursor += 1;
            }
        }

        if cursor > 0 {
            self.pty_buffer.drain(..cursor);
        }
    }
}
