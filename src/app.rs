use anyhow::{Context, Result};
use dependency::JavaDependency;
use maven_registry::SearchResponseDoc;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;

use crate::{
    dependency::{self, MavenFile},
    events::{
        self, AppAsyncOrchestrator, AppEvent, AppExecutor, AppIntentHandler, Effect, EventContext,
    },
    list::List,
    maven_registry, ui,
    views::Views,
};

pub struct App {
    tx: mpsc::Sender<events::AppEvent>,
    rx: mpsc::Receiver<events::AppEvent>,
    state: AppState,
    views: Views,
}

#[derive(Clone)]
pub struct AppState {
    pub maven_file: MavenFile,
    pub mode: InteractionMode,
    pub exit: bool,
    pub search_phrase: String,
    pub found_dependencies: List<SearchResponseDoc>,
    pub dependencies: List<JavaDependency>,
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
            views: Views::new(),
            state: AppState {
                found_dependencies: Default::default(),
                search_phrase: String::default(),
                dependencies: Default::default(),
                maven_file: Default::default(),
                mode: InteractionMode::Normal,
                exit: false,
            },
        };

        Ok(me)
    }

    fn init(&mut self) -> Result<()> {
        let maven_file = MavenFile::search_project_maven_file()?;
        let dependencies = maven_file
            .get_dependencies()
            .context("no dependencies found")?;
        self.state.dependencies.items = dependencies;
        self.state.dependencies.state.select_first();
        self.spawn_input_task(self.tx.clone());
        return Ok(());
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.init()?;
        let mut effects: Vec<Effect> = vec![];

        while !self.state.exit {
            terminal.draw(|frame| ui::ui(frame, &mut self.state, &self.views))?;

            for effect in effects.drain(..) {
                tokio::spawn(AppAsyncOrchestrator::handle_async_event(
                    effect,
                    self.tx.clone(),
                ));
            }

            if let Some(event) = self.rx.recv().await {
                match event {
                    AppEvent::Raw(ev) => {
                        let ctx = EventContext::from(&self.state);
                        if let Some(intent) = AppIntentHandler::event_to_intent(ev, ctx) {
                            AppExecutor::handle_intent(
                                AppEvent::User(intent),
                                &mut self.state,
                                &mut effects,
                            );
                        }
                    }
                    other => {
                        AppExecutor::handle_intent(other, &mut self.state, &mut effects);
                    }
                }
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
