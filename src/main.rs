mod app;
mod terminal;
mod ui;
mod keybindings;
mod state;
mod config;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    log::info!("Starting OpenZoo Terminal Manager");
    
    let mut app = app::App::new().await?;
    app.run().await?;
    
    Ok(())
}
