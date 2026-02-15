pub mod dependency_search_view;
pub mod dependency_view;

use ratatui::{buffer::Buffer, layout::Rect, prelude::Widget};
use crossterm::event::KeyCode;

use crate::app::{Data, UIState};

#[derive(PartialEq, Clone, Debug)]
pub enum ViewId {
    DependencyView,
    DependencySearchView,
}

pub trait View {

    fn render(&self, buffer: &mut Buffer, area: Rect, state: &Data, ui_state: &mut UIState);

    fn handle_key(&mut self, keycode: KeyCode);
}
