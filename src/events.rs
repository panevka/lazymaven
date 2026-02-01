use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::{
    app::{AppState, InteractionMode, Navigation},
    maven_registry::SearchResponseDoc,
};

pub struct EventContext {
    pub mode: InteractionMode,
}

impl EventContext {
    pub fn from(app_state: &AppState) -> Self {
        Self {
            mode: app_state.mode,
        }
    }
}

#[derive(Debug)]
pub enum AppEvent {
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
    MavenSearchSucceeded(Vec<SearchResponseDoc>),
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

    pub fn spawn_input_handler() {}

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
            _ => return None,
        };

        return Some(intent);
    }
}

pub struct AppExecutor {}

impl AppExecutor {
    pub fn handle_intent(event: AppEvent, state: &mut AppState) {
        match event {
            AppEvent::User(Intent::Exit) => Self::exit_app(state),
            AppEvent::User(Intent::EnterInputMode) => Self::enter_input_mode(state),
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
                Self::navigate_dependency_list(state, direction);
            }
            AppEvent::User(Intent::FindNewDependencies(search_phrase)) => {
                // TODO handle find new dependenices event without spawning within the app executor
                // Self::find_new_dependencies(search_phrase);
            }
            _ => state.exit = true,
        }
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

    fn navigate_dependency_list(state: &mut AppState, direction: Navigation) {
        match direction {
            Navigation::Next => state.dependencies.state.select_next(),
            Navigation::Previous => state.dependencies.state.select_previous(),
        }
    }

    fn handle_input(text: &mut String, key_code: KeyCode) {
        let mut updated_text = text.clone();

        if let Some(char) = key_code.as_char() {
            updated_text = String::from(format!("{}{}", updated_text, char));
        } else if key_code.is_backspace() {
            updated_text.pop();
        }

        *text = updated_text;
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

    // fn find_new_dependencies(&self, search_phrase: String) {
    //     let tx = self.tx.clone();

    //     tokio::spawn({
    //         async move {
    //             let found = MavenRegistry::search_dependencies(search_phrase).await?;
    //             tx.send(AppEvent::DependenciesFound(found.response.docs))
    //                 .await?;
    //             return Ok::<(), anyhow::Error>(());
    //         }
    //     });

    //     return ();
    // }
}
