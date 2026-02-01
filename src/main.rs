mod app;
mod dependency;
mod events;
mod list;
mod maven_registry;
mod ui;

use app::App;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut terminal = ratatui::init();
    App::new()?.run(&mut terminal).await?;
    ratatui::restore();
    return Ok(());
}
