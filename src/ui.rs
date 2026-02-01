use crate::{app::AppState, dependency::JavaDependency, maven_registry::SearchResponseDoc, App};
use ratatui::{
    layout::{Constraint, Direction, Margin},
    style::{palette::tailwind::SLATE, Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, HighlightSpacing, List, ListItem},
    Frame,
};

const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

pub fn ui(f: &mut Frame, app_state: &mut AppState) {
    let chunks = ratatui::layout::Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    let list = render_list(&app_state.dependencies.items);
    let search = render_search(&app_state.search_phrase);

    f.render_stateful_widget(list, chunks[0], &mut app_state.dependencies.state);
    f.render_widget(&search, chunks[1]);

    let search_results = chunks[1].inner(Margin {
        horizontal: 0,
        vertical: 10,
    });

    let deps = &mut app_state.found_dependencies;
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
