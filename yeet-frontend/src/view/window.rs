use ratatui::Frame;

use crate::{error::AppError, model::Model};

use super::{buffer, tabbar};

pub fn view(model: &Model, frame: &mut Frame) -> Result<u16, AppError> {
    let vertical_offset = tabbar::render(&model.app, frame);
    buffer::view(
        &model.state.modes.current,
        &model.app,
        frame,
        0,
        vertical_offset,
    );

    let window = model.app.current_window()?;
    Ok(window.get_height().saturating_add(vertical_offset))
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
    fn view_returns_window_height_without_tabbar() {
        let model = make_model(1);
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("create terminal");
        let mut rendered_height = None;

        terminal
            .draw(|frame| {
                rendered_height = Some(view(&model, frame).expect("render view"));
            })
            .expect("draw frame");

        assert_eq!(rendered_height, Some(11));
    }

    #[test]
    fn view_adds_tabbar_height_when_multiple_tabs() {
        let model = make_model(2);
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("create terminal");
        let mut rendered_height = None;

        terminal
            .draw(|frame| {
                rendered_height = Some(view(&model, frame).expect("render view"));
            })
            .expect("draw frame");

        assert_eq!(rendered_height, Some(12));
    }
}
