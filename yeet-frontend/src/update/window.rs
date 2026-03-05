use crate::{
    error::AppError,
    model::{App, Window},
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn update(app: &mut App, area: Rect) -> Result<(), AppError> {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(100),
            Constraint::Length(1),
            Constraint::Length(u16::try_from(app.commandline.buffer.lines.len())?),
        ])
        .split(area);

    set_buffer_vp(&mut app.window, main[0])?;
    set_commandline_vp(&mut app.commandline, main[2])?;

    Ok(())
}

fn set_buffer_vp(window: &mut Window, area: Rect) -> Result<(), AppError> {
    match window {
        Window::Horizontal { first, second, .. } => {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(Constraint::from_ratios([(1, 2), (1, 2)]))
                .split(area);
            set_buffer_vp(first, layout[0])?;
            set_buffer_vp(second, layout[1])?;
        }
        Window::Directory(parent_vp, current_vp, preview_vp) => {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(Constraint::from_ratios([(1, 5), (2, 5), (2, 5)]))
                .split(area);

            let parent_rect = layout[0];
            let current_rect = layout[1];
            let preview_rect = layout[2];

            parent_vp.height = parent_rect.height;
            parent_vp.width = parent_rect.width;
            parent_vp.x = parent_rect.x;
            parent_vp.y = parent_rect.y;

            current_vp.height = current_rect.height;
            current_vp.width = current_rect.width;
            current_vp.x = current_rect.x;
            current_vp.y = current_rect.y;

            preview_vp.height = preview_rect.height;
            preview_vp.width = preview_rect.width;
            preview_vp.x = preview_rect.x;
            preview_vp.y = preview_rect.y;
        }
        Window::Tasks(vp) => {
            vp.height = area.height;
            vp.width = area.width;
            vp.x = area.x;
            vp.y = area.y;
        }
    }

    Ok(())
}

fn set_commandline_vp(
    commandline: &mut crate::model::CommandLine,
    rect: Rect,
) -> Result<(), AppError> {
    commandline.viewport.height = rect.height;

    let key_sequence_offset = u16::try_from(commandline.key_sequence.chars().count())?;
    commandline.viewport.width = rect.width.saturating_sub(key_sequence_offset);

    Ok(())
}

#[cfg(test)]
mod test {
    use yeet_buffer::model::viewport::ViewPort;

    use crate::model::{SplitFocus, Window};

    use super::*;

    #[test]
    fn set_buffer_vp_horizontal_splits_vertically() {
        let mut tree = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::Tasks(ViewPort::default())),
            focus: SplitFocus::First,
        };

        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 40,
        };

        set_buffer_vp(&mut tree, area).unwrap();

        match &tree {
            Window::Horizontal { first, second, .. } => {
                // first child should start at y=0
                match first.as_ref() {
                    Window::Directory(parent, current, preview) => {
                        assert_eq!(parent.y, 0);
                        assert_eq!(current.y, 0);
                        assert_eq!(preview.y, 0);
                        assert!(parent.height > 0);
                        assert!(current.height > 0);
                    }
                    _ => panic!("expected Directory"),
                }
                // second child should start below first
                match second.as_ref() {
                    Window::Tasks(vp) => {
                        assert!(vp.y > 0, "tasks viewport y should be > 0");
                        assert_eq!(vp.width, 80);
                        assert!(vp.height > 0);
                    }
                    _ => panic!("expected Tasks"),
                }
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn set_buffer_vp_tasks_sets_dimensions() {
        let mut window = Window::Tasks(ViewPort::default());
        let area = Rect {
            x: 5,
            y: 10,
            width: 60,
            height: 20,
        };

        set_buffer_vp(&mut window, area).unwrap();

        match &window {
            Window::Tasks(vp) => {
                assert_eq!(vp.x, 5);
                assert_eq!(vp.y, 10);
                assert_eq!(vp.width, 60);
                assert_eq!(vp.height, 20);
            }
            _ => panic!("expected Tasks"),
        }
    }
}
