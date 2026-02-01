use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent};
use tokio::sync::mpsc;

use crate::{
    app::{AppState, InteractionMode},
    list::{List, Navigation},
    maven_registry::{MavenRegistry, MavenResponse, SearchResponseDoc},
};

pub struct EventContext<'a> {
    pub mode: InteractionMode,
    pub search_phrase: &'a String,
}

impl<'a> EventContext<'a> {
    pub fn from(app_state: &'a AppState) -> Self {
        Self {
            mode: app_state.mode,
            search_phrase: &app_state.search_phrase,
        }
    }
}

#[derive(Debug)]
pub enum AppEvent {
    Raw(Event),
    User(Intent),
    Async(AsyncEvent),
}

#[derive(Debug)]
pub enum Intent {
    Exit,
    EnterInputMode,
    LeaveInputMode,
    UpdateInput(KeyCode),
    SubmitDependencyChanges,
    DeleteSelectedDependency,
    NavigateDependencyList(Navigation),
    FindNewDependencies(String),
}

#[derive(Debug)]
pub enum AsyncEvent {
    MavenDependenciesFound(Vec<SearchResponseDoc>),
}

#[derive(Debug)]
pub enum Effect {
    SearchMaven(String),
}

pub struct AppIntentHandler {}

impl AppIntentHandler {
    pub fn event_to_intent(event: Event, ctx: EventContext) -> Option<Intent> {
        let intent = match event {
            Event::Key(key_event) => AppIntentHandler::handle_key_event(key_event, ctx),
            _ => return None,
        };

        return intent;
    }

    fn handle_key_event(key_event: KeyEvent, ctx: EventContext) -> Option<Intent> {
        if let InteractionMode::Input = ctx.mode {
            let intent = match key_event.code {
                KeyCode::Esc => Some(Intent::LeaveInputMode),
                _ => Some(Intent::UpdateInput(key_event.code)),
            };

            return intent;
        }

        let intent = match key_event.code {
            KeyCode::Char('q') => Intent::Exit,
            KeyCode::Char('i') => Intent::EnterInputMode,
            KeyCode::Char('j') => Intent::NavigateDependencyList(Navigation::Next),
            KeyCode::Char('k') => Intent::NavigateDependencyList(Navigation::Previous),
            KeyCode::Char('d') => Intent::DeleteSelectedDependency,
            KeyCode::Char('a') => Intent::SubmitDependencyChanges,
            KeyCode::Char('w') => Intent::FindNewDependencies(ctx.search_phrase.to_string()),
            _ => return None,
        };

        return Some(intent);
    }
}

pub struct AppExecutor {}

impl AppExecutor {
    pub fn handle_intent(event: AppEvent, state: &mut AppState, effects: &mut Vec<Effect>) {
        match event {
            AppEvent::User(Intent::Exit) => Self::exit_app(state),
            AppEvent::User(Intent::EnterInputMode) => {
                Self::enter_input_mode(state);
            }
            AppEvent::User(Intent::LeaveInputMode) => Self::leave_input_mode(state),
            AppEvent::User(Intent::DeleteSelectedDependency) => {
                Self::delete_selected_dependency(state)
            }
            AppEvent::User(Intent::UpdateInput(key_code)) => {
                Self::handle_input(&mut state.search_phrase, key_code);
            }
            AppEvent::User(Intent::SubmitDependencyChanges) => {
                Self::submit_dependency_changes(state);
            }
            AppEvent::User(Intent::NavigateDependencyList(direction)) => {
                Self::navigate_list(&mut state.dependencies, direction);
            }
            AppEvent::User(Intent::FindNewDependencies(search_phrase)) => {
                let effect = Self::find_new_dependencies(search_phrase);
                effects.push(effect);
            }
            AppEvent::Async(AsyncEvent::MavenDependenciesFound(dependencies)) => {
                state.found_dependencies.items = dependencies;
            }
            _ => (),
        };
    }

    fn exit_app(state: &mut AppState) {
        state.exit = true;
    }

    fn enter_input_mode(state: &mut AppState) {
        state.mode = InteractionMode::Input;
    }

    fn leave_input_mode(state: &mut AppState) {
        state.mode = InteractionMode::Normal;
    }

    fn submit_dependency_changes(state: &mut AppState) {
        state
            .maven_file
            .update_dependencies(&state.dependencies.items);

        // TODO Send event on success and / or on error.
    }

    fn navigate_list<T>(list: &mut List<T>, direction: Navigation) {
        list.navigate(direction);
    }

    fn handle_input(text: &mut String, key_code: KeyCode) {
        match key_code {
            KeyCode::Backspace => {
                text.pop();
            }
            KeyCode::Char(char) => text.push(char),
            _ => (),
        };
    }

    fn delete_selected_dependency(state: &mut AppState) {
        if let Some(index) = state.dependencies.state.selected() {
            state.dependencies.items.remove(index);

            let len = state.dependencies.items.len();
            if len == 0 {
                state.dependencies.state.select(None);
            } else if index >= len {
                state.dependencies.state.select(Some(len - 1));
            }
        }
    }

    fn find_new_dependencies(search_phrase: String) -> Effect {
        return Effect::SearchMaven(search_phrase);
    }
}

pub struct AppAsyncOrchestrator {}

impl AppAsyncOrchestrator {
    pub async fn handle_async_event(effect: Effect, tx: mpsc::Sender<AppEvent>) -> Result<()> {
        let result = match effect {
            Effect::SearchMaven(search_phrase) => Self::search_maven(search_phrase),
        };

        let response = result.await?.response.docs;

        let event = AppEvent::Async(AsyncEvent::MavenDependenciesFound(response));

        tx.send(event).await?;

        return Ok(());
    }

    fn search_maven(search_phrase: String) -> impl Future<Output = Result<MavenResponse>> {
        let result = MavenRegistry::search_dependencies(search_phrase);
        return result;
    }
}
