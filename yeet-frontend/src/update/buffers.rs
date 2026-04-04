use crate::model::{App, Buffer};

pub fn update(app: &mut App) {
    let referenced = match app.current_window() {
        Ok(window) => window.buffer_ids(),
        Err(_) => return,
    };

    let stale_images: Vec<usize> = app
        .contents
        .buffers
        .iter()
        .filter_map(|(id, buffer)| {
            if matches!(
                buffer,
                Buffer::Image(_) | Buffer::Tasks(_) | Buffer::QuickFix(_)
            ) && !referenced.contains(id)
            {
                Some(*id)
            } else {
                None
            }
        })
        .collect();

    if !stale_images.is_empty() {
        tracing::trace!(
            stale_image_ids = ?stale_images,
            referenced_ids = ?referenced,
            total_buffers_before = app.contents.buffers.len(),
            "removing stale image buffers"
        );
    }

    for id in stale_images {
        app.contents.buffers.remove(&id);
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use ratatui_image::protocol::{sixel::Sixel, Protocol};

    use yeet_buffer::model::viewport::ViewPort;

    use crate::model::{
        App, Buffer, ContentBuffer, DirectoryBuffer, PreviewImageBuffer, SplitFocus, TasksBuffer,
        Window,
    };

    use super::update;

    #[test]
    fn removes_unreferenced_images() {
        let mut app = App::default();

        let image_id = 42;
        app.contents.buffers.insert(
            image_id,
            Buffer::Image(PreviewImageBuffer {
                path: PathBuf::from("/tmp/image.png"),
                protocol: Protocol::Sixel(Sixel::default()),
            }),
        );

        update(&mut app);

        assert!(!app.contents.buffers.contains_key(&image_id));
    }

    #[test]
    fn keeps_referenced_image() {
        let mut app = App::default();
        let window = app.current_window().expect("test requires current tab");
        let (_, _, preview_id) =
            crate::update::app::get_focused_directory_buffer_ids(window).unwrap();

        app.contents.buffers.insert(
            preview_id,
            Buffer::Image(PreviewImageBuffer {
                path: PathBuf::from("/tmp/preview.png"),
                protocol: Protocol::Sixel(Sixel::default()),
            }),
        );

        update(&mut app);

        assert!(matches!(
            app.contents.buffers.get(&preview_id),
            Some(Buffer::Image(_))
        ));
    }

    #[test]
    fn does_not_touch_non_image_buffers() {
        let mut app = App::default();

        let content_id = 77;
        app.contents.buffers.insert(
            content_id,
            Buffer::Content(ContentBuffer {
                path: PathBuf::from("/tmp/file.txt"),
                ..Default::default()
            }),
        );

        let directory_id = 78;
        app.contents
            .buffers
            .insert(directory_id, Buffer::Directory(DirectoryBuffer::default()));

        update(&mut app);

        assert!(matches!(
            app.contents.buffers.get(&content_id),
            Some(Buffer::Content(_))
        ));
        assert!(matches!(
            app.contents.buffers.get(&directory_id),
            Some(Buffer::Directory(_))
        ));
    }

    #[test]
    fn removes_unreferenced_task_buffers() {
        let mut app = App::default();

        let buffer_id = 42;
        app.contents
            .buffers
            .insert(buffer_id, Buffer::Tasks(TasksBuffer::default()));

        update(&mut app);

        assert!(!app.contents.buffers.contains_key(&buffer_id));
    }

    #[test]
    fn keeps_referenced_tasks_buffer() {
        let mut app = App::default();

        let tasks_buffer_id = 100;
        app.contents
            .buffers
            .insert(tasks_buffer_id, Buffer::Tasks(TasksBuffer::default()));

        let window = app.current_window_mut().expect("test requires current tab");
        let old_window = std::mem::take(window);
        *window = Window::Horizontal {
            first: Box::new(old_window),
            second: Box::new(Window::Tasks(ViewPort {
                buffer_id: tasks_buffer_id,
                ..Default::default()
            })),
            focus: SplitFocus::Second,
        };

        update(&mut app);

        assert!(matches!(
            app.contents.buffers.get(&tasks_buffer_id),
            Some(Buffer::Tasks(_))
        ));
    }
}
