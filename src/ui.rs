use crate::{
    app::{UIState, Data},
    views::{View, ViewId, dependency_view::DependencyView, dependency_search_view::DependencySearchView}
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction},
    style::{Color, palette::tailwind::SLATE},
};
use std::collections::BTreeMap;

const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;

pub struct UI;

impl UI {

    pub fn render(f: &mut Frame, ui_state: &mut UIState, app_state: &Data) {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(f.area());

        let views: &Vec<(ViewId, Box<dyn View>)> = &app_state.views;

        let mut buffer = f.buffer_mut();


        for (view_id, view) in views.iter() {
            match view_id {
                ViewId::DependencyView => view.render(buffer, chunks[0], app_state, ui_state),
                ViewId::DependencySearchView => view.render(buffer, chunks[1], app_state, ui_state),
            }
        }

    }
}

pub fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

#[derive(Debug, Clone)]
pub enum Navigation {
    Next,
    Previous,
}
