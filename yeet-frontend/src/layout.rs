use ratatui::prelude::{Constraint, Direction, Layout, Rect};

#[derive(Clone)]
pub struct AppLayout {
    // TODO: split layout to enable file buffer related layout
    pub parent: Rect,
    pub current: Rect,
    pub preview: Rect,
    pub statusline: Rect,
    pub commandline: Rect,
}

impl AppLayout {
    pub fn new(rect: Rect, commandline_height: u16) -> Self {
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

impl Default for AppLayout {
    fn default() -> Self {
        AppLayout::new(Rect::default(), 0)
    }
}

#[derive(Clone)]
pub struct CommandLineLayout {
    pub buffer: Rect,
    pub key_sequence: Rect,
}

impl CommandLineLayout {
    pub fn new(rect: Rect, key_sequence_length: u16) -> Self {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(100),
                Constraint::Length(key_sequence_length),
            ])
            .split(rect);

        Self {
            buffer: layout[0],
            key_sequence: layout[1],
        }
    }
}
