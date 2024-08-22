use std::sync::Arc;
use std::time::{Duration, Instant};
use shipyard::{UniqueView, UniqueViewMut, World};
use tracing::error;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;
use crate::camera::Camera;
use crate::input::InputManager;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::render;
use crate::workloads::startup;

mod capture_state;
mod input;
mod resize;
pub mod delta_time;

pub use capture_state::CaptureState;

pub fn run() {
    let event_loop = EventLoopBuilder::new().build()
        .expect("event loop built successfully");

    let window = WindowBuilder::new()
        .with_title("voxel game")
        .build(&event_loop)
        .expect("window built successfully");

    let window = Arc::new(window);

    let world = World::new();

    world.add_workload(startup);

    world.add_unique(GraphicsContext::new(window));
    world.run_workload(startup).expect("TODO: panic message");

    let mut last_render_time = Instant::now();

    let res = event_loop.run(move |event, control_flow| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta }, ..
            } => world.run_with_data(input::mouse_motion, delta),
            Event::WindowEvent {
                ref event,
                window_id,
            } if world.borrow::<UniqueView<GraphicsContext>>().unwrap().window.id() == window_id =>
                if !world.run(capture_state::is_captured) || !world.run_with_data(input::input, event) {
                    match event {
                        WindowEvent::RedrawRequested => { // TODO: check to ensure it's the same window
                            world.run_with_data(update_camera_from_input_manager, &last_render_time.elapsed());
                            last_render_time = Instant::now();


                            match world.run(render::render) {
                                Ok(_) => {}
                                // Reconfigure the surface if lost
                                Err(wgpu::SurfaceError::Lost) => world.run(resize::reconfigure),
                                // Quit if the system is out of memory
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    error!("System is out of memory!");
                                    panic!("System is out of memory!");
                                },
                                // All other errors (Outdated, Timeout) should be resolved by the next frame
                                Err(e) => error!("{e:?}"),
                            }
                        }
                        WindowEvent::CloseRequested => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => world.run_with_data(resize::resize, *physical_size),

                        WindowEvent::Focused(focused) => world.run_with_data(capture_state::set_from_focus, *focused),
                        WindowEvent::KeyboardInput {
                            event: KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                            ..
                        } => world.run(capture_state::toggle_captured),
                        _ => {}
                    }
                }
            Event::AboutToWait => {
                // RedrawRequested only triggers once manually requested
                world.run(|g_ctx: UniqueView<GraphicsContext>|
                    g_ctx.window.request_redraw()
                );
            }
            _ => {}
        }
    });

    if let Err(err) = res {
        error!("Event loop encountered an error: {err:?}");
    }
}

fn update_camera_from_input_manager(delta_time: &Duration, mut camera: UniqueViewMut<Camera>, mut input_manager: UniqueViewMut<InputManager>) {
    camera.update_with_input(&mut input_manager, delta_time);
}
