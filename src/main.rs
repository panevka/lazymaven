mod debug;
mod dependency;

use color_eyre::Result;
use core::fmt;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use dependency::JavaDependencyNode;
use std::{collections::HashMap, fs, io};
use xmltree::{Element, Error};
// use ratatui::{DefaultTerminal, Frame};
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

use crate::dependency::MavenFile;

const TODO_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;

fn main() -> Result<(), Error> {
    let maven = MavenFile::from_file("./static/pom.xml".to_string())?;
    let dependencies = maven.get_dependencies();

    println!("{:#?}", dependencies);

    //
    let mut terminal = ratatui::init();
    let _ = App::default().run(&mut terminal);
    ratatui::restore();
    return Ok(());
}

pub struct DependencyList {
    items: Vec<JavaDependencyNode>,
    state: ListState,
}

impl Default for DependencyList {
    fn default() -> Self {
        Self {
            items: Default::default(),
            state: Default::default(),
        }
    }
}

impl fmt::Debug for DependencyList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DependencyList")
            .field("items", &self.items)
            .field("state", &self.state)
            .finish()
    }
}

// impl FromIterator<(Status, &'static str, &'static str)> for TodoList {
//     fn from_iter<I: IntoIterator<Item = (Status, &'static str, &'static str)>>(iter: I) -> Self {
//         let items = iter
//             .into_iter()
//             .map(|(status, todo, info)| TodoItem::new(status, todo, info))
//             .collect();
//         let state = ListState::default();
//         Self { items, state }
//     }
// }

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    // dependencies: Vec<HashMap<String, String>>,
    dependencies: DependencyList,
    exit: bool,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let maven = MavenFile::from_file("./static/pom.xml".to_string())?;
        let dependencies = maven.get_dependencies()?;
        self.dependencies.items = dependencies;

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
            _ => {}
        }
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

        // Paragraph::new(counter_text.clone())
        //     .centered()
        //     .block(block)
        //     .render(area, buf);
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
    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new().title(Line::raw("Dependencies").centered());

        let items: Vec<ListItem> = self
            .dependencies
            .items
            .iter()
            .enumerate()
            .map(|(i, dependency)| {
                let item =
                    String::from(format!("{} {}", &dependency.group_id, &dependency.version));

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
    }
}
