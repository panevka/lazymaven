use crate::{App, dependency::JavaDependency, maven_registry::SearchResponseDoc};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Margin},
    style::{Color, Modifier, Style, Stylize, palette::tailwind::SLATE},
    text::{Line, Span},
    widgets::{Block, HighlightSpacing, List, ListItem},
};

const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = ratatui::layout::Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    let list = render_list(&app.dependencies.items);
    let search = render_search(&app.search_phrase);

    f.render_stateful_widget(list, chunks[0], &mut app.dependencies.state);
    f.render_widget(&search, chunks[1]);

    let search_results = chunks[1].inner(Margin {
        horizontal: 0,
        vertical: 10,
    });

    let deps = &mut app.found_dependencies;
    let found_dependencies = render_found_dependencies(&deps.items);

    f.render_stateful_widget(found_dependencies, search_results, &mut deps.state);
}

pub fn render_search<'a>(input_content: &'a String) -> Block<'a> {
    let block = Block::new().title(Line::raw(input_content).centered());
    return block;
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

pub fn render_found_dependencies(found_dependencies: &Vec<SearchResponseDoc>) -> List {
    let block = Block::new().title(Line::raw("Dependencies").centered());

    let items: Vec<ListItem> = found_dependencies
        .iter()
        .enumerate()
        .map(|(i, dependency)| {
            let item = String::from(format!("{}", dependency.id));

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

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}
