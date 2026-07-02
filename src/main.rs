mod app;
mod terminal;
mod state;
mod config;
mod plugin;
mod template;

use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();
    log::info!("Starting OpenZoo Terminal Manager");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("OpenZoo - AI 终端管理器"),
        ..Default::default()
    };

    eframe::run_native(
        "OpenZoo",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("GUI 启动失败: {}", e))
}
