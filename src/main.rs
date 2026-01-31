mod app;
mod dependency;
mod events;
mod maven_registry;
mod ui;

use app::App;
use color_eyre::Result;

use crate::dependency::MavenFile;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let maven = MavenFile::from_file("./static/pom.xml".to_string())?;

    let mut terminal = ratatui::init();
    App::new(maven)?.run(&mut terminal).await?;
    ratatui::restore();
    return Ok(());
}
