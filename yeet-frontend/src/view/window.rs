use ratatui::Frame;

use crate::{error::AppError, model::Model};

use super::{buffer, tabbar};

pub fn view(model: &Model, frame: &mut Frame) -> Result<(), AppError> {
    tabbar::render(&model.app, &model.settings, frame);
    buffer::view(
        &model.state.modes.current,
        &model.app,
        &model.settings,
        frame,
    );

    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use ratatui::style::Color;
    use ratatui::{backend::TestBackend, Terminal};
    use yeet_buffer::model::viewport::ViewPort;
    use yeet_buffer::model::TextBuffer;

    use crate::{
        model::{App, Buffer, CommandLine, Contents, Model, TasksBuffer, Window},
        settings::Settings,
    };

    use super::view;

    fn make_model(tab_count: usize) -> Model {
        let tabbar_offset = if tab_count > 1 { 1 } else { 0 };
        let mut buffers = HashMap::new();
        let mut tabs = HashMap::new();

        for id in 1..=tab_count {
            let buffer_id = id;
            buffers.insert(buffer_id, Buffer::Tasks(TasksBuffer::default()));
            tabs.insert(
                id,
                Window::Tasks(ViewPort {
                    buffer_id,
                    y: tabbar_offset,
                    width: 80,
                    height: 10,
                    ..Default::default()
                }),
            );
        }

        let app = App {
            commandline: CommandLine::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: tab_count,
            },
            tabs,
            current_tab_id: 1,
        };

        Model {
            app,
            settings: Settings::default(),
            state: Default::default(),
        }
    }

    #[test]
    fn view_applies_buffer_background_color() {
        let mut model = make_model(1);
        model.settings.theme.buffer_bg = Color::Rgb(0x10, 0x20, 0x30);

        let backend = TestBackend::new(20, 6);
        let mut terminal = Terminal::new(backend).expect("create terminal");

        terminal
            .draw(|frame| {
                view(&model, frame).expect("render view");
            })
            .expect("draw frame");

        let buffer = terminal.backend().buffer();
        let cell = buffer.cell((0, 1)).expect("cell exists");
        assert_eq!(cell.bg, Color::Rgb(0x10, 0x20, 0x30));
    }

    #[test]
    fn view_applies_distinct_border_backgrounds() {
        let mut model = make_model(1);
        model.settings.theme.miller_border_bg = Color::Rgb(0x11, 0x11, 0x11);
        model.settings.theme.split_border_bg = Color::Rgb(0x22, 0x22, 0x22);

        let buffer_id = 1;
        model.app.contents.buffers.insert(
            buffer_id,
            Buffer::Directory(crate::model::DirectoryBuffer {
                buffer: TextBuffer::default(),
                path: Default::default(),
                state: Default::default(),
            }),
        );

        let left = ViewPort {
            buffer_id,
            x: 0,
            y: 0,
            width: 8,
            height: 3,
            show_border: true,
            ..Default::default()
        };
        let right = ViewPort {
            buffer_id,
            x: 8,
            y: 0,
            width: 8,
            height: 3,
            show_border: true,
            ..Default::default()
        };
        let preview = ViewPort {
            buffer_id,
            x: 16,
            y: 0,
            width: 4,
            height: 3,
            show_border: false,
            ..Default::default()
        };

        let right_tasks = ViewPort {
            buffer_id,
            x: 20,
            y: 0,
            width: 6,
            height: 3,
            show_border: true,
            ..Default::default()
        };

        model.app.tabs.insert(
            1,
            Window::Vertical {
                first: Box::new(Window::Directory(left, right, preview)),
                second: Box::new(Window::Tasks(right_tasks)),
                focus: crate::model::SplitFocus::First,
            },
        );

        let backend = TestBackend::new(26, 6);
        let mut terminal = Terminal::new(backend).expect("create terminal");

        terminal
            .draw(|frame| {
                view(&model, frame).expect("render view");
            })
            .expect("draw frame");

        let buffer = terminal.backend().buffer();
        let miller_border_cell = buffer.cell((7, 0)).expect("miller border cell");
        let split_border_cell = buffer.cell((25, 0)).expect("split border cell");

        assert_eq!(miller_border_cell.bg, Color::Rgb(0x11, 0x11, 0x11));
        assert_eq!(split_border_cell.bg, Color::Rgb(0x22, 0x22, 0x22));
    }

    #[test]
    fn view_keeps_default_border_background_reset() {
        let model = make_model(1);

        let backend = TestBackend::new(20, 6);
        let mut terminal = Terminal::new(backend).expect("create terminal");

        terminal
            .draw(|frame| {
                view(&model, frame).expect("render view");
            })
            .expect("draw frame");

        let buffer = terminal.backend().buffer();
        let cell = buffer.cell((0, 1)).expect("cell exists");
        assert_eq!(cell.bg, Color::Reset);
    }

    #[test]
    fn view_renders_without_tabbar() {
        let model = make_model(1);
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("create terminal");
        let mut result = None;

        terminal
            .draw(|frame| {
                result = Some(view(&model, frame));
            })
            .expect("draw frame");

        assert!(matches!(result, Some(Ok(()))));

        let buffer = terminal.backend().buffer();
        let row = read_row(buffer, 0);
        assert!(row.trim().is_empty(), "expected no tabbar for single tab");
    }

    #[test]
    fn view_renders_with_tabbar() {
        let model = make_model(2);
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("create terminal");
        let mut result = None;

        terminal
            .draw(|frame| {
                result = Some(view(&model, frame));
            })
            .expect("draw frame");

        assert!(matches!(result, Some(Ok(()))));

        let buffer = terminal.backend().buffer();
        let row = read_row(buffer, 0);
        assert!(row.contains(" 1 "), "expected tabbar labels");
    }

    fn read_row(buffer: &ratatui::buffer::Buffer, y: u16) -> String {
        let mut row = String::new();
        for x in 0..buffer.area.width {
            let cell = buffer.cell((x, y)).expect("cell exists");
            row.push_str(cell.symbol());
        }
        row
    }
}
