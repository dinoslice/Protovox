use std::time::{Duration, Instant};
use shipyard::Unique;

#[derive(Unique)] // TODO: define this in same file, effectively local
pub struct LastWorldInteraction(pub Instant);

impl Default for LastWorldInteraction {
    fn default() -> Self {
        Self(Instant::now())
    }
}

impl LastWorldInteraction {
    pub fn cooldown_passed(&self) -> bool {
        self.0.elapsed() >= Duration::from_secs_f32(0.2)
    }
    
    pub fn reset_cooldown(&mut self) {
        self.0 = Instant::now();
    }
}