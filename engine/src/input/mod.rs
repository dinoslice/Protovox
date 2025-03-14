use shipyard::{Unique, UniqueViewMut};
use crate::input::action_map::{Action, ActionMap};
use crate::input::mouse_manager::MouseManager;

pub mod action_map;
pub mod mouse_manager;
pub mod last_frame_events;

#[derive(Debug, Unique)]
pub struct InputManager {
    pressed: ActionMap,
    just_pressed: ActionMap,
    
    queue: Vec<(Action, bool)>,
    
    pub mouse_manager: MouseManager,
}

impl InputManager {
    pub fn with_mouse_manager(mouse_manager: MouseManager) -> Self {
        Self {
            pressed: ActionMap::default(),
            just_pressed: ActionMap::default(),
            queue: Vec::default(),
            mouse_manager,
        }
    }

    pub fn process_input(&mut self, input: impl TryInto<Action>, is_pressed: bool) -> bool {
        match input.try_into() {
            Ok(action) => {
                self.queue.push((action, is_pressed));
                true
            }
            Err(_) => false
        }
    }
    
    pub fn process(&mut self) {
        self.just_pressed.reset_all();
        
        for (action, state) in self.queue.drain(..) {
            if state && !self.pressed.get_action(action) {
                self.just_pressed.set_action(action, true);
            }

            self.pressed.set_action(action, state);
        }
    }
    
    pub fn pressed(&self) -> &ActionMap {
        &self.pressed
    }
    
    pub fn just_pressed(&self) -> &ActionMap {
        &self.just_pressed
    }

    pub fn reset_all(&mut self) {
        self.pressed.reset_all();
        self.just_pressed.reset_all();
        self.queue.clear();
        self.mouse_manager.reset_scroll_rotate();
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::with_mouse_manager(MouseManager::default())
    }
}

pub fn reset_mouse_manager_state(mut input_manager: UniqueViewMut<InputManager>) {
    input_manager.mouse_manager.reset_scroll_rotate();
}