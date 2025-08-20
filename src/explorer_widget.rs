use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use eframe::egui::{self, RichText};
use egui_extras::{Size, StripBuilder};

use crate::utils::{
    get_desc_from_mime_type, get_formatted_icon_path, get_mime_type_from_path,
    to_human_readable_size,
};

const COLUMN_SIZES: [f32; 4] = [100.0, 80.0, 80.0, 120.0];
const HEADER_HEIGHT: f32 = 28.0;
const ROW_HEIGHT: f32 = 24.0;

/// The main widget for exploring files and directories
pub struct ExplorerWidget {
    /// The list of files and directories in the current directory
    files: Vec<FileItem>,
    /// The current directory being explored
    current_directory: Option<PathBuf>,
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
                .clone()
                .map_or("N/A".to_string(), |path| path.to_string_lossy().to_string())
        ));
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                StripBuilder::new(ui)
                    .size(Size::exact(HEADER_HEIGHT))
                    .sizes(Size::exact(ROW_HEIGHT), self.files.len())
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            StripBuilder::new(ui)
                                .size(Size::remainder().at_least(COLUMN_SIZES[0]))
                                .size(Size::exact(COLUMN_SIZES[1]))
                                .size(Size::exact(COLUMN_SIZES[2]))
                                .size(Size::exact(COLUMN_SIZES[3]))
                                .horizontal(|mut strip| {
                                    let contents = ["Name", "Size", "Type", "Modified"];
                                    for title in contents {
                                        Self::render_cell(&mut strip, |ui| ui.label(title));
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
                                            &Self::get_absolute_path_string(
                                                self.current_directory.clone(),
                                                &file.name,
                                            )
                                            .unwrap_or_default(),
                                        );
                                    }
                                });

                                StripBuilder::new(ui)
                                    .size(Size::remainder().at_least(COLUMN_SIZES[0]))
                                    .size(Size::exact(COLUMN_SIZES[1]))
                                    .size(Size::exact(COLUMN_SIZES[2]))
                                    .size(Size::exact(COLUMN_SIZES[3]))
                                    .horizontal(|mut strip| {
                                        Self::render_cell(&mut strip, |ui| {
                                            ui.image(&file.icon_path);
                                            ui.label(if file.is_hidden {
                                                RichText::new(&file.name)
                                                    .color(egui::Color32::DARK_GRAY)
                                            } else {
                                                RichText::new(&file.name)
                                            });
                                        });

                                        let contents = [
                                            file.size.clone(),
                                            file.file_type.clone(),
                                            file.modified_at.clone(),
                                        ];

                                        for content in contents {
                                            Self::render_cell(&mut strip, |ui| ui.label(content));
                                        }
                                    });
                            })
                        }
                    });
            });
    }

    fn render_cell<R>(strip: &mut egui_extras::Strip<'_, '_>, f: impl FnOnce(&mut egui::Ui) -> R) {
        strip.cell(|ui| {
            egui::ScrollArea::horizontal()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.allocate_ui_with_layout(
                        ui.available_size(),
                        egui::Layout::left_to_right(egui::Align::Center),
                        f,
                    );
                });
        })
    }

    fn open_file(file: &FileItem, current_directory: Option<PathBuf>) {
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

        for entry in
            fs::read_dir(self.current_directory.clone().unwrap_or_default())?.filter_map(Result::ok)
        {
            let path = entry.path();
            if path.is_dir() {
                self.files.push(FileItem {
                    name: entry.file_name().to_string_lossy().to_string(),
                    size: "--".to_string(),
                    file_type: "Directory".to_string(),
                    modified_at: "--".to_string(),
                    is_directory: true,
                    is_hidden: entry.file_name().to_string_lossy().starts_with('.'),
                    icon_path: get_formatted_icon_path("inode/directory", 48),
                });
            } else {
                let mime_type = get_mime_type_from_path(&path);
                let metadata = entry.metadata()?;
                let file_type = get_desc_from_mime_type(&mime_type);
                let size = to_human_readable_size(metadata.len());
                let modified: DateTime<Local> = metadata.modified()?.into();
                let formatted_modified = modified.format("%Y-%m-%d %H:%M").to_string();

                self.files.push(FileItem {
                    name: entry.file_name().to_string_lossy().to_string(),
                    size,
                    file_type,
                    modified_at: formatted_modified,
                    is_directory: false,
                    is_hidden: entry.file_name().to_string_lossy().starts_with('.'),
                    icon_path: get_formatted_icon_path(&mime_type, 48),
                });
            }
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

    fn get_absolute_path_string(
        current_directory: Option<PathBuf>,
        item_name: &str,
    ) -> Option<String> {
        if let Some(current_dir) = current_directory {
            let mut path = current_dir;
            path.push(item_name);
            Some(path.to_string_lossy().to_string())
        } else {
            None
        }
    }
}
