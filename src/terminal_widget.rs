use eframe::egui::{self, Color32, FontId, Pos2, Rect, RichText, TextFormat, text::LayoutJob};

use crate::terminal_buffer::TerminalBuffer;

pub struct TerminalWidget {
    pub buffer: TerminalBuffer,
    pub font_size: f32,
    pub char_width: f32,
    pub line_height: f32,
    pub show_cursor: bool,
    pub input_buffer: String,
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
            input_buffer: String::new(),
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
}
