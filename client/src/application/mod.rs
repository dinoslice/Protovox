use std::sync::Arc;
use std::time::Instant;
use shipyard::{UniqueView, World};
use tracing::error;
use wgpu::SurfaceError;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering;
use crate::workloads::{startup, update};

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
    world.add_workload(update);
    world.add_workload(rendering::render);

    world.add_unique(GraphicsContext::new(window));
    world.run_workload(startup)
        .expect("TODO: panic message");

    let mut last_render_time = Instant::now();

    let res = event_loop.run(move |event, control_flow| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta }, ..
            } => world.run_with_data(input::mouse_motion, delta),
            Event::WindowEvent {
                ref event,
                window_id,
            } if world.borrow::<UniqueView<GraphicsContext>>()
                .map_or(false, |g_ctx|
                    g_ctx.window.id() == window_id
                )
            => if !world.run(capture_state::is_captured) || !world.run_with_data(input::input, event) {
                match event {
                    WindowEvent::RedrawRequested => { // TODO: check to ensure it's the same window
                        world.run_with_data(delta_time::update_delta_time, last_render_time);
                        last_render_time = Instant::now();

                        world.run_workload(update)
                            .expect("TODO: panic message");

                        if let Err(err) = world.run_workload(rendering::render) {
                            match err
                                .custom_error()
                                .expect("TODO: workload error")
                                .downcast_ref::<SurfaceError>()
                                .expect("TODO: unhandled/unexpected error returned from system")
                            {
                                SurfaceError::Lost => world.run(resize::reconfigure),
                                SurfaceError::OutOfMemory => panic!("System is out of memory!"),
                                err => error!("{err:?}"),
                            }
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
