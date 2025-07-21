use std::path::Path;

use chrono::{DateTime, Local};
use eframe::egui;
use egui_extras::{Size, StripBuilder};
use walkdir::WalkDir;

use crate::utils::{
    get_desc_from_mime_type, get_formatted_icon_path, get_mime_type_from_path,
    to_human_readable_size,
};

pub struct ExplorerWidget {
    files: Vec<FileItem>,
    current_directory: Option<String>,
    selected_index: Option<usize>,
}

struct FileItem {
    name: String,
    size: String,
    file_type: String,
    modified: String,
    is_directory: bool,
    icon_path: String,
}

impl ExplorerWidget {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            current_directory: None,
            selected_index: None,
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
                                let is_selected = self.selected_index == Some(index);

                                let bg_color = if is_selected {
                                    ui.style().visuals.selection.bg_fill
                                } else if index % 2 == 1 {
                                    ui.style().visuals.faint_bg_color
                                } else {
                                    egui::Color32::TRANSPARENT
                                };

                                if bg_color != egui::Color32::TRANSPARENT {
                                    ui.painter().rect_filled(
                                        ui.available_rect_before_wrap(),
                                        0.0,
                                        bg_color,
                                    );
                                }

                                let rect = ui.max_rect();
                                let id = ui.make_persistent_id(index);
                                let response = ui.interact(rect, id, egui::Sense::click());
                                if response.clicked() {
                                    self.selected_index = Some(index);
                                }

                                if response.double_clicked() {
                                    if file.is_directory
                                        && let Some(input) = crate::app::INPUT_BUFFER.get()
                                    {
                                        let cd_command =
                                            format!("cd {}", file.name.replace(" ", "\\ "));
                                        let b = format!("\x15{cd_command}/\r");

                                        let mut input = input.lock();
                                        input.extend_from_slice(b.as_bytes());
                                    } else {
                                        let current_dir =
                                            self.current_directory.clone().unwrap_or_default();
                                        let file_path = Path::new(&current_dir).join(&file.name);
                                        if let Err(e) = open::that(file_path) {
                                            log::error!("Failed to open file: {e}");
                                        }
                                    }
                                }

                                StripBuilder::new(ui)
                                    .size(Size::remainder().at_least(100.0))
                                    .size(Size::exact(80.0))
                                    .size(Size::exact(80.0))
                                    .size(Size::exact(120.0))
                                    .horizontal(|mut strip| {
                                        strip.cell(|ui| {
                                            egui::ScrollArea::horizontal()
                                                .auto_shrink([false, false])
                                                .show(ui, |ui| {
                                                    ui.allocate_ui_with_layout(
                                                        ui.available_size(),
                                                        egui::Layout::left_to_right(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            ui.image(&file.icon_path);
                                                            ui.label(&file.name);
                                                        },
                                                    );
                                                });
                                        });

                                        let contents = [
                                            file.size.clone(),
                                            file.file_type.clone(),
                                            file.modified.clone(),
                                        ];

                                        for content in contents {
                                            strip.cell(|ui| {
                                                egui::ScrollArea::horizontal()
                                                    .auto_shrink([false, false])
                                                    .show(ui, |ui| {
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
        self.selected_index = None;

        if let Some(current_dir) = &self.current_directory {
            let path = Path::new(current_dir);
            if path.parent().is_some() {
                self.files.push(FileItem {
                    name: "..".to_string(),
                    size: "--".to_string(),
                    file_type: "Directory".to_string(),
                    modified: "--".to_string(),
                    is_directory: true,
                    icon_path: get_formatted_icon_path("inode/directory", 48),
                });
            }
        }

        for entry in WalkDir::new(self.current_directory.clone().unwrap_or_default())
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
        {
            let mime_type = get_mime_type_from_path(entry.path());
            let metadata = entry.metadata()?;
            let file_type = if metadata.is_dir() {
                "Directory".to_string()
            } else {
                get_desc_from_mime_type(&mime_type)
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
                icon_path: get_formatted_icon_path(&mime_type, 48),
            });
        }

        Ok(())
    }
}
