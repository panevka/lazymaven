mod app;
mod dependency;
mod events;
mod list;
mod maven_registry;
mod ui;

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    App::new()?.run(&mut terminal).await?;
    ratatui::restore();

    return Ok(());
}
