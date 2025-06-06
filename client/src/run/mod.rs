use std::sync::Arc;
use shipyard::{UniqueView, World};
use tracing::error;
use wgpu::SurfaceError;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::window::WindowBuilder;
use engine::application::exit::{request_exit, ExitRequested};
use engine::application::plugin_manager::PluginManager;
use engine::rendering::graphics_context::GraphicsContext;
use core_workloads::{startup_core, update_core, post_update_core};
use strck::IntoCk;
use dino_plugins::path;

mod core_workloads;
mod input;
pub mod resize;
pub mod lost_focus;

pub fn run(plugin_manager: PluginManager) {
    let client = "client".ck().expect("valid ident");

    // initialize world and workloads
    let world = World::new();

    if let Some(dep) = plugin_manager.first_unmet_dependency() {
        error!(r#"One or more plugins depends on "{dep}", which isn't loaded."#);
        return;
    }

    plugin_manager.build_into(&world, client);

    world.add_workload(startup_core);
    world.add_workload(update_core);
    world.add_workload(post_update_core);

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

    world.run_workload(path!({client}::startup))
        .expect("TODO: panic message");

    let res = event_loop.run(move |event, control_flow| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta }, ..
            } => world.run_with_data(input::mouse_motion, delta),
            Event::WindowEvent {
                ref event,
                window_id,
            } if world.borrow::<UniqueView<GraphicsContext>>()
                .is_ok_and(|g_ctx| g_ctx.window.id() == window_id)
            => if !world.run_with_data(input::input, event) {
                match event {
                    WindowEvent::RedrawRequested => { // TODO: check to ensure it's the same window
                        world.run_workload(update_core)
                            .expect("TODO: panic message");

                        world.run_workload(path!({client}::update))
                            .expect("TODO: failed to run update workload");

                        world.run_workload(post_update_core)
                            .expect("TODO: panic message");

                        if let Err(err) = world.run_workload(path!({client}::render)) {
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
                            world.run_workload(path!({client}::shutdown))
                                .expect("TODO: failed to run shutdown workload");
                            control_flow.exit();
                        }
                    }
                    WindowEvent::CloseRequested => world.run(request_exit),
                    WindowEvent::Resized(physical_size) => world.run_with_data(resize::resize, *physical_size),
                    WindowEvent::Focused(focused) => world.run_with_data(lost_focus::on_window_focus_change, *focused),
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