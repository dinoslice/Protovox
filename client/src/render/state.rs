use std::time::Duration;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::PhysicalKey;
use winit::window::Window;
use crate::render::camera::Camera;
use crate::input::InputManager;
use crate::render::renderer::Renderer;

pub struct State<'a> {
    pub renderer: Renderer<'a>,

    pub camera: Camera,
    pub input_manager: InputManager,
}

impl<'a> State<'a> {
    pub fn new(window: &'a Window) -> State<'a> {
        use glm::Vec3;
        use na::Perspective3;

        let renderer = Renderer::new(window);

        let camera = Camera {
            position: Vec3::new(0.0, 1.0, 2.0),
            yaw: -90.0f32.to_radians(),
            pitch: -20.0f32.to_radians(),
            speed: 4.0,
            perspective: Perspective3::new(
                renderer.aspect(),
                45.0f32.to_radians(),
                0.1,
                100.0
            )
        };

        let input_manager = InputManager::with_mouse_sensitivity(0.4);

        Self { renderer, camera, input_manager }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.renderer.resize(new_size);
            self.camera.perspective.set_aspect(self.renderer.aspect());
        } else {
            tracing::warn!("Ignoring resize with non-positive width or height");
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(key),
                    state,
                    ..
                },
                ..
            } => self.input_manager.action_map.process_input(key, *state == ElementState::Pressed),
            WindowEvent::MouseWheel { delta, .. } => {
                self.input_manager.mouse_manager.process_scroll(delta);
                true
            }
            _ => false, // returns false if the event hasn't been processed, so it can be further processed later
        }
    }

    pub fn update(&mut self, delta_time: &Duration) {
        self.camera.update_with_input(&mut self.input_manager, delta_time);
    }
}