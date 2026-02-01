use ratatui::widgets::ListState;

#[derive(Debug)]
pub enum Navigation {
    Previous,
    Next,
}

#[derive(Default, Clone)]
pub struct List<T> {
    pub items: Vec<T>,
    pub state: ListState,
}

impl<T> List<T> {
    pub fn navigate(&mut self, direction: Navigation) {
        match direction {
            Navigation::Next => self.state.select_next(),
            Navigation::Previous => self.state.select_previous(),
        }
    }
}
