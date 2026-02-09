use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent};
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::{
    app::{AppState, UIState, InteractionMode},
    maven_registry::{MavenRegistry, MavenResponse, SearchResponseDoc},
    views::ViewId,
    ui::Navigation,
};

pub struct EventContext<'a> {
    pub mode: InteractionMode,
    pub search_phrase: &'a String,
}

impl<'a> EventContext<'a> {
    pub fn from(app_state: &'a AppState) -> Self {
        Self {
            mode: app_state.data.mode,
            search_phrase: &app_state.data.search_phrase,
        }
    }
}

#[derive(Debug)]
pub enum AppEvent {
    Raw(Event),
    User(Intent),
    Async(AsyncEvent),
}

#[derive(Debug, Clone)]
pub enum Intent {
    Exit,
    EnterInputMode,
    LeaveInputMode,
    UpdateInput(KeyCode),
    SubmitDependencyChanges,
    DeleteSelectedDependency,
    NavigateDependencyList(Navigation),
    FindNewDependencies(String),
    FocusNextView,
    FocusPreviousView
}

#[derive(Debug)]
pub enum AsyncEvent {
    MavenDependenciesFound(Vec<SearchResponseDoc>),
}

#[derive(Debug)]
pub enum Effect {
    SearchMaven(String),
}

trait IntentMapping {
    fn get_mapping(ctx: EventContext) -> HashMap<KeyCode, Intent>;
}

pub struct AppIntentHandler {}

impl IntentMapping for AppIntentHandler {
    fn get_mapping(ctx: EventContext) -> HashMap<KeyCode, Intent> {
        let default_mapping = HashMap::from([
            (KeyCode::Char('q'), Intent::Exit),
            (KeyCode::Char('i'), Intent::EnterInputMode),
            (
                KeyCode::Char('j'),
                Intent::NavigateDependencyList(Navigation::Next),
            ),
            (
                KeyCode::Char('k'),
                Intent::NavigateDependencyList(Navigation::Previous),
            ),
            (KeyCode::Char('d'), Intent::DeleteSelectedDependency),
            (KeyCode::Char('a'), Intent::SubmitDependencyChanges),
            (
                KeyCode::Char('w'),
                Intent::FindNewDependencies(ctx.search_phrase.to_string()),
            ),
            (KeyCode::Tab, Intent::FocusNextView),
            (KeyCode::BackTab, Intent::FocusPreviousView),
        ]);

        return default_mapping;
    }
}

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

        let mapping = Self::get_mapping(ctx);

        let intent = mapping.get(&key_event.code)?;

        return Some(intent.clone());
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
                Self::handle_input(&mut state.data.search_phrase, key_code);
            }
            AppEvent::User(Intent::SubmitDependencyChanges) => {
                Self::submit_dependency_changes(state);
            }
            AppEvent::User(Intent::NavigateDependencyList(direction)) => {
                Self::navigate_list(state, direction);
            }
            AppEvent::User(Intent::FindNewDependencies(search_phrase)) => {
                let effect = Self::find_new_dependencies(search_phrase);
                effects.push(effect);
            }
            AppEvent::Async(AsyncEvent::MavenDependenciesFound(dependencies)) => {
                state.data.found_dependencies = dependencies;
            }
            AppEvent::User(Intent::FocusNextView) => {
                Self::focus_next_view(state);
            }
            _ => (),
        };
    }

    fn exit_app(state: &mut AppState) {
        state.data.exit = true;
    }

    fn enter_input_mode(state: &mut AppState) {
        state.data.mode = InteractionMode::Input;
    }

    fn leave_input_mode(state: &mut AppState) {
        state.data.mode = InteractionMode::Normal;
    }

    fn submit_dependency_changes(state: &mut AppState) {
        state
            .data
            .maven_file
            .update_dependencies(&state.data.dependencies);

        // TODO Send event on success and / or on error.
    }

    fn navigate_list(state: &mut AppState, direction: Navigation) {
        match direction {
            Navigation::Next => state.ui_state.dependency_list_state.select_next(),
            Navigation::Previous => state.ui_state.dependency_list_state.select_previous()
        };
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
        if let Some(index) = state.ui_state.dependency_list_state.selected() {
            state.data.dependencies.remove(index);

            let len = state.data.dependencies.len();
            if len == 0 {
                state.ui_state.dependency_list_state.select(None);
            } else if index >= len {
                state.ui_state.dependency_list_state.select(Some(len - 1));
            }
        }
    }

    fn focus_next_view(state: &mut AppState) {
        // let keys: Vec<ViewId> = state.ui.views.keys().cloned().collect();

        // if let Some(current_index) = keys.iter().position(|v| *v == state.currently_focused_view) {
          //   let next_index = (current_index + 1) % keys.len();
            // state.currently_focused_view = keys[next_index].clone();
        //}
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
