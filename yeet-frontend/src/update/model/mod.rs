use ratatui::layout::Rect;
use yeet_buffer::model::viewport::ViewPort;

pub mod commandline;
pub mod current;
pub mod parent;
pub mod preview;

fn set_viewport_dimensions(vp: &mut ViewPort, rect: &Rect) {
    vp.height = usize::from(rect.height);
    vp.width = usize::from(rect.width);
}
