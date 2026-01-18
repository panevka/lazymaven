mod debug;
mod dependency;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use dependency::JavaDependency;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Rect},
    style::{
        Color, Modifier, Style, Stylize,
        palette::{
            material::{BLUE, GREEN},
            tailwind::SLATE,
        },
    },
    symbols::border,
    text::{Line, Text},
    widgets::{
        Block, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget,
    },
};
use std::io;
use xmltree::Error;

use crate::dependency::MavenFile;

const TODO_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;

fn main() -> Result<(), Error> {
    let maven = MavenFile::from_file("./static/pom.xml".to_string())?;

    let mut terminal = ratatui::init();
    App::new(maven)?.run(&mut terminal);
    ratatui::restore();
    return Ok(());
}

pub struct DependencyList {
    items: Vec<JavaDependency>,
    state: ListState,
}

pub struct App {
    counter: u8,
    maven_file: MavenFile,
    dependencies: DependencyList,
    exit: bool,
}

impl App {
    fn new(maven_file: MavenFile) -> Result<Self, Error> {
        let dependencies = {
            let deps = maven_file.get_dependencies()?;
            deps
        };

        let dependency_list = DependencyList {
            items: dependencies,
            state: ListState::default(),
        };

        let me = Self {
            counter: 0,
            dependencies: dependency_list,
            maven_file: maven_file,
            exit: false,
        };

        Ok(me)
    }

    fn submit_dependency_changes(&mut self) -> Result<(), std::io::Error> {
        let result = self
            .maven_file
            .update_dependencies(&self.dependencies.items);
        return result;
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50)])
            .split(frame.area());

        frame.render_widget(&mut *self, chunks[0]);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            KeyCode::Char('j') => self.select_next(),
            KeyCode::Char('k') => self.select_previous(),
            KeyCode::Char('d') => self.delete_selected_dependency(),
            KeyCode::Char('a') => {
                self.submit_dependency_changes();
                return ();
            }
            _ => {}
        }

        return ();
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn get_selected_dependency(&self) -> Option<&JavaDependency> {
        let selected_item_index = self.dependencies.state.selected()?;
        let selected_item = &self.dependencies.items[selected_item_index];

        return Some(selected_item);
    }

    fn select_none(&mut self) {
        self.dependencies.state.select(None);
    }

    fn select_next(&mut self) {
        self.dependencies.state.select_next();
    }
    fn select_previous(&mut self) {
        self.dependencies.state.select_previous();
    }

    fn select_first(&mut self) {
        self.dependencies.state.select_first();
    }

    fn select_last(&mut self) {
        self.dependencies.state.select_last();
    }

    fn delete_selected_dependency(&mut self) {
        if let Some(index) = self.dependencies.state.selected() {
            self.dependencies.items.remove(index);

            let len = self.dependencies.items.len();
            if len == 0 {
                self.dependencies.state.select(None);
            } else if index >= len {
                self.dependencies.state.select(Some(len - 1));
            }
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.clone().centered())
            .title_bottom(instructions.clone().centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        self.render_list(area, buf);
    }
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

impl App {
    fn render_list(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let block = Block::new().title(Line::raw("Dependencies").centered());

        let items: Vec<ListItem> = self
            .dependencies
            .items
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

        StatefulWidget::render(list, area, buf, &mut self.dependencies.state);
        Ok(())
    }
}
