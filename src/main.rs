mod app;
mod terminal_buffer;
mod terminal_cell;
mod terminal_widget;

use crate::app::App;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Portable PTY Example")
            .with_inner_size(eframe::egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "explotty",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
