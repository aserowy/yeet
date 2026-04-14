use crate::{
    error::AppError,
    model::{App, Window},
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn update(app: &mut App, area: Rect) -> Result<(), AppError> {
    let mut contraints = vec![
        Constraint::Percentage(100),
        Constraint::Length(u16::try_from(app.commandline.buffer.lines.len())?),
    ];

    let mut index_offset = 0;
    if app.tabs.len() > 1 {
        contraints.insert(0, Constraint::Length(1));
        index_offset = 1;
    }

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints(contraints)
        .split(area);

    let window = app.current_window_mut()?;
    set_buffer_vp(window, main[index_offset])?;
    set_commandline_vp(&mut app.commandline, main[1 + index_offset])?;

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
        Window::Vertical { first, second, .. } => {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(Constraint::from_ratios([(1, 2), (1, 2)]))
                .split(area);
            set_buffer_vp(first, layout[0])?;
            set_buffer_vp(second, layout[1])?;
        }
        // NOTE: the -1 for height is to account for the statusline at the bottom of each pane
        Window::Directory(parent_vp, current_vp, preview_vp) => {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(Constraint::from_ratios([(1, 5), (2, 5), (2, 5)]))
                .split(area);

            let parent_rect = layout[0];
            let current_rect = layout[1];
            let preview_rect = layout[2];

            parent_vp.height = parent_rect.height.saturating_sub(1);
            parent_vp.width = parent_rect.width;
            parent_vp.x = parent_rect.x;
            parent_vp.y = parent_rect.y;

            current_vp.height = current_rect.height.saturating_sub(1);
            current_vp.width = current_rect.width;
            current_vp.x = current_rect.x;
            current_vp.y = current_rect.y;

            preview_vp.height = preview_rect.height.saturating_sub(1);
            preview_vp.width = preview_rect.width;
            preview_vp.x = preview_rect.x;
            preview_vp.y = preview_rect.y;
        }
        // NOTE: the -1 for height is to account for the statusline at the bottom of each pane
        Window::Tasks(vp) | Window::QuickFix(vp) | Window::Help(vp) => {
            vp.height = area.height.saturating_sub(1);
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
    commandline.viewport.x = rect.x;
    commandline.viewport.y = rect.y;
    commandline.viewport.height = rect.height;

    let key_sequence_offset = u16::try_from(commandline.key_sequence.chars().count())?;
    commandline.viewport.width = rect.width.saturating_sub(key_sequence_offset);

    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use yeet_buffer::model::viewport::ViewPort;

    use crate::model::{Buffer, CommandLine, Contents, SplitFocus, Window};

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
    fn set_buffer_vp_all_viewports_nonzero_after_layout() {
        let mut tree = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::Tasks(ViewPort::default())),
            focus: SplitFocus::Second,
        };

        let area = Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 50,
        };

        set_buffer_vp(&mut tree, area).unwrap();

        match &tree {
            Window::Horizontal { first, second, .. } => {
                match first.as_ref() {
                    Window::Directory(parent, current, preview) => {
                        assert!(parent.width > 0 && parent.height > 0, "parent non-zero");
                        assert!(current.width > 0 && current.height > 0, "current non-zero");
                        assert!(preview.width > 0 && preview.height > 0, "preview non-zero");
                    }
                    _ => panic!("expected Directory"),
                }
                match second.as_ref() {
                    Window::Tasks(vp) => {
                        assert!(vp.width > 0 && vp.height > 0, "tasks non-zero");
                    }
                    _ => panic!("expected Tasks"),
                }
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn set_buffer_vp_tasks_below_directory() {
        let mut tree = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::Tasks(ViewPort::default())),
            focus: SplitFocus::Second,
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
                let dir_y = match first.as_ref() {
                    Window::Directory(_, current, _) => current.y,
                    _ => panic!("expected Directory"),
                };
                let task_y = match second.as_ref() {
                    Window::Tasks(vp) => vp.y,
                    _ => panic!("expected Tasks"),
                };
                assert!(
                    task_y > dir_y,
                    "task viewport y ({task_y}) should be below directory y ({dir_y})"
                );
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
                assert_eq!(
                    vp.height, 19,
                    "height should be area.height - 1 for statusline"
                );
            }
            _ => panic!("expected Tasks"),
        }
    }

    #[test]
    fn set_buffer_vp_vertical_splits_horizontally() {
        let mut tree = Window::Vertical {
            first: Box::new(Window::Tasks(ViewPort::default())),
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
            Window::Vertical { first, second, .. } => match (first.as_ref(), second.as_ref()) {
                (Window::Tasks(left), Window::Tasks(right)) => {
                    assert_eq!(left.x, 0);
                    assert!(right.x > 0, "right pane x should be > 0");
                    assert_eq!(left.y, 0);
                    assert_eq!(right.y, 0);
                    assert!(left.width > 0);
                    assert!(right.width > 0);
                    assert_eq!(left.width + right.width, 80);
                }
                _ => panic!("expected Tasks"),
            },
            _ => panic!("expected Vertical"),
        }
    }

    #[test]
    fn set_buffer_vp_vertical_children_same_y() {
        let mut tree = Window::Vertical {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            focus: SplitFocus::First,
        };

        let area = Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 50,
        };

        set_buffer_vp(&mut tree, area).unwrap();

        match &tree {
            Window::Vertical { first, second, .. } => match (first.as_ref(), second.as_ref()) {
                (Window::Directory(lp, lc, lprev), Window::Directory(rp, rc, rprev)) => {
                    assert_eq!(lc.y, rc.y, "both directories should start at same y");
                    assert!(rp.x > lprev.x, "right parent x should be > left preview x");

                    for vp in &[lp, lc, lprev, rp, rc, rprev] {
                        assert!(vp.width > 0 && vp.height > 0);
                    }
                }
                _ => panic!("expected Directory"),
            },
            _ => panic!("expected Vertical"),
        }
    }

    #[test]
    fn update_uses_current_tab_window() {
        let mut tabs = HashMap::new();
        tabs.insert(1, Window::Tasks(ViewPort::default()));
        tabs.insert(2, Window::Tasks(ViewPort::default()));

        let mut buffers = HashMap::new();
        buffers.insert(1, Buffer::Empty);

        let mut app = App {
            commandline: CommandLine::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: 1,
            },
            tabs,
            current_tab_id: 2,
        };

        let area = Rect {
            x: 0,
            y: 0,
            width: 80,
            height: 40,
        };

        update(&mut app, area).unwrap();

        let tab_one = app.tabs.get(&1).expect("tab 1 exists");
        let tab_two = app.tabs.get(&2).expect("tab 2 exists");

        let tab_one_height = match tab_one {
            Window::Tasks(vp) => vp.height,
            _ => panic!("expected Tasks for tab 1"),
        };
        let tab_two_height = match tab_two {
            Window::Tasks(vp) => vp.height,
            _ => panic!("expected Tasks for tab 2"),
        };

        assert_eq!(tab_one_height, 0, "tab 1 should be unchanged");
        assert_eq!(tab_two_height, 38, "tab 2 should be updated");
    }
}
