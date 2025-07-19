use chrono::{DateTime, Local};
use eframe::egui;
use egui_extras::{Size, StripBuilder};
use walkdir::WalkDir;

use crate::utils::to_human_readable_size;

pub struct ExplorerWidget {
    files: Vec<FileItem>,
    current_directory: Option<String>,
}

struct FileItem {
    name: String,
    size: String,
    file_type: String,
    modified: String,
    is_directory: bool,
}

impl ExplorerWidget {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            current_directory: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, pid: Option<u32>) {
        let new_directory = crate::utils::get_current_dir_from_pty(pid.unwrap_or(0));
        if new_directory != self.current_directory {
            self.current_directory = new_directory;
            if let Err(e) = self.refresh_files() {
                ui.label(format!("Error refreshing files: {e}"));
            }
        }

        ui.label(format!(
            "Current Directory: {}",
            self.current_directory
                .as_ref()
                .unwrap_or(&"N/A".to_string())
        ));
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                StripBuilder::new(ui)
                    .size(Size::exact(28.0))
                    .sizes(Size::exact(24.0), self.files.len())
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            StripBuilder::new(ui)
                                .size(Size::remainder().at_least(100.0))
                                .size(Size::exact(80.0))
                                .size(Size::exact(80.0))
                                .size(Size::exact(120.0))
                                .horizontal(|mut strip| {
                                    let contents = ["Name", "Size", "Type", "Modified"];
                                    for title in contents {
                                        strip.cell(|ui| {
                                            ui.allocate_ui_with_layout(
                                                ui.available_size(),
                                                egui::Layout::left_to_right(egui::Align::Center),
                                                |ui| {
                                                    ui.label(title);
                                                },
                                            );
                                        });
                                    }
                                });
                        });

                        for (index, file) in self.files.iter().enumerate() {
                            strip.cell(|ui| {
                                if index % 2 == 1 {
                                    ui.painter().rect_filled(
                                        ui.available_rect_before_wrap(),
                                        0.0,
                                        ui.style().visuals.faint_bg_color,
                                    );
                                }

                                StripBuilder::new(ui)
                                    .size(Size::remainder().at_least(100.0))
                                    .size(Size::exact(80.0))
                                    .size(Size::exact(80.0))
                                    .size(Size::exact(120.0))
                                    .horizontal(|mut strip| {
                                        let contents = [
                                            if file.is_directory {
                                                format!("📁 {}", file.name)
                                            } else {
                                                format!("📄 {}", file.name)
                                            },
                                            file.size.clone(),
                                            file.file_type.clone(),
                                            file.modified.clone(),
                                        ];
                                        for content in contents {
                                            strip.cell(|ui| {
                                                ui.allocate_ui_with_layout(
                                                    ui.available_size(),
                                                    egui::Layout::left_to_right(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        ui.label(content);
                                                    },
                                                );
                                            });
                                        }
                                    });
                            })
                        }
                    });
            });
    }
    pub fn refresh_files(&mut self) -> anyhow::Result<()> {
        self.files.clear();
        for entry in WalkDir::new(self.current_directory.clone().unwrap_or_default())
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
        {
            let metadata = entry.metadata()?;
            let file_type = if metadata.is_dir() {
                "Directory".to_string()
            } else {
                "File".to_string()
            };
            let size = if metadata.is_dir() {
                "--".to_string()
            } else {
                to_human_readable_size(metadata.len())
            };
            let modified: DateTime<Local> = metadata.modified()?.into();
            let formatted_modified = modified.format("%Y-%m-%d %H:%M").to_string();

            self.files.push(FileItem {
                name: entry.file_name().to_string_lossy().to_string(),
                size,
                file_type,
                modified: formatted_modified,
                is_directory: metadata.is_dir(),
            });
        }
        Ok(())
    }
}
