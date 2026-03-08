use ratatui::Frame;

use crate::{error::AppError, model::Model};

use super::{buffer, tabbar};

pub fn view(model: &Model, frame: &mut Frame) -> Result<(), AppError> {
    tabbar::render(&model.app, frame);
    buffer::view(&model.state.modes.current, &model.app, frame);

    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use ratatui::{backend::TestBackend, Terminal};
    use yeet_buffer::model::viewport::ViewPort;

    use crate::{
        model::{App, Buffer, CommandLine, Contents, Model, TasksBuffer, Window},
        settings::Settings,
    };

    use super::view;

    fn make_model(tab_count: usize) -> Model {
        let mut buffers = HashMap::new();
        let mut tabs = HashMap::new();

        for id in 1..=tab_count {
            let buffer_id = id;
            buffers.insert(buffer_id, Buffer::Tasks(TasksBuffer::default()));
            tabs.insert(
                id,
                Window::Tasks(ViewPort {
                    buffer_id,
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
        assert!(row.contains("1:"), "expected tabbar labels");
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
