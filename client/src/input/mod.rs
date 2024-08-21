use shipyard::Unique;
use crate::input::action_map::ActionMap;
use crate::input::mouse_manager::MouseManager;

pub mod action_map;
pub mod mouse_manager;

#[derive(Debug, Unique)]
pub struct InputManager {
    pub action_map: ActionMap,
    pub mouse_manager: MouseManager,
}

impl InputManager {
    pub fn with_mouse_sensitivity(sensitivity: f32) -> Self {
        Self {
            action_map: Default::default(),
            mouse_manager: MouseManager::with_sensitivity(sensitivity),
        }
    }
}
