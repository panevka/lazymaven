use ratatui::{
    buffer::Buffer,
    prelude::{Color, Modifier, Rect, Style, style::palette::tailwind::SLATE},
    style::Stylize,
    text::Line,
    widgets::{Block, HighlightSpacing, List, ListItem, StatefulWidget},
};
use crate::{views::View, app::{Data, UIState}, ui::alternate_colors};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

pub struct DependencyView;

impl View for DependencyView {

    fn render(&self, buffer: &mut Buffer, area: Rect, state: &Data, ui_state: &mut UIState) {
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

        StatefulWidget::render(list, area, buffer, &mut ui_state.dependency_list_state);
    }
}
