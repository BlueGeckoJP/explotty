use eframe::egui;
use walkdir::WalkDir;

use crate::utils::to_human_readable_size;

pub struct ExplorerWidget {
    files: Vec<FileItem>,
    selected_item_index: Option<usize>,
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
            selected_item_index: None,
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
                let available_width = ui.available_width();

                egui::Grid::new("explorer_grid")
                    .num_columns(4)
                    .spacing([10.0, 5.0])
                    .striped(true)
                    .min_col_width(available_width / 4.0)
                    .show(ui, |ui| {
                        ui.label("Name");
                        ui.label("Size");
                        ui.label("Type");
                        ui.label("Modified");
                        ui.end_row();

                        for file in &self.files {
                            ui.label(if file.is_directory {
                                format!("📁 {}", file.name)
                            } else {
                                format!("📄 {}", file.name)
                            });
                            ui.label(&file.size);
                            ui.label(&file.file_type);
                            ui.label(&file.modified);
                            ui.end_row();
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
            let modified = metadata
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| format!("{d:?}"))
                .unwrap_or_else(|_| "N/A".to_string());

            self.files.push(FileItem {
                name: entry.file_name().to_string_lossy().to_string(),
                size,
                file_type,
                modified,
                is_directory: metadata.is_dir(),
            });
        }
        Ok(())
    }
}
