use shipyard::{Unique, UniqueViewMut};
use winit::event::WindowEvent;

#[derive(Unique)]
pub struct LastFrameEvents(pub Vec<WindowEvent>);

pub fn clear_last_frame_events(mut last_frame_events: UniqueViewMut<LastFrameEvents>) {
    last_frame_events.0.clear();
}