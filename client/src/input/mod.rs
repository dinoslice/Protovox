use shipyard::Unique;
use crate::input::action_map::{Action, ActionMap};
use crate::input::mouse_manager::MouseManager;

pub mod action_map;
pub mod mouse_manager;

#[derive(Debug, Unique)]
pub struct InputManager {
    pressed: ActionMap,
    just_pressed: ActionMap,
    
    queue: Vec<(Action, bool)>,
    
    pub mouse_manager: MouseManager,
}

impl InputManager {
    pub fn with_mouse_sensitivity(sensitivity: f32) -> Self {
        Self {
            pressed: ActionMap::default(),
            just_pressed: ActionMap::default(),
            mouse_manager: MouseManager::with_sensitivity(sensitivity),
            queue: Vec::default(),
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
            self.pressed.set_action(action, state);
            
            if state {
                self.just_pressed.set_action(action, true);
            }
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
