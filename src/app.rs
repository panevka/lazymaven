use anyhow::{Context, Result};
use dependency::JavaDependency;
use maven_registry::SearchResponseDoc;
use ratatui::{DefaultTerminal, widgets::ListState};
use tokio::sync::mpsc;

use crate::{
    dependency::{self, MavenFile},
    events::{
        self, AppAsyncOrchestrator, AppEvent, AppExecutor, AppIntentHandler, Effect, EventContext,
    },
    maven_registry, ui,
    ui::UI,
    views::{
        View, ViewId, dependency_search_view::DependencySearchView, dependency_view::DependencyView,
    },
};

pub struct App {
    tx: mpsc::Sender<events::AppEvent>,
    rx: mpsc::Receiver<events::AppEvent>,
    state: AppState,
}

pub struct AppState {
    pub ui_state: UIState,
    pub data: Data,
}

pub struct UIState {
    pub views: Vec<(ViewId, Box<dyn View>)>,
    pub currently_focused_view: ViewId,
}

pub struct Data {
    pub mode: InteractionMode,
    pub maven_file: MavenFile,
    pub search_phrase: String,
    pub found_dependencies: Vec<SearchResponseDoc>,
    pub dependencies: Vec<JavaDependency>,
    pub exit: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum InteractionMode {
    Normal,
    Input,
}

impl App {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel::<events::AppEvent>(100);

        let me = Self {
            tx,
            rx,
            state: AppState {
                ui_state: UIState {
                    views: vec![
                        (ViewId::DependencyView, Box::new(DependencyView::new())),
                        (
                            ViewId::DependencySearchView,
                            Box::new(DependencySearchView::new()),
                        ),
                    ],
                    currently_focused_view: ViewId::DependencyView,
                },
                data: Data {
                    mode: InteractionMode::Normal,
                    found_dependencies: Default::default(),
                    search_phrase: String::default(),
                    dependencies: Default::default(),
                    maven_file: Default::default(),
                    exit: false,
                },
            },
        };

        Ok(me)
    }

    fn init(&mut self) -> Result<()> {
        let maven_file = MavenFile::search_project_maven_file()?;
        let dependencies = maven_file
            .get_dependencies()
            .context("no dependencies found")?;
        self.state.data.dependencies = dependencies;
        self.spawn_input_task(self.tx.clone());
        return Ok(());
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.init()?;
        let mut effects: Vec<Effect> = vec![];

        while !self.state.data.exit {
            terminal.draw(|frame| UI::render(frame, &mut self.state.ui_state, &self.state.data))?;

            for effect in effects.drain(..) {
                tokio::spawn(AppAsyncOrchestrator::handle_async_event(
                    effect,
                    self.tx.clone(),
                ));
            }

            if let Some(event) = self.rx.recv().await {
                AppExecutor::handle_event(event, &mut self.state, &mut effects)
            }
        }

        Ok(())
    }

    fn spawn_input_task(&self, tx: mpsc::Sender<AppEvent>) {
        tokio::spawn(async move {
            loop {
                let event = match tokio::task::spawn_blocking(crossterm::event::read)
                    .await
                    .and_then(|r| Ok(r))
                {
                    Ok(Ok(ev)) => ev,
                    _ => {
                        eprintln!("Error reading event");
                        continue;
                    }
                };

                if let Err(e) = tx.send(AppEvent::Raw(event)).await {
                    eprintln!("Failed to send raw AppEvent: {:?}", e);
                }
            }
        });
    }
}
