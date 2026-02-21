use std::collections::HashSet;

use crate::model::{App, Buffer, Window};

pub fn update(app: &mut App) {
    let referenced: HashSet<usize> = match &app.window {
        Window::Horizontal(_, _) => return,
        Window::Directory(parent, current, preview) => {
            HashSet::from([parent.buffer_id, current.buffer_id, preview.buffer_id])
        }
    };

    let stale_images: Vec<usize> = app
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
            total_buffers_before = app.buffers.len(),
            "removing stale image buffers"
        );
    }

    for id in stale_images {
        app.buffers.remove(&id);
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
        app.buffers.insert(
            image_id,
            Buffer::Image(PreviewImageBuffer {
                path: PathBuf::from("/tmp/image.png"),
                protocol: Protocol::Sixel(Sixel::default()),
            }),
        );

        update(&mut app);

        assert!(!app.buffers.contains_key(&image_id));
    }

    #[test]
    fn keeps_referenced_image() {
        let mut app = App::default();
        let (_, _, preview_id) = crate::update::app::directory_buffer_ids(&app);

        app.buffers.insert(
            preview_id,
            Buffer::Image(PreviewImageBuffer {
                path: PathBuf::from("/tmp/preview.png"),
                protocol: Protocol::Sixel(Sixel::default()),
            }),
        );

        update(&mut app);

        assert!(matches!(
            app.buffers.get(&preview_id),
            Some(Buffer::Image(_))
        ));
    }

    #[test]
    fn does_not_touch_non_image_buffers() {
        let mut app = App::default();

        let content_id = 77;
        app.buffers.insert(
            content_id,
            Buffer::Content(ContentBuffer {
                path: PathBuf::from("/tmp/file.txt"),
                ..Default::default()
            }),
        );

        let directory_id = 78;
        app.buffers
            .insert(directory_id, Buffer::Directory(DirectoryBuffer::default()));

        update(&mut app);

        assert!(matches!(
            app.buffers.get(&content_id),
            Some(Buffer::Content(_))
        ));
        assert!(matches!(
            app.buffers.get(&directory_id),
            Some(Buffer::Directory(_))
        ));
    }
}
