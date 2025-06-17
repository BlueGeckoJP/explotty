use eframe::egui::{self, Color32, FontId, Pos2, Rect, TextFormat, text::LayoutJob};

use crate::{terminal_buffer::TerminalBuffer, terminal_cell::TerminalCell};

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
                        }
                        egui::Key::Backspace => {
                            output.extend_from_slice(b"\x08");
                        }
                        egui::Key::Tab => {
                            output.extend_from_slice(b"\t");
                        }
                        egui::Key::ArrowUp => {
                            output.extend_from_slice(b"\x1b[A");
                        }
                        egui::Key::ArrowDown => {
                            output.extend_from_slice(b"\x1b[B");
                        }
                        egui::Key::ArrowLeft => {
                            output.extend_from_slice(b"\x1b[D");
                        }
                        egui::Key::ArrowRight => {
                            output.extend_from_slice(b"\x1b[C");
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
                todo!("Send response back to PTY: {response}");
                // TODO: HERE
            }

            // Erase in Display/Line - Erase in Display
            ch if ch.ends_with('J') => {
                let num = sequence.trim_end_matches('J').parse::<usize>().unwrap_or(0);
                match num {
                    0 => self
                        .buffer
                        .clear_range(Some((self.buffer.cursor_x, self.buffer.cursor_y)), None),
                    1 => self
                        .buffer
                        .clear_range(None, Some((self.buffer.cursor_x, self.buffer.cursor_y))),
                    2 => self.buffer.clear_screen(),
                    3 => self.buffer.clear_screen(), // Clear entire screen (including scrollback) (Not implemented yet, and same behaviour as 2)
                    _ => {
                        warn!("Unsupported erase in display parameter: {num}");
                    }
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
                let original_sequence = sequence.clone();
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
                                        process_256_color_palette(color_index);
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
                                        process_256_color_palette(color_index);
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
                            self.buffer.current_fg_color = to_bright(Color32::BLACK);
                        }
                        "91" => {
                            self.buffer.current_fg_color = to_bright(Color32::RED);
                        }
                        "92" => {
                            self.buffer.current_fg_color = to_bright(Color32::GREEN);
                        }
                        "93" => {
                            self.buffer.current_fg_color = to_bright(Color32::YELLOW);
                        }
                        "94" => {
                            self.buffer.current_fg_color = to_bright(Color32::BLUE);
                        }
                        "95" => {
                            self.buffer.current_fg_color = to_bright(Color32::MAGENTA);
                        }
                        "96" => {
                            self.buffer.current_fg_color = to_bright(Color32::CYAN);
                        }
                        "97" => {
                            self.buffer.current_fg_color = to_bright(Color32::WHITE);
                        }
                        "100" => {
                            self.buffer.current_bg_color = to_bright(Color32::BLACK);
                        }
                        "101" => {
                            self.buffer.current_bg_color = to_bright(Color32::RED);
                        }
                        "102" => {
                            self.buffer.current_bg_color = to_bright(Color32::GREEN);
                        }
                        "103" => {
                            self.buffer.current_bg_color = to_bright(Color32::YELLOW);
                        }
                        "104" => {
                            self.buffer.current_bg_color = to_bright(Color32::BLUE);
                        }
                        "105" => {
                            self.buffer.current_bg_color = to_bright(Color32::MAGENTA);
                        }
                        "106" => {
                            self.buffer.current_bg_color = to_bright(Color32::CYAN);
                        }
                        "107" => {
                            self.buffer.current_bg_color = to_bright(Color32::WHITE);
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

            // Other CSI sequences
            _ => {
                warn!("Unhandled CSI sequence: {sequence}");
            }
        }
    }
}

fn process_256_color_palette(color_index: u8) -> Color32 {
    if color_index < 16 {
        // 16 basic colors
        match color_index {
            0 => Color32::BLACK,
            1 => Color32::RED,
            2 => Color32::GREEN,
            3 => Color32::YELLOW,
            4 => Color32::BLUE,
            5 => Color32::MAGENTA,
            6 => Color32::CYAN,
            7 => Color32::WHITE,
            8 => to_bright(Color32::BLACK),
            9 => to_bright(Color32::RED),
            10 => to_bright(Color32::GREEN),
            11 => to_bright(Color32::YELLOW),
            12 => to_bright(Color32::BLUE),
            13 => to_bright(Color32::MAGENTA),
            14 => to_bright(Color32::CYAN),
            15 => to_bright(Color32::WHITE),
            _ => unreachable!(),
        }
    } else if (16..232).contains(&color_index) {
        // 6x6x6 rgb color cube
        let r_6 = (color_index - 16) / 36;
        let g_6 = ((color_index - 16) % 36) / 6;
        let b_6 = (color_index - 16) % 6;

        let rgb: (u8, u8, u8) = [r_6, g_6, b_6]
            .map(|x| match x {
                0 => 0,
                1 => 95,
                2 => 135,
                3 => 175,
                4 => 215,
                5 => 255,
                _ => unreachable!(),
            })
            .into();

        Color32::from_rgb(rgb.0, rgb.1, rgb.2)
    } else {
        // 232..=255
        // Grayscale colors
        let gray_value = (color_index - 232) * 10 + 8; // 8, 18, ..., 238
        Color32::from_gray(gray_value)
    }
}

fn to_bright(color: Color32) -> Color32 {
    let rgb = color.to_array();
    Color32::from_rgb(
        (rgb[0] as f32 * 1.2).min(255.0) as u8,
        (rgb[1] as f32 * 1.2).min(255.0) as u8,
        (rgb[2] as f32 * 1.2).min(255.0) as u8,
    )
}
