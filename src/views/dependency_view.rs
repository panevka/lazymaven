use ratatui::{
    buffer::Buffer,
    prelude::{Color, Modifier, Rect, Style, style::palette::tailwind::SLATE},
    style::Stylize,
    text::Line,
    widgets::{ListState, Block, HighlightSpacing, List, ListItem, StatefulWidget},
};
use crate::{views::View, app::{Data, UIState}, ui::alternate_colors};

use crossterm::event::{Event, KeyCode};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

pub struct DependencyView {
    list_state: ListState
}

impl DependencyView {
    pub fn new () -> Self {
        Self {
            list_state: Default::default()
        }
    }
}

impl View for DependencyView {

    fn render(&mut self, buffer: &mut Buffer, area: Rect, state: &Data) {
        let block = Block::new().title(Line::raw("Dependencies").centered());

        let items: Vec<ListItem> = state
            .dependencies
            .iter()
            .enumerate()
            .map(|(i, dependency)| {
                let item = String::from(format!("{} {}", dependency.group_id, dependency.version));

                let color = alternate_colors(i);
                ListItem::new(item).bg(color)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buffer, &mut self.list_state);
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key_event) = event {
            let keycode = key_event.code;

            match keycode {
                KeyCode::Char('j') => self.list_state.select_next(),
                KeyCode::Char('k') => self.list_state.select_previous(),
                _ => ()
            }
        }
    }
}
