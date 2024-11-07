use glm::Vec2;
use shipyard::{UniqueView, UniqueViewMut};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::PhysicalKey;
use crate::application::capture_state::CaptureState;
use crate::input::InputManager;
use crate::rendering::EguiRenderer;
use crate::rendering::graphics_context::GraphicsContext;

pub fn mouse_motion(delta: (f64, f64), capture: UniqueView<CaptureState>, mut input: UniqueViewMut<InputManager>) {
    if capture.is_captured() {
        input.mouse_manager.rotate = Vec2::new(delta.0 as _, delta.1 as _);
    }
}

pub fn input(event: &WindowEvent, mut input_manager: UniqueViewMut<InputManager>, mut egui_renderer: UniqueViewMut<EguiRenderer>, g_ctx: UniqueView<GraphicsContext>, capture: UniqueView<CaptureState>) -> bool {
    if !capture.is_captured() {
        egui_renderer.handle_input(&g_ctx.window, event);
        return false;
    }
    
    match event {
        WindowEvent::KeyboardInput {
            event: KeyEvent {
                physical_key: PhysicalKey::Code(key),
                state,
                ..
            },
            ..
        } => input_manager.process_input(key, *state == ElementState::Pressed),
        WindowEvent::MouseWheel { delta, .. } => {
            input_manager.mouse_manager.process_scroll(delta);
            true
        },
        WindowEvent::MouseInput { button, state, .. } => input_manager.process_input(button, *state == ElementState::Pressed),
        _ => false, // returns false if the event hasn't been processed, so it can be further processed later
    }
}