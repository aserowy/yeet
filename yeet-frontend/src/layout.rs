use ratatui::prelude::{Constraint, Direction, Layout, Rect};

#[derive(Clone, Debug)]
pub struct AppLayout {
    pub parent: Rect,
    pub current: Rect,
    pub preview: Rect,
    pub statusline: Rect,
    pub commandline: Rect,
}

impl AppLayout {
    pub fn default(rect: Rect, commandline_height: u16) -> Self {
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(100),
                Constraint::Length(1),
                Constraint::Length(commandline_height),
            ])
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
