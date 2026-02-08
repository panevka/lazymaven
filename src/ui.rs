use crate::{
    app::AppState,
    views::{ViewId, Views},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction},
    style::{Color, palette::tailwind::SLATE},
};

const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;

pub fn ui(f: &mut Frame, app_state: &mut AppState, views: &Views) {
    let chunks = ratatui::layout::Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    let mut buffer = f.buffer_mut();

    views.render(ViewId::DependencyView, &mut buffer, chunks[0], app_state);
    views.render(
        ViewId::DependencySearchView,
        &mut buffer,
        chunks[1],
        app_state,
    );
}

pub fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}
