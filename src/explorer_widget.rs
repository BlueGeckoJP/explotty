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
        Self {
            files: Vec::new(),
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
                    ui.heading("Name");
                    ui.heading("Size");
                    ui.heading("Type");
                    ui.heading("Modified");
                    ui.end_row();

                    for (i, file) in self.files.iter().enumerate() {
                        ui.label(&file.size);
                        ui.label(&file.file_type);
                        ui.label(&file.modified);
                        ui.end_row();
                    }
                });
        });
    }
}
