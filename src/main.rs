mod dependency;
mod maven_registry;
mod ui;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use dependency::JavaDependency;
use ratatui::{DefaultTerminal, widgets::ListState};
use std::io;
use xmltree::Error;

use crate::dependency::MavenFile;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let maven = MavenFile::from_file("./static/pom.xml".to_string())?;

    let mut terminal = ratatui::init();
    App::new(maven)?.run(&mut terminal)?;
    ratatui::restore();
    return Ok(());
}

pub struct DependencyList {
    items: Vec<JavaDependency>,
    state: ListState,
}

pub struct App {
    maven_file: MavenFile,
    dependencies: DependencyList,
    search_phrase: String,
    input_mode: bool,
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
            search_phrase: String::default(),
            dependencies: dependency_list,
            maven_file,
            input_mode: true,
            exit: false,
        };

        Ok(me)
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| ui::ui(frame, self))?;
            self.handle_events()?;
        }
        Ok(())
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
        if self.input_mode && !key_event.code.is_esc() {
            self.update_input(key_event);
            return ();
        }

        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('s') => self.input_mode = true,
            KeyCode::Esc => self.input_mode = false,
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

    fn exit(&mut self) {
        self.exit = true;
    }

    fn update_input(&mut self, key_event: KeyEvent) {
        self.search_phrase = String::from(format!(
            "{}{}",
            self.search_phrase,
            key_event.code.to_string()
        ));
    }
}

impl App {
    fn submit_dependency_changes(&mut self) -> Result<(), std::io::Error> {
        let result = self
            .maven_file
            .update_dependencies(&self.dependencies.items);
        return result;
    }

    fn select_next(&mut self) {
        self.dependencies.state.select_next();
    }

    fn select_previous(&mut self) {
        self.dependencies.state.select_previous();
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
