pub mod dependency_search_view;
pub mod dependency_view;

use ratatui::{buffer::Buffer, layout::Rect, prelude::Widget};

use crate::app::{Data, UIState};

pub enum ViewId {
    DependencyView,
    DependencySearchView,
}

pub trait View {
    fn render(&self, buffer: &mut Buffer, area: Rect, state: &Data, ui_state: &mut UIState);
}
