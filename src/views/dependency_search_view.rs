use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize, palette::tailwind::SLATE},
    text::Line,
    widgets::{Block, HighlightSpacing, List, ListItem, StatefulWidget, Widget},
};

use crate::{app::AppState, ui::alternate_colors};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

pub struct DependencySearchView {}

impl DependencySearchView {
    pub fn render(&self, buffer: &mut Buffer, area: Rect, state: &mut AppState) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(50),
            ])
            .split(area);

        let block = Block::new().title(Line::raw("Search Dependencies").centered());
        block.render(layout[0], buffer);

        let text_input = Block::new().title(Line::raw(&state.search_phrase).centered());
        text_input.render(layout[1], buffer);

        let items: Vec<ListItem> = state
            .found_dependencies
            .items
            .iter()
            .enumerate()
            .map(|(i, dependency)| {
                let item = String::from(format!("{}", dependency.id));

                let color = alternate_colors(i);
                ListItem::new(item).bg(color)
            })
            .collect();

        let list = List::new(items)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, layout[2], buffer, &mut state.found_dependencies.state);
    }
}
