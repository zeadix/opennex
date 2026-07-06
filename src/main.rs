use anyhow::Result;
use eframe::egui;

mod app;
mod completion;
mod snapshot;

fn main() -> Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("OpenZoo - AI Terminal Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "OpenZoo",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to start: {}", e))
}
