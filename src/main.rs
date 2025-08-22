mod app;
mod config;
mod explorer_widget;
mod terminal_buffer;
mod terminal_cell;
mod terminal_widget;
mod utils;

#[macro_use]
extern crate log;

use std::sync::{Arc, OnceLock};

use crate::app::App;

static CONFIG: OnceLock<Arc<config::Config>> = OnceLock::new();

fn main() -> eframe::Result {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK");
        return Err(eframe::Error::AppCreation(
            "Failed to initialize GTK".into(),
        ));
    }

    let config_path = config::Config::get_first_existing_path();
    match config_path {
        Some(path) => {
            let config = config::Config::load(&path).unwrap_or_default();
            CONFIG.set(Arc::new(config)).unwrap();
        }
        None => {
            warn!("No configuration file found, using default settings");
            CONFIG.set(Arc::new(config::Config::default())).unwrap();
        }
    }

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(eframe::egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "explotty",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
