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
    pub currently_focused_view: &'a ViewId
}

impl<'a> EventContext<'a> {
    pub fn from(app_state: &'a AppState) -> Self {
        Self {
            mode: app_state.data.mode,
            currently_focused_view: &app_state.ui_state.currently_focused_view,
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
    SubmitDependencyChanges,
    DeleteSelectedDependency { index: usize },
    FindNewDependencies(String),
    FocusNextView,
    FocusPreviousView,
    HandleViewMapping(ViewId, Event)
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
            (KeyCode::Char('a'), Intent::SubmitDependencyChanges),
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
        let mapping = Self::get_mapping(ctx);

        let intent = mapping.get(&key_event.code)?;

        return Some(intent.clone());
    }
}

pub struct AppExecutor {}

impl AppExecutor {

    pub fn handle_event(event: AppEvent, state: &mut AppState, effects: &mut Vec<Effect>) {

        if let AppEvent::Raw(raw_event) = event {
            AppExecutor::handle_event_for_focused_view(&raw_event, state).
                map(|intent| 
                    AppExecutor::execute(AppEvent::User(intent), state, effects));

            let ctx = EventContext::from(&state);
            AppIntentHandler::event_to_intent(raw_event, ctx).
                map(|intent| 
                    AppExecutor::execute(AppEvent::User(intent), state, effects));
        } else {
            AppExecutor::execute(event, state, effects);
        }

    }

    pub fn execute(event: AppEvent, state: &mut AppState, effects: &mut Vec<Effect>) {

        match event {
            AppEvent::User(Intent::Exit) => Self::exit_app(state),
            AppEvent::User(Intent::DeleteSelectedDependency { index }) => {
                Self::delete_selected_dependency(index, state)
            }
            AppEvent::User(Intent::SubmitDependencyChanges) => {
                Self::submit_dependency_changes(state);
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

    fn handle_event_for_focused_view(event: &Event, state: &mut AppState) -> Option<Intent> {
        let focused = &state.ui_state.currently_focused_view;
        let views = &mut state.ui_state.views;

        if let Some((view_id, view)) = views.iter_mut().find(|(view_id, view)| *view_id == *focused) {
            return view.handle_event(&event);
        }

        return None;
    }

    fn exit_app(state: &mut AppState) {
        state.data.exit = true;
    }

    fn submit_dependency_changes(state: &mut AppState) {
        state
            .data
            .maven_file
            .update_dependencies(&state.data.dependencies);

        // TODO Send event on success and / or on error.
    }

    fn delete_selected_dependency(index: usize, state: &mut AppState) {
         state.data.dependencies.remove(index);
    }

    fn focus_next_view(state: &mut AppState) {
        let views = &state.ui_state.views;

        if let Some(current_index) = views.iter().position(|(v, _)| *v == state.ui_state.currently_focused_view) {
            let next_index = (current_index + 1) % views.len();
            state.ui_state.currently_focused_view = views[next_index].0.clone();
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
