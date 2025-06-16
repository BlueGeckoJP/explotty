use eframe::egui::{self, Color32, FontId, Pos2, Rect, TextFormat, text::LayoutJob};

use crate::terminal_buffer::TerminalBuffer;

pub struct TerminalWidget {
    pub buffer: TerminalBuffer,
    pub font_size: f32,
    pub char_width: f32,
    pub line_height: f32,
    pub show_cursor: bool,
}

impl TerminalWidget {
    pub fn new(width: usize, height: usize) -> Self {
        let font_size = 14.0;
        Self {
            buffer: TerminalBuffer::new(width, height),
            font_size,
            char_width: font_size * 0.6,
            line_height: font_size * 1.2,
            show_cursor: true,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let available_size = ui.available_size();

        // Calculate terminal size
        let cols = (available_size.x / self.char_width) as usize;
        let rows = (available_size.y / self.line_height) as usize;

        // Adjust buffer size
        if cols != self.buffer.width || rows != self.buffer.height {
            self.buffer.resize(cols, rows);
        }

        let response = ui.allocate_response(available_size, egui::Sense::click_and_drag());

        // Draw background
        ui.painter().rect_filled(response.rect, 0.0, Color32::BLACK);

        // Draw the terminal cells (characters)
        for (row_index, row) in self.buffer.cells.iter().enumerate() {
            for (col_index, cell) in row.iter().enumerate() {
                let pos = Pos2::new(
                    response.rect.left() + col_index as f32 * self.char_width,
                    response.rect.top() + row_index as f32 * self.line_height,
                );

                // Draw background color
                if cell.bg_color != Color32::TRANSPARENT {
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            pos,
                            egui::vec2(self.char_width, self.line_height),
                        ),
                        0.0,
                        cell.bg_color,
                    );
                }

                // Draw character
                if cell.character != ' ' {
                    let mut color = cell.fg_color;
                    let font_id = FontId::monospace(self.font_size);

                    if cell.bold {
                        color = Color32::from_rgb(
                            (color.r() as u16 * 3 / 2).min(255) as u8,
                            (color.g() as u16 * 3 / 2).min(255) as u8,
                            (color.b() as u16 * 3 / 2).min(255) as u8,
                        );
                    }

                    if !cell.italic {
                        ui.painter().text(
                            pos,
                            egui::Align2::LEFT_TOP,
                            cell.character,
                            font_id,
                            color,
                        );
                    } else {
                        let mut job = LayoutJob::default();
                        job.append(
                            &cell.character.to_string(),
                            0.0,
                            TextFormat {
                                font_id,
                                italics: true,
                                color,
                                ..Default::default()
                            },
                        );

                        let galley = ui.painter().layout_job(job);
                        ui.painter().galley(Pos2::new(pos.x, pos.y), galley, color);
                    }

                    if cell.underline {
                        let underline_y = pos.y + self.line_height - 2.0;
                        ui.painter().line_segment(
                            [
                                Pos2::new(pos.x, underline_y),
                                Pos2::new(pos.x + self.char_width, underline_y),
                            ],
                            egui::Stroke::new(1.0, color),
                        );
                    }
                }
            }
        }

        // Draw cursor
        self.draw_cursor(ui, &response.rect);

        response
    }

    fn draw_cursor(&mut self, ui: &mut egui::Ui, rect: &Rect) {
        if self.show_cursor {
            let cursor_pos = Pos2::new(
                rect.left() + self.buffer.cursor_x as f32 * self.char_width,
                rect.top() + self.buffer.cursor_y as f32 * self.line_height,
            );

            ui.painter().rect_filled(
                Rect::from_min_size(cursor_pos, egui::vec2(self.char_width, self.line_height)),
                0.0,
                Color32::from_rgba_premultiplied(255, 255, 255, 128),
            );
        }
    }

    pub fn handle_input(&mut self, ctx: &egui::Context) -> Vec<u8> {
        let mut output = Vec::new();

        ctx.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Key {
                        key, pressed: true, ..
                    } => match key {
                        egui::Key::Enter => {
                            output.extend_from_slice(b"\r");
                            self.buffer.new_line();
                        }
                        egui::Key::Backspace => {
                            output.extend_from_slice(b"\x08");
                            self.buffer.backspace();
                        }
                        egui::Key::Tab => {
                            output.extend_from_slice(b"\t");
                            for _ in 0..4 {
                                self.buffer.put_char(' ');
                            }
                        }
                        _ => {}
                    },
                    egui::Event::Text(text) => {
                        output.extend_from_slice(text.as_bytes());
                        for ch in text.chars() {
                            if ch.is_control() {
                                continue;
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        output
    }

    pub fn process_output(&mut self, data: &[u8]) {
        let text = String::from_utf8_lossy(data);

        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '\r' => self.buffer.carriage_return(),
                '\n' => self.buffer.new_line(),
                '\t' => {
                    for _ in 0..4 {
                        self.buffer.put_char(' ');
                    }
                }
                '\x08' => self.buffer.backspace(), // Handle backspace
                '\x1b' => {
                    // Handle escape sequences
                    if let Some('[') = chars.peek() {
                        chars.next(); // Skip '['
                        self.process_csi_sequence(&mut chars);
                    }
                }
                ch if ch.is_control() => {}
                ch => self.buffer.put_char(ch),
            }
        }
    }

    fn process_csi_sequence(&mut self, chars: &mut std::iter::Peekable<std::str::Chars>) {
        let mut sequence = String::new();

        // Collect CSI sequences
        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_alphabetic() {
                sequence.push(ch);
                chars.next();
                break;
            } else if ch.is_ascii_digit() || ch == ';' || ch == '?' {
                sequence.push(ch);
                chars.next();
            } else {
                break;
            }
        }

        debug!("Processing CSI sequence: {sequence}");

        // Process the CSI sequence
        match sequence.as_str() {
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
                let x = parts
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                let y = parts
                    .get(1)
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                self.buffer
                    .move_cursor(x.saturating_sub(1), y.saturating_sub(1));
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
                todo!("Send response back to PTY: {response}");
                // TODO: HERE
            }

            // Erase in Display/Line - Erase in Display
            ch if ch.ends_with('J') => {
                let num = sequence.trim_end_matches('J').parse::<usize>().unwrap_or(0);
                match num {
                    0 => self.buffer.clear_screen(),
                    // 1 => Erase entire screen (may include scroll buffer),
                    // 2 => Erase the entire screen, including the scroll buffer (Linux 3.0 and later)
                    _ => self.buffer.scroll_region_top = self.buffer.cursor_y,
                }
            }

            // Erase in Display/Line - Erase in Line
            ch if ch.ends_with('K') => {
                let num = sequence.trim_end_matches('K').parse::<usize>().unwrap_or(0);
                match num {
                    0 => {
                        // Erase from cursor to end of line
                        for x in self.buffer.cursor_x..self.buffer.width {
                            self.buffer.cells[self.buffer.cursor_y][x] = Default::default();
                        }
                    }
                    1 => {
                        // Erase from start of line to cursor
                        for x in 0..=self.buffer.cursor_x {
                            self.buffer.cells[self.buffer.cursor_y][x] = Default::default();
                        }
                    }
                    2 => {
                        // Erase entire line
                        for x in 0..self.buffer.width {
                            self.buffer.cells[self.buffer.cursor_y][x] = Default::default();
                        }
                    }
                    _ => {}
                }
            }

            // Select Graphic Rendition (SGR)
            ch if ch.ends_with('m') => {
                let params: Vec<&str> = sequence.trim_end_matches('m').split(';').collect();
                for param in params {
                    match param {
                        "0" => {
                            // Reset all attributes
                            self.buffer.current_fg_color = Color32::WHITE;
                            self.buffer.current_bg_color = Color32::TRANSPARENT;
                            self.buffer.current_bold = false;
                            self.buffer.current_underline = false;
                            self.buffer.current_italic = false;
                        }
                        "1" => self.buffer.current_bold = true,
                        "2" => {
                            let current_color = self.buffer.current_fg_color;
                            self.buffer.current_fg_color = Color32::from_rgb(
                                (current_color.r() as u16 * 4 / 5).min(255) as u8,
                                (current_color.g() as u16 * 4 / 5).min(255) as u8,
                                (current_color.b() as u16 * 4 / 5).min(255) as u8,
                            );
                        }
                        "3" => self.buffer.current_italic = true,
                        "4" => self.buffer.current_underline = true,
                        // "5" => Blink
                        // "7" => Reverse
                        // "8" => Hidden
                        // "9" => Strikethrough
                        _ if param.contains("30") => {
                            self.buffer.current_fg_color = Color32::BLACK;
                        }
                        _ if param.contains("31") => {
                            self.buffer.current_fg_color = Color32::RED;
                        }
                        _ if param.contains("32") => {
                            self.buffer.current_fg_color = Color32::GREEN;
                        }
                        _ if param.contains("33") => {
                            self.buffer.current_fg_color = Color32::YELLOW;
                        }
                        _ if param.contains("34") => {
                            self.buffer.current_fg_color = Color32::BLUE;
                        }
                        _ if param.contains("35") => {
                            self.buffer.current_fg_color = Color32::MAGENTA;
                        }
                        _ if param.contains("36") => {
                            self.buffer.current_fg_color = Color32::CYAN;
                        }
                        _ if param.contains("37") => {
                            self.buffer.current_fg_color = Color32::WHITE;
                        }
                        _ if param.starts_with("38;5;") => {
                            // 256-color foreground
                            if let Ok(color_index) = param[5..].parse::<u8>() {
                                self.buffer.current_fg_color =
                                    Color32::from_rgb(color_index, color_index, color_index);
                            }
                        }
                        _ if param.starts_with("38;2;") => {
                            // 24bit RGB foreground
                            let parts: Vec<&str> = param[5..].split(';').collect();
                            if parts.len() == 3
                                && let (Ok(r), Ok(g), Ok(b)) = (
                                    parts[0].parse::<u8>(),
                                    parts[1].parse::<u8>(),
                                    parts[2].parse::<u8>(),
                                )
                            {
                                self.buffer.current_fg_color = Color32::from_rgb(r, g, b);
                            }
                        }
                        _ if param.starts_with("40") => {
                            self.buffer.current_bg_color = Color32::BLACK;
                        }
                        _ if param.starts_with("41") => {
                            self.buffer.current_bg_color = Color32::RED;
                        }
                        _ if param.starts_with("42") => {
                            self.buffer.current_bg_color = Color32::GREEN;
                        }
                        _ if param.starts_with("43") => {
                            self.buffer.current_bg_color = Color32::YELLOW;
                        }
                        _ if param.starts_with("44") => {
                            self.buffer.current_bg_color = Color32::BLUE;
                        }
                        _ if param.starts_with("45") => {
                            self.buffer.current_bg_color = Color32::MAGENTA;
                        }
                        _ if param.starts_with("46") => {
                            self.buffer.current_bg_color = Color32::CYAN;
                        }
                        _ if param.starts_with("47") => {
                            self.buffer.current_bg_color = Color32::WHITE;
                        }
                        _ if param.starts_with("48;5;") => {
                            // 256-color background
                            if let Ok(color_index) = param[5..].parse::<u8>() {
                                self.buffer.current_bg_color =
                                    Color32::from_rgb(color_index, color_index, color_index);
                            }
                        }
                        _ if param.starts_with("48;2;") => {
                            // 24bit RGB background
                            let parts: Vec<&str> = param[5..].split(';').collect();
                            if parts.len() == 3
                                && let (Ok(r), Ok(g), Ok(b)) = (
                                    parts[0].parse::<u8>(),
                                    parts[1].parse::<u8>(),
                                    parts[2].parse::<u8>(),
                                )
                            {
                                self.buffer.current_bg_color = Color32::from_rgb(r, g, b);
                            }
                        }
                        _ => {}
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
            // ch if ch.ends_with('P') => {} // Delete characters
            // ch if ch.ends_with('X') => {} // Erase characters
            // ch if ch.ends_with('@') => {} // Insert characters

            // Set Mode/Reset Mode
            // Not implemented yet

            // Other CSI sequences
            _ => {}
        }
    }
}
