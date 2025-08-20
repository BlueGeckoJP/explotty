use std::path::Path;

use chrono::{DateTime, Local};
use eframe::egui::{self, RichText};
use egui_extras::{Size, StripBuilder};
use walkdir::WalkDir;

use crate::utils::{
    get_desc_from_mime_type, get_formatted_icon_path, get_mime_type_from_path,
    to_human_readable_size,
};

/// The main widget for exploring files and directories
pub struct ExplorerWidget {
    /// The list of files and directories in the current directory
    files: Vec<FileItem>,
    /// The current directory being explored
    current_directory: Option<String>,
    /// The index of the currently selected file or directory
    selected_index: Option<usize>,
}

/// This structure containing file information to be displayed in the UI
struct FileItem {
    /// The name of the file or directory. Not including absolute path
    name: String,
    /// The size of the file or directory. Human readable format
    size: String,
    /// The type description of the file or directory
    file_type: String,
    /// The last modified date and time of the file or directory
    modified_at: String,
    /// Whether the item is a directory
    is_directory: bool,
    /// Whether the item is hidden (starts with a dot)
    is_hidden: bool,
    /// The URI path to the icon (starts with file:///)
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
                                    Self::open_file(file, self.current_directory.clone());
                                }

                                response.context_menu(|ui| {
                                    if ui.button("Open").clicked() {
                                        Self::open_file(file, self.current_directory.clone());
                                    }
                                    if ui.button("Copy").clicked() {
                                        crate::utils::copy_file_uri_to_clipboard(
                                            Path::new(
                                                &self.current_directory.clone().unwrap_or_default(),
                                            )
                                            .join(&file.name)
                                            .to_str()
                                            .unwrap_or(""),
                                        );
                                    }
                                });

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
                                                            ui.label(if file.is_hidden {
                                                                RichText::new(&file.name)
                                                                    .color(egui::Color32::DARK_GRAY)
                                                            } else {
                                                                RichText::new(&file.name)
                                                            });
                                                        },
                                                    );
                                                });
                                        });

                                        let contents = [
                                            file.size.clone(),
                                            file.file_type.clone(),
                                            file.modified_at.clone(),
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

    fn open_file(file: &FileItem, current_directory: Option<String>) {
        if file.is_directory {
            if let Some(input) = crate::app::INPUT_BUFFER.get() {
                let cd_command = format!("cd {}", file.name.replace(" ", "\\ "));
                let b = format!("\x15{cd_command}/\r");

                let mut input = input.lock();
                input.extend_from_slice(b.as_bytes());
            }
        } else {
            let current_dir = current_directory.clone().unwrap_or_default();
            let file_path = Path::new(&current_dir).join(&file.name);
            if let Err(e) = open::that(file_path) {
                log::error!("Failed to open file: {e}");
            }
        }
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
                    modified_at: "--".to_string(),
                    is_directory: true,
                    is_hidden: false,
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
                modified_at: formatted_modified,
                is_directory: metadata.is_dir(),
                is_hidden: entry.file_name().to_string_lossy().starts_with('.'),
                icon_path: get_formatted_icon_path(&mime_type, 48),
            });
        }

        self.files.sort_by(|a, b| {
            if !a.is_hidden && b.is_hidden {
                std::cmp::Ordering::Less
            } else if a.is_hidden && !b.is_hidden {
                std::cmp::Ordering::Greater
            } else if a.is_directory && !b.is_directory {
                std::cmp::Ordering::Less
            } else if !a.is_directory && b.is_directory {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });

        Ok(())
    }
}
