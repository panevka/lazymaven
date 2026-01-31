mod dependency;
mod events;
mod maven_registry;
mod ui;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use dependency::JavaDependency;
use maven_registry::{MavenRegistry, SearchResponseDoc};
use ratatui::{DefaultTerminal, widgets::ListState};
use tokio::{sync::mpsc, task};
use xmltree::Error;

use crate::dependency::MavenFile;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let maven = MavenFile::from_file("./static/pom.xml".to_string())?;

    let mut terminal = ratatui::init();
    App::new(maven)?.run(&mut terminal).await?;
    ratatui::restore();
    return Ok(());
}

pub struct DependencyList {
    items: Vec<JavaDependency>,
    state: ListState,
}

#[derive(Default)]
pub struct FoundDependencyList {
    items: Vec<SearchResponseDoc>,
    state: ListState,
}

pub struct App {
    intent_tx: mpsc::Sender<Effects>,
    intent_rx: mpsc::Receiver<Effects>,
    maven_file: MavenFile,
    dependencies: DependencyList,
    search_phrase: String,
    input_mode: bool,
    pub found_dependencies: FoundDependencyList,
    exit: bool,
}

pub enum Navigation {
    Previous,
    Next,
}

pub enum Intent {
    Exit,
    EnterInputMode,
    LeaveInputMode,
    SubmitDependencyChanges,
    DeleteSelectedDependency,
    UpdateInput(KeyCode),
    NavigateDependencyList(Navigation),
    FindNewDependencies(String),
}

pub enum Effects {
    DependenciesFound(Vec<SearchResponseDoc>),
}

// Core app logic
impl App {
    fn new(maven_file: MavenFile) -> Result<Self, Error> {
        let dependencies = {
            let deps = maven_file.get_dependencies()?;
            deps
        };

        let dependency_list = DependencyList {
            items: dependencies,
            state: ListState::default(),
        };

        let (tx, rx) = mpsc::channel::<Effects>(100);

        let me = Self {
            intent_tx: tx,
            intent_rx: rx,
            found_dependencies: Default::default(),
            search_phrase: String::default(),
            dependencies: dependency_list,
            maven_file,
            input_mode: false,
            exit: false,
        };

        Ok(me)
    }

    async fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.exit {
            terminal.draw(|frame| ui::ui(frame, self))?;

            tokio::select! {
                Some(intent_result) = self.intent_rx.recv() => {
                    self.handle_intent_result(intent_result).await;
                }
                Ok(event) = App::read_key_async() => {
                    if let Some(intent) = self.handle_event(event){
                       self.apply_intent(intent)?;
                    }
                }

            }
        }
        Ok(())
    }

    async fn read_key_async() -> Result<crossterm::event::Event, std::io::Error> {
        task::spawn_blocking(|| event::read())
            .await
            .expect("task panicked")
    }

    fn handle_event(&mut self, event: Event) -> Option<Intent> {
        // it's important to check that the event is a key press event as
        // crossterm also emits key release and repeat events on Windows.
        if let Some(key_event) = event.as_key_press_event() {
            let intent: Option<Intent> = self.handle_key_event(key_event);
            return intent;
        }

        return None;
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<Intent> {
        if self.input_mode {
            let intent = match key_event.code {
                KeyCode::Esc => Some(Intent::LeaveInputMode), // allow leaving input mode
                _ => Some(Intent::UpdateInput(key_event.code)), // all other keys update input
            };

            return intent;
        }

        let intent = match key_event.code {
            KeyCode::Char('q') => Some(Intent::Exit),
            KeyCode::Char('i') => Some(Intent::EnterInputMode),
            KeyCode::Char('j') => Some(Intent::NavigateDependencyList(Navigation::Next)),
            KeyCode::Char('k') => Some(Intent::NavigateDependencyList(Navigation::Previous)),
            KeyCode::Char('d') => Some(Intent::DeleteSelectedDependency),
            KeyCode::Char('a') => Some(Intent::SubmitDependencyChanges),
            KeyCode::Char('w') => Some(Intent::FindNewDependencies(self.search_phrase.clone())),
            KeyCode::Char(_) if self.input_mode => Some(Intent::UpdateInput(key_event.code)),
            _ => None,
        };

        return intent;
    }
}

// Event management
impl App {
    pub fn apply_intent(&mut self, intent: Intent) -> anyhow::Result<()> {
        match intent {
            Intent::Exit => self.exit_app(),
            Intent::EnterInputMode => self.enter_input_mode(),
            Intent::LeaveInputMode => self.leave_input_mode(),
            Intent::NavigateDependencyList(direction) => self.navigate_dependency_list(direction),
            Intent::SubmitDependencyChanges => self.submit_dependency_changes()?,
            Intent::DeleteSelectedDependency => self.delete_selected_dependency(),
            Intent::UpdateInput(key_code) => self.update_input(key_code),
            Intent::FindNewDependencies(search_phrase) => self.find_new_dependencies(search_phrase),
        }

        return Ok(());
    }

    async fn handle_intent_result(&mut self, intent_result: Effects) {
        match intent_result {
            Effects::DependenciesFound(search_response_docs) => {
                self.found_dependencies.items = search_response_docs
            }
        };
    }

    fn exit_app(&mut self) {
        self.exit = true;
    }
    fn enter_input_mode(&mut self) {
        self.input_mode = true;
    }

    fn leave_input_mode(&mut self) {
        self.input_mode = false;
    }

    fn submit_dependency_changes(&mut self) -> Result<(), std::io::Error> {
        let result = self
            .maven_file
            .update_dependencies(&self.dependencies.items);
        return result;
    }

    fn navigate_dependency_list(&mut self, direction: Navigation) {
        match direction {
            Navigation::Next => self.dependencies.state.select_next(),
            Navigation::Previous => self.dependencies.state.select_previous(),
        }
    }

    fn update_input(&mut self, key_code: KeyCode) {
        if let Some(char) = key_code.as_char() {
            self.search_phrase = String::from(format!("{}{}", self.search_phrase, char));
        }

        if key_code.is_backspace() {
            self.search_phrase.pop();
        }
    }

    fn delete_selected_dependency(&mut self) {
        if let Some(index) = self.dependencies.state.selected() {
            self.dependencies.items.remove(index);

            let len = self.dependencies.items.len();
            if len == 0 {
                self.dependencies.state.select(None);
            } else if index >= len {
                self.dependencies.state.select(Some(len - 1));
            }
        }
    }

    fn find_new_dependencies(&self, search_phrase: String) {
        let tx = self.intent_tx.clone();

        tokio::spawn({
            async move {
                let found = MavenRegistry::search_dependencies(search_phrase).await?;
                tx.send(Effects::DependenciesFound(found.response.docs))
                    .await?;
                return Ok::<(), anyhow::Error>(());
            }
        });

        return ();
    }
}
