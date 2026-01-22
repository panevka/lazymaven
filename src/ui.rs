use ratatui::{
    Frame,
    layout::{Constraint, Direction},
    style::{
        Color, Modifier, Style, Stylize,
        palette::{
            material::{BLUE, GREEN},
            tailwind::SLATE,
        },
    },
    text::{Line, Span, Text},
    widgets::{self, Block, HighlightSpacing, List, ListItem},
};
use tui_textarea::TextArea;

use crate::{
    App,
    dependency::JavaDependency,
    maven_registry::{MavenRegistry, MavenResponse, SearchResponseDoc},
};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = ratatui::layout::Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    let list = render_list(&app.dependencies.items);

    let search = render_search(&app.search_phrase);

    f.render_stateful_widget(list, chunks[0], &mut app.dependencies.state);

    f.render_widget(&search, chunks[1]);
}

pub fn render_search<'a>(input_content: &'a String) -> Span<'a> {
    let span = Span::raw(input_content);
    return span;
}

const TODO_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

pub fn render_list(dependencies: &Vec<JavaDependency>) -> List {
    let block = Block::new().title(Line::raw("Dependencies").centered());

    let items: Vec<ListItem> = dependencies
        .iter()
        .enumerate()
        .map(|(i, dependency)| {
            let item = String::from(format!("{} {}", dependency.group_id, dependency.version));

            let color = alternate_colors(i);
            ListItem::new(item).bg(color)
        })
        .collect();

    List::new(items)
        .block(block)
        .highlight_style(SELECTED_STYLE)
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always)
}
