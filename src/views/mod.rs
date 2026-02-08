pub mod dependency_search_view;
pub mod dependency_view;

use dependency_search_view::DependencySearchView;
use dependency_view::DependencyView;

use ratatui::{buffer::Buffer, layout::Rect, prelude::Widget};

use crate::app::AppState;

pub enum ViewId {
    DependencyView,
    DependencySearchView,
}

pub struct Views {
    dependency: DependencyView,
    dependency_search_view: DependencySearchView,
}

impl Views {
    pub fn new() -> Self {
        Self {
            dependency: DependencyView {},
            dependency_search_view: DependencySearchView {},
        }
    }

    pub fn render(&self, view_id: ViewId, buffer: &mut Buffer, area: Rect, state: &mut AppState) {
        match view_id {
            ViewId::DependencyView => self
                .dependency
                .render(buffer, area, &mut state.dependencies),
            ViewId::DependencySearchView => self.dependency_search_view.render(buffer, area, state),
        }
    }
}

pub trait View: Widget {}
