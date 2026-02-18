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
    versions_list_state: ListState,
    version_list_focused: bool,
    input_mode: bool,
    input: String,
}

impl DependencySearchView {
    pub fn new() -> Self {
        Self {
            list_state: Default::default(),
            versions_list_state: Default::default(),
            version_list_focused: false,
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

        let dependencies_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(layout[2]);

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

        StatefulWidget::render(list, dependencies_layout[0], buffer, &mut self.list_state);

        if let Some(index) = self.list_state.selected() {
            if let Some(currently_selected) = state.found_dependencies.get(index) {
                let id = format!("{}:{}", currently_selected.g, currently_selected.a);
                
                let mut sum = state.found_dependency_versions.get(&id);
                let default = &Vec::new();

                let versions = sum.get_or_insert(default);

                let items: Vec<ListItem> = versions 
                    .iter()
                    .enumerate()
                    .map(|(i, version)| {
                        let item = String::from(format!("{}", version.v));

                        let color = alternate_colors(i);
                        ListItem::new(item).bg(color)
                    })
                    .collect();

                let list = List::new(items)
                    .highlight_style(SELECTED_STYLE)
                    .highlight_symbol(">")
                    .highlight_spacing(HighlightSpacing::Always);

                StatefulWidget::render(list, dependencies_layout[1], buffer, &mut self.versions_list_state);
            }
        }


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

                return None;
            }

            let focused_list = match self.version_list_focused {
                true => &mut self.versions_list_state,
                false => &mut self.list_state,
            };

            match keycode {
                KeyCode::Char('i') => {
                    self.input_mode = true;
                }
                KeyCode::Char('s') => {
                    return Some(Intent::FindNewDependencies(self.input.to_string()));
                }
                KeyCode::Char('j') => focused_list.select_next(),
                KeyCode::Char('l') => self.version_list_focused = true,
                KeyCode::Char('h') => self.version_list_focused = false,
                KeyCode::Char('k') => focused_list.select_previous(),
                KeyCode::Enter => {
                    if let Some(index) = self.list_state.selected() {
                        return Some(Intent::GetAvailableDependencyVersions { index });
                    }
                }
                _ => ()
            }
        }

        return None;
    }
}
