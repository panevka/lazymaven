pub mod dependency_search_view;
pub mod dependency_view;

use ratatui::{buffer::Buffer, layout::Rect, prelude::Widget};
use crossterm::event::Event;

use crate::app::{Data, UIState};

#[derive(PartialEq, Clone, Debug)]
pub enum ViewId {
    DependencyView,
    DependencySearchView,
}

pub trait View {

    fn render(&mut self, buffer: &mut Buffer, area: Rect, state: &Data);

    fn handle_event(&mut self, event: &Event);
}
