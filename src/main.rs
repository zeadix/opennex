mod app;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    log::info!("Starting OpenZoo Terminal Manager");
    
    let mut app = app::App::new().await?;
    app.run().await?;
    
    Ok(())
}
