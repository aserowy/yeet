use std::collections::HashSet;

use crate::model::{App, Buffer, Window};

pub fn update(app: &mut App) {
    let referenced: HashSet<usize> = match &app.window {
        Window::Horizontal { .. } => return,
        Window::Directory(parent, current, preview) => {
            HashSet::from([parent.buffer_id, current.buffer_id, preview.buffer_id])
        }
        Window::Tasks(_) => return,
    };

    let stale_images: Vec<usize> = app
        .contents
        .buffers
        .iter()
        .filter_map(|(id, buffer)| {
            if matches!(buffer, Buffer::Image(_)) && !referenced.contains(id) {
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

    use crate::model::{App, Buffer, ContentBuffer, DirectoryBuffer, PreviewImageBuffer};

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
        let (_, _, preview_id) = crate::update::app::get_focused_directory_buffer_ids(&app);

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
}
