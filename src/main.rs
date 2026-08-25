// Windows: build as a GUI-subsystem binary in release builds. Without
// this the default console subsystem makes Windows spawn an extra cmd.exe
// window whenever the app is launched from a shortcut/Explorer. Debug
// builds keep the console so log output stays visible during development.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use eframe::egui;

mod app;
mod completion;
mod history_db;
mod i18n;
mod persist;
mod proc_stats;
mod shells;
mod terminal;
mod theme;
mod updater;

fn main() -> Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("OpenNex - AI Terminal Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "OpenNex",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to start: {}", e))
}
