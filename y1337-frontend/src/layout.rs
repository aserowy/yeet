use ratatui::prelude::{Constraint, Direction, Layout, Rect};

#[derive(Debug)]
pub struct AppLayout {
    pub parent: Rect,
    pub current: Rect,
    pub preview: Rect,
    pub statusline: Rect,
    pub commandline: Rect,
}

impl AppLayout {
    pub fn default(rect: Rect) -> Self {
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Max(1), Constraint::Max(1)])
            .split(rect);

        let files = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(Constraint::from_ratios([(1, 5), (2, 5), (2, 5)]))
            .split(main[0]);

        Self {
            parent: files[0],
            current: files[1],
            preview: files[2],
            statusline: main[1],
            commandline: main[2],
        }
    }
}