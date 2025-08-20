mod color;

use eframe::egui::{self, Color32, FontId, Pos2, Rect, TextFormat, text::LayoutJob};

use crate::{terminal_buffer::TerminalBuffer, terminal_cell::TerminalCell};

pub struct TerminalWidget {
    pub buffer: TerminalBuffer,
    pub font_size: f32,
    pub char_width: f32,
    pub line_height: f32,
    pub show_cursor: bool,
    pty_buffer: Vec<u8>,
    selection_start: Option<(usize, usize)>,
    selection_end: Option<(usize, usize)>,
    bracket_paste_mode: bool,
    // Storage location for current screen information used when Alternative Screen Buffer is used
    saved_screen_buffer: Option<TerminalBuffer>,
    decckm_mode: bool, // DECCKM - DEC Private Mode
    scroll_offset: usize,
    max_scroll_lines: usize,
    scrollback_buffer: Vec<Vec<TerminalCell>>,
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
            pty_buffer: Vec::new(),
            selection_start: None,
            selection_end: None,
            bracket_paste_mode: false,
            saved_screen_buffer: None,
            decckm_mode: false,
            scroll_offset: 0,
            max_scroll_lines: 1000,
            scrollback_buffer: Vec::new(),
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
            self.adjust_scrollback_buffer_width(cols);
        }

        let response = ui.allocate_response(available_size, egui::Sense::click_and_drag());

        // Handle scrolling with mouse wheel and keyboard
        self.handle_scroll(ui);

        // Selection logic
        let rect = response.rect;

        if response.drag_started()
            && let Some(pos) = response.hover_pos()
        {
            let col = ((pos.x - rect.left()) / self.char_width).floor() as usize;
            let row = ((pos.y - rect.top()) / self.line_height).floor() as usize;
            let clamped_col = col.min(self.buffer.width.saturating_sub(1));
            let clamped_row = row.min(self.buffer.height.saturating_sub(1));
            self.selection_start = Some((clamped_col, clamped_row));
            self.selection_end = Some((clamped_col, clamped_row));
        }

        if response.dragged()
            && let Some(pos) = response.hover_pos()
        {
            let col = ((pos.x - rect.left()) / self.char_width).floor() as usize;
            let row = ((pos.y - rect.top()) / self.line_height).floor() as usize;
            let clamped_col = col.min(self.buffer.width.saturating_sub(1));
            let clamped_row = row.min(self.buffer.height.saturating_sub(1));
            self.selection_end = Some((clamped_col, clamped_row));
        }

        if response.clicked() {
            self.selection_start = None;
            self.selection_end = None;
        }

        // Draw background
        ui.painter().rect_filled(response.rect, 0.0, Color32::BLACK);

        // Draw the terminal cells (characters) with scrolling consideration
        self.draw_terminal_content(ui, &rect);

        // Draw cursor (only when at the bottom of scroll)
        if self.scroll_offset == 0 {
            self.draw_cursor(ui, &rect);
        }

        // Draw selection
        self.draw_selection(ui, &rect);

        // Draw scroll indicator if scrolled
        if self.scroll_offset > 0 {
            self.draw_scroll_indicator(ui, &rect);
        }

        response
    }

    fn handle_scroll(&mut self, ui: &mut egui::Ui) {
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

    fn draw_terminal_content(&self, ui: &mut egui::Ui, rect: &Rect) {
        let visible_lines = self.get_visible_lines();

        for (row_index, row) in visible_lines.iter().enumerate() {
            for (col_index, cell) in row.iter().enumerate() {
                let pos = Pos2::new(
                    rect.left() + col_index as f32 * self.char_width,
                    rect.top() + row_index as f32 * self.line_height,
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
    }

    fn get_visible_lines(&self) -> Vec<Vec<TerminalCell>> {
        if self.scroll_offset == 0 {
            // At the bottom, show current buffer
            return self.buffer.cells.clone();
        }

        let mut visible_lines = Vec::new();

        for i in 0..self.buffer.height {
            let line_index_from_bottom = self.scroll_offset + self.buffer.height - 1 - i;

            if line_index_from_bottom < self.buffer.height {
                // This line is in the current buffer
                let buffer_line_index = self.buffer.height - 1 - line_index_from_bottom;
                visible_lines.push(self.buffer.cells[buffer_line_index].clone());
            } else {
                // This line is in the scrollback buffer
                let scrollback_index = line_index_from_bottom - self.buffer.height;
                if scrollback_index < self.scrollback_buffer.len() {
                    let scrollback_line_index = self.scrollback_buffer.len() - 1 - scrollback_index;
                    visible_lines.push(self.scrollback_buffer[scrollback_line_index].clone());
                } else {
                    // Empty line if we're beyond available history
                    visible_lines.push(vec![TerminalCell::default(); self.buffer.width]);
                }
            }
        }

        visible_lines
    }

    fn draw_scroll_indicator(&self, ui: &mut egui::Ui, rect: &Rect) {
        let indicator_text = format!("[↑{}]", self.scroll_offset);
        let indicator_pos = Pos2::new(rect.right() - 100.0, rect.top() + 10.0);

        ui.painter().text(
            indicator_pos,
            egui::Align2::LEFT_TOP,
            indicator_text,
            FontId::monospace(self.font_size * 0.8),
            Color32::YELLOW,
        );
    }

    fn add_line_to_scrollback(&mut self, line: Vec<TerminalCell>) {
        self.scrollback_buffer.push(line);

        // Limit the size of scrollback buffer
        if self.scrollback_buffer.len() > self.max_scroll_lines {
            self.scrollback_buffer.remove(0);
        }
    }

    fn adjust_scrollback_buffer_width(&mut self, new_width: usize) {
        // Adjust existing scrollback lines to new width
        for line in &mut self.scrollback_buffer {
            if line.len() < new_width {
                line.resize(new_width, TerminalCell::default());
            } else if line.len() > new_width {
                line.truncate(new_width);
            }
        }
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

    fn draw_selection(&self, ui: &mut egui::Ui, rect: &Rect) {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let (start_row, end_row) = (start.1.min(end.1), start.1.max(end.1));
            let (start_col, end_col) = (start.0.min(end.0), start.0.max(end.0));

            for r in start_row..=end_row {
                for c in start_col..=end_col {
                    let pos = Pos2::new(
                        rect.left() + c as f32 * self.char_width,
                        rect.top() + r as f32 * self.line_height,
                    );
                    let selection_rect = egui::Rect::from_min_size(
                        pos,
                        egui::vec2(self.char_width, self.line_height),
                    );
                    ui.painter().rect_filled(
                        selection_rect,
                        0.0,
                        Color32::from_rgba_premultiplied(100, 100, 100, 100),
                    );
                }
            }
        }
    }

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
                    self.buffer.new_line();
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

    fn process_csi_sequence(&mut self, sequence: &str) {
        debug!("Processing CSI sequence: {sequence}");

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

            // CSI ? 2004 h (Enable Bracketed Paste Mode)
            ch if ch.ends_with('h') && sequence.contains("2004") => {
                self.bracket_paste_mode = true;
            }
            // CSI ? 2004 l (Disable Bracketed Paste Mode)
            ch if ch.ends_with('l') && sequence.contains("2004") => {
                self.bracket_paste_mode = false;
            }

            // CSI ? 1049 h (Enable Alternate Screen Buffer)
            ch if ch.ends_with('h') && sequence.contains("1049") => {
                // Switch to alternate screen buffer
                let new_buffer = TerminalBuffer::new(self.buffer.width, self.buffer.height);
                self.saved_screen_buffer = Some(std::mem::replace(&mut self.buffer, new_buffer));
                self.buffer.cursor_x = 0;
                self.buffer.cursor_y = 0;
            }

            // CSI ? 1049 l (Disable Alternate Screen Buffer)
            ch if ch.ends_with('l') && sequence.contains("1049") => {
                // Switch back to main screen buffer
                if let Some(saved_buffer) = self.saved_screen_buffer.take() {
                    self.buffer = saved_buffer;
                } else {
                    warn!("No saved screen buffer to switch back to");
                }
                self.saved_screen_buffer = None;
            }

            // CSI ? 25 h (Show Cursor)
            ch if ch.ends_with('h') && sequence.contains("25") => {
                self.show_cursor = true;
            }

            // CSI ? 25 l (Hide Cursor)
            ch if ch.ends_with('l') && sequence.contains("25") => {
                self.show_cursor = false;
            }

            // CSI ? 1 h (DECCKM)
            ch if ch.ends_with('h') && sequence.contains("1") => {
                self.decckm_mode = true;
            }
            // CSI ? 1 l (Reset DECCKM)
            ch if ch.ends_with('l') && sequence.contains("1") => {
                self.decckm_mode = false;
            }

            // Other CSI sequences
            _ => {
                warn!("Unhandled CSI sequence: {sequence}");
            }
        }
    }

    fn process_osc_sequence(&mut self, ctx: &egui::Context, sequence: &str) {
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
