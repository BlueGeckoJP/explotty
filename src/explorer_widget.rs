use eframe::egui;
use walkdir::WalkDir;

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
        let files = vec![
            FileItem {
                name: "example.txt".to_string(),
                size: "1.2 KB".to_string(),
                file_type: "Text File".to_string(),
                modified: "2023-10-01 12:00".to_string(),
                is_directory: false,
            },
            FileItem {
                name: "documents".to_string(),
                size: "--".to_string(),
                file_type: "Directory".to_string(),
                modified: "2023-10-01 11:00".to_string(),
                is_directory: true,
            },
        ];

        Self {
            files,
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
                format!("{} bytes", metadata.len())
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
