use eframe::egui;

pub struct ExplorerWidget {
    files: Vec<FileItem>,
    selected_item_index: Option<usize>,
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
                size: "".to_string(),
                file_type: "Directory".to_string(),
                modified: "2023-10-01 11:00".to_string(),
                is_directory: true,
            },
            // Add more sample files as needed
        ];

        Self {
            files,
            selected_item_index: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, pid: Option<u32>) {
        ui.label(format!(
            "Current Directory: {}",
            crate::utils::get_current_dir_from_pty(pid.unwrap_or(0))
                .unwrap_or_else(|| "N/A".to_string())
        ));
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("explorer_grid")
                .num_columns(4)
                .spacing([10.0, 5.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Name");
                    ui.label("Size");
                    ui.label("Type");
                    ui.label("Modified");
                    ui.end_row();

                    for (i, file) in self.files.iter().enumerate() {
                        ui.label(&file.name);
                        ui.label(&file.size);
                        ui.label(&file.file_type);
                        ui.label(&file.modified);
                        ui.end_row();
                    }
                });
        });
    }
}
