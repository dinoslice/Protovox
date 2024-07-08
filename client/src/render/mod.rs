use std::time::Instant;
use glm::Vec2;
use tracing::error;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, WindowBuilder};
use crate::render::state::State;

pub mod state;
pub mod vertex;
pub mod texture;
pub mod camera;
mod instance;
mod graphics_context;
mod camera_uniform_buffer;
mod renderer;

pub fn run() {
    let event_loop = EventLoopBuilder::new().build().unwrap();
    let window = WindowBuilder::new()
        .with_title("voxel game")
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&window);
    let mut last_render_time = Instant::now();

    let mut active = false;

    event_loop.run(move |event, control_flow| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta }, ..
            } => if active {
                state.input_manager.mouse_manager.rotate = Vec2::new(delta.0 as f32, delta.1 as f32);
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.renderer.graphics_context.window.id() => if !active || !state.input(event) {
                match event {
                    WindowEvent::RedrawRequested => { // TODO: check to ensure it's the same window
                        state.update(&last_render_time.elapsed());
                        last_render_time = Instant::now();
                        match state.renderer.render(&state.camera) {
                            Ok(_) => {}
                            // Reconfigure the surface if lost
                            Err(wgpu::SurfaceError::Lost) => state.renderer.reconfigure(),
                            // Quit if the system is out of memory
                            Err(wgpu::SurfaceError::OutOfMemory) => panic!("OOM, TODO: properly exit"),
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => error!("{:?}", e),
                        }
                    }
                    WindowEvent::CloseRequested => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::Focused(focused) => {
                        let cursor_grab = match *focused {
                            true => CursorGrabMode::Confined,
                            false => CursorGrabMode::None,
                        };

                        active = *focused;
                        state.renderer.graphics_context.window.set_cursor_grab(cursor_grab)
                            .expect("failed to set cursor grab");
                        state.renderer.graphics_context.window.set_cursor_visible(!focused);
                    }
                    WindowEvent::KeyboardInput {
                        event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                        ..
                    } => {
                        active = !active;

                        let cursor_grab = match active {
                            true => CursorGrabMode::Confined,
                            false => CursorGrabMode::None,
                        };

                        state.renderer.graphics_context.window.set_cursor_grab(cursor_grab)
                            .expect("failed to set cursor grab");
                        state.renderer.graphics_context.window.set_cursor_visible(!active);
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                // RedrawRequested only triggers once manually requested
                state.renderer.graphics_context.window.request_redraw();
            }
            _ => {}
        }
    }).expect("event loop run");
}