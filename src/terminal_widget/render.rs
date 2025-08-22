use eframe::egui::{self, Color32, FontId, Pos2, Rect, TextFormat, text::LayoutJob};

use crate::terminal_widget::TerminalWidget;

impl TerminalWidget {
    pub fn draw_terminal_content(&self, ui: &mut egui::Ui, rect: &Rect) {
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
                if cell.character != ' ' && !cell.wide_tail {
                    // Draw debug outline if debug-outline feature is enabled
                    #[cfg(feature = "debug-outline")]
                    {
                        use egui::Stroke;

                        ui.painter().rect(
                            Rect {
                                min: pos,
                                max: pos + egui::vec2(self.char_width, self.line_height),
                            },
                            0,
                            Color32::TRANSPARENT,
                            Stroke::new(1.0, Color32::RED),
                            egui::StrokeKind::Middle,
                        );
                    }

                    let mut color = cell.fg_color;
                    let font_id = FontId::monospace(self.font_size);

                    if cell.bold {
                        color = Color32::from_rgb(
                            (color.r() as u16 * 3 / 2).min(255) as u8,
                            (color.g() as u16 * 3 / 2).min(255) as u8,
                            (color.b() as u16 * 3 / 2).min(255) as u8,
                        );
                    }

                    let mut job = LayoutJob::default();
                    job.append(
                        &cell.character.to_string(),
                        0.0,
                        TextFormat {
                            font_id,
                            italics: cell.italic,
                            color,
                            ..Default::default()
                        },
                    );

                    let galley = ui.painter().layout_job(job);
                    ui.painter().galley(Pos2::new(pos.x, pos.y), galley, color);

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

    pub fn draw_cursor(&mut self, ui: &mut egui::Ui, rect: &Rect) {
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

    pub fn draw_selection(&self, ui: &mut egui::Ui, rect: &Rect) {
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

    pub fn draw_scroll_indicator(&self, ui: &mut egui::Ui, rect: &Rect) {
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
}
