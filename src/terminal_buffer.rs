use std::vec;

use eframe::egui::Color32;

use crate::terminal_cell::TerminalCell;

pub struct TerminalBuffer {
    pub cells: Vec<Vec<TerminalCell>>,
    pub width: usize,
    pub height: usize,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub scroll_region_top: usize,
    pub scroll_region_bottom: usize,
    pub current_fg_color: Color32,
    pub current_bg_color: Color32,
    pub current_bold: bool,
    pub current_underline: bool,
    pub current_italic: bool,
    pub saved_cursor_x: usize,
    pub saved_cursor_y: usize,
}

impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = Vec::with_capacity(height);
        for _ in 0..height {
            cells.push(vec![TerminalCell::default(); width]);
        }

        Self {
            cells,
            width,
            height,
            cursor_x: 0,
            cursor_y: 0,
            scroll_region_top: 0,
            scroll_region_bottom: height - 1,
            current_fg_color: Color32::WHITE,
            current_bg_color: Color32::TRANSPARENT,
            current_bold: false,
            current_underline: false,
            current_italic: false,
            saved_cursor_x: 0,
            saved_cursor_y: 0,
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        self.width = new_width;
        self.height = new_height;

        // Adjust the cells
        if self.cells.len() < new_height {
            while self.cells.len() < new_height {
                self.cells.push(vec![TerminalCell::default(); new_width]);
            }
        } else if self.cells.len() > new_height {
            self.cells.truncate(new_height);
        }

        // Adjust each row to the new width
        for row in &mut self.cells {
            if row.len() < new_width {
                row.resize(new_width, TerminalCell::default());
            } else if row.len() > new_width {
                row.truncate(new_width);
            }
        }

        // Adjust cursor position
        self.cursor_x = self.cursor_x.min(new_width.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(new_height.saturating_sub(1));
        self.scroll_region_bottom = new_height - 1;
    }

    pub fn put_char(&mut self, ch: char) {
        // If cursor position is out of bounds, move to a new row
        if self.cursor_x >= self.width {
            self.new_line();
        }

        // Insert the character at the current cursor position
        if self.cursor_y < self.height {
            self.cells[self.cursor_y][self.cursor_x] = TerminalCell {
                character: ch,
                fg_color: self.current_fg_color,
                bg_color: self.current_bg_color,
                bold: self.current_bold,
                underline: self.current_underline,
                italic: self.current_italic,
            };
            self.cursor_x += 1;
        }
    }

    pub fn new_line(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y > self.scroll_region_bottom {
            self.scroll_up();
            self.cursor_y = self.scroll_region_bottom;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            self.cells[self.cursor_y][self.cursor_x] = TerminalCell::default();
        }
    }

    pub fn scroll_up(&mut self) {
        for y in self.scroll_region_top..self.scroll_region_bottom {
            self.cells[y] = self.cells[y + 1].clone();
        }
        self.cells[self.scroll_region_bottom] = vec![TerminalCell::default(); self.width];
    }

    pub fn clear_screen(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = TerminalCell::default();
            }
        }

        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    pub fn clear_range(
        &mut self,
        start_pos: Option<(usize, usize)>,
        end_pos: Option<(usize, usize)>,
    ) {
        let start_x = start_pos.map_or(0, |(x, _)| x);
        let start_y = start_pos.map_or(0, |(_, y)| y);
        let end_x = end_pos.map_or(self.width.saturating_sub(1), |(x, _)| x);
        let end_y = end_pos.map_or(self.height.saturating_sub(1), |(_, y)| y);

        // y range within the height of the buffer
        let y_start = start_y.min(self.height);
        let y_end = end_y.min(self.height.saturating_sub(1));

        for y in y_start..=y_end {
            // x range within the width of the buffer
            let x_start = start_x.min(self.width);
            let x_end = end_x.min(self.width.saturating_sub(1));

            if x_start <= x_end {
                self.cells[y][x_start..=x_end].fill(TerminalCell::default());
            }
        }
    }

    pub fn move_cursor(&mut self, x: usize, y: usize) {
        self.cursor_x = x.min(self.width.saturating_sub(1));
        self.cursor_y = y.min(self.height.saturating_sub(1));
    }

    pub fn carriage_return(&mut self) {
        self.cursor_x = 0;
    }
}
