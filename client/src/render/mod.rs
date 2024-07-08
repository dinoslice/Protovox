use std::time::Instant;
use glm::Vec2;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;
use crate::render::state::State;

pub mod state;
pub mod vertex;
pub mod texture;
pub mod camera;
mod instance;

pub fn run() {
    let event_loop = EventLoopBuilder::new().build().unwrap();
    let window = WindowBuilder::new()
        .with_title("voxel game")
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(&window);
    let mut last_render_time = Instant::now();
    event_loop.run(move |event, control_flow| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta }, ..
            } => if state.input_manager.mouse_manager.pressed {
                state.input_manager.mouse_manager.rotate = Vec2::new(delta.0 as f32, delta.1 as f32);
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) {
                match event {
                    WindowEvent::RedrawRequested => { // TODO: check to ensure it's the same window
                        state.update(&last_render_time.elapsed());
                        last_render_time = Instant::now();
                        match state.render() {
                            Ok(_) => {}
                            // Reconfigure the surface if lost
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            // Quit if the system is out of memory
                            Err(wgpu::SurfaceError::OutOfMemory) => panic!("OOM, TODO: properly exit"),
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                // RedrawRequested only triggers once manually requested
                state.window().request_redraw();
            }
            _ => {}
        }
    }).expect("event loop run");
}