use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize, palette::tailwind::SLATE},
    text::Line,
    widgets::{ListState, Block, HighlightSpacing, List, ListItem, StatefulWidget, Widget},
};

use crossterm::event::{KeyCode, Event};

use crate::{
    app::{Data, UIState}, 
    ui::alternate_colors, 
    views::View,
    events::Intent
};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

pub struct DependencySearchView {
    list_state: ListState,
    input_mode: bool,
    input: String,
}

impl DependencySearchView {
    pub fn new() -> Self {
        Self {
            list_state: Default::default(),
            input_mode: false,
            input: Default::default(),
        }
    }
}

impl View for DependencySearchView {

    fn render(&mut self, buffer: &mut Buffer, area: Rect, state: &Data) {
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

        let text_input = Block::new().title(Line::raw(&self.input).centered());
        text_input.render(layout[1], buffer);

        let items: Vec<ListItem> = state
            .found_dependencies
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

        StatefulWidget::render(list, layout[2], buffer, &mut self.list_state);
    }

    fn handle_event(&mut self, event: &Event) -> Option<Intent> {
        if let Event::Key(key_event) = event {
            let keycode = key_event.code;

            if self.input_mode {
                match keycode {
                    KeyCode::Esc => {
                        self.input_mode = false;
                    }
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    KeyCode::Char(char) => self.input.push(char),
                    _ => (),
                };
            }

            match keycode {
                KeyCode::Char('i') => {
                    self.input_mode = true;
                }
                KeyCode::Char('s') => {
                    return Some(Intent::FindNewDependencies(self.input.to_string()));
                }
                KeyCode::Char('j') => self.list_state.select_next(),
                KeyCode::Char('k') => self.list_state.select_previous(),
                _ => ()
            }
        }

        return None;
    }
}
