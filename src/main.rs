mod app;
mod dependency;
mod events;
mod maven_registry;
mod ui;
mod views;

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    App::new()?.run(&mut terminal).await?;
    ratatui::restore();

    return Ok(());
}
