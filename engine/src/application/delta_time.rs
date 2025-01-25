use std::time::{Duration, Instant};
use shipyard::{Unique, UniqueViewMut};

#[derive(Unique, Default)]
pub struct LastDeltaTime(pub Duration);

impl LastDeltaTime {
    pub fn since_instant(instant: Instant) -> Self {
        Self(instant.elapsed())
    }
}

pub fn update_delta_time(last_render_time: Instant, mut last_delta_time: UniqueViewMut<LastDeltaTime>) {
    *last_delta_time = LastDeltaTime::since_instant(last_render_time);
}