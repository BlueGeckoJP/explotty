mod terminal_buffer;
mod terminal_cell;
mod terminal_widget;

use eframe::egui;
use portable_pty::{Child, PtyPair};

use crate::terminal_widget::TerminalWidget;

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

struct App {
    terminal_widget: TerminalWidget,
    pty_pair: Option<PtyPair>,
    child: Option<Box<dyn Child + Send + Sync>>,
}

impl Default for App {
    fn default() -> Self {
        Self {}
    }
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        App::default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("This is a simple example of using portable_pty with eframe.");
            if ui.button("Spawn PTY").clicked() {}
        });
    }
}
