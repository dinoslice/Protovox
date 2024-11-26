use std::sync::Arc;
use std::time::Instant;
use shipyard::{AsLabel, IntoWorkload, UniqueView, Workload, WorkloadModificator, World};
use tracing::error;
use wgpu::SurfaceError;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering;
use crate::application::exit::{request_exit, ExitRequested};
use crate::application::core_workloads::{startup_core, update_core};

mod capture_state;
mod input;
mod resize;
pub mod delta_time;
pub mod exit;
mod core_workloads;

pub use capture_state::CaptureState;
use crate::input::action_map::Action;
use crate::input::InputManager;

pub fn run_game() {
    use crate::workloads::*;

    run(startup, update, rendering::render, shutdown);
}

pub fn run(startup: fn() -> Workload, update: fn() -> Workload, render: fn() -> Workload, shutdown: fn() -> Workload) {
    // initialize world and workloads
    let world = World::new();

    startup
        .rename("startup")
        .add_to_world(&world)
        .expect("failed to add startup workload");

    update
        .rename("update")
        .add_to_world(&world)
        .expect("failed to add update workload");

    render
        .rename("render")
        .add_to_world(&world)
        .expect("failed to add render workload");

    shutdown
        .rename("shutdown")
        .add_to_world(&world)
        .expect("failed to add shutdown workload");

    world.add_workload(startup_core);
    world.add_workload(update_core);

    // create window and event loop
    let event_loop = EventLoopBuilder::new().build()
        .expect("event loop built successfully");

    let window = WindowBuilder::new()
        .with_title("voxel game")
        .build(&event_loop)
        .expect("window built successfully");

    let window = Arc::new(window);
    world.add_unique(GraphicsContext::new(window));

    world.run_workload(startup_core)
        .expect("TODO: panic message");

    world.run_workload("startup")
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
            => if !world.run_with_data(input::input, event) {
                match event {
                    WindowEvent::RedrawRequested => { // TODO: check to ensure it's the same window
                        if world.borrow::<UniqueView<InputManager>>()
                            .expect("input manager to exist")
                            .just_pressed()
                            .get_action(Action::DumpECSInfo)
                        {



                            std::fs::write(
                                "ecs info.json",
                                serde_json::to_string(&world.workloads_info()).unwrap(),
                            )
                                .unwrap();

                            tracing::info!("dumped ecs info to file");
                        }


                        world.run_with_data(delta_time::update_delta_time, last_render_time);
                        last_render_time = Instant::now();

                        world.run_workload(update_core)
                            .expect("TODO: panic message");

                        world.run_workload("update")
                            .expect("TODO: failed to run update workload");

                        if let Err(err) = world.run_workload("render") {
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

                        if world.get_unique::<&ExitRequested>().is_ok() {
                            // TODO: for now, immediately exit upon receiving ExitRequested
                            world.run_workload("shutdown")
                                .expect("TODO: failed to run shutdown workload");
                            control_flow.exit();
                        }
                    }
                    WindowEvent::CloseRequested => world.run(request_exit),
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
