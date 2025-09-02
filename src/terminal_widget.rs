mod color;
mod input;
mod parser;
mod parser_csi;
mod parser_osc;
mod parser_vt100;
mod render;

use eframe::egui::{self, Color32};

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
    // DEC Private Mode states
    decckm_mode: bool,    // DECCKM - Cursor Key Application Mode (?1h/l)
    decom_mode: bool,     // DECOM - Origin Mode (?6h/l)
    decawm_mode: bool,    // DECAWM - Auto Wrap Mode (?7h/l)
    reverse_video_mode: bool, // DECSCNM - Screen Reverse Video (?5h/l)
    scroll_offset: usize,
    max_scroll_lines: usize,
    scrollback_buffer: Vec<Vec<TerminalCell>>,
    new_line_mode: bool,
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
            // Initialize DEC Private Mode states to their default values
            decckm_mode: false,      // Cursor key normal mode
            decom_mode: false,       // Absolute origin mode
            decawm_mode: true,       // Auto wrap mode enabled by default
            reverse_video_mode: false, // Normal video mode
            scroll_offset: 0,
            max_scroll_lines: 1000,
            scrollback_buffer: Vec::new(),
            new_line_mode: true,
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
}
