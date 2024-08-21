use std::sync::Arc;
use std::time::{Duration, Instant};
use glm::{Vec2, Vec3};
use na::Perspective3;
use rand::prelude::SliceRandom;
use rand::Rng;
use shipyard::{AllStoragesView, UniqueView, UniqueViewMut, World};
use tracing::error;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;
use game::block::Block;
use game::chunk::data::ChunkData;
use crate::camera::Camera;
use crate::input::InputManager;
use crate::rendering::chunk_mesh::ChunkMesh;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::{render, renderer};
use crate::capture_state::CaptureState;

pub fn run() {
    let event_loop = EventLoopBuilder::new().build()
        .expect("event loop built successfully");

    let window = WindowBuilder::new()
        .with_title("voxel game")
        .build(&event_loop)
        .expect("window built successfully");

    let window = Arc::new(window);

    let world = World::new();

    world.add_unique(GraphicsContext::new(window));
    world.add_workload(renderer::initialize_renderer);

    world.run_workload(renderer::initialize_renderer)
        .expect("failed to initialize renderer");

    world.run(init_chunk_faces);

    world.add_unique(InputManager::with_mouse_sensitivity(0.75));
    world.add_unique(CaptureState(false));

    world.run(initialize_camera);

    let mut last_render_time = Instant::now();

    let res = event_loop.run(move |event, control_flow| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta }, ..
            } => if world.borrow::<UniqueView<CaptureState>>().unwrap().is_captured() {
                world.borrow::<UniqueViewMut<InputManager>>()
                    .unwrap()
                    .mouse_manager
                    .rotate = Vec2::new(delta.0 as f32, delta.1 as f32);
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if world.borrow::<UniqueView<GraphicsContext>>().unwrap().window.id() == window_id =>
                if !world.borrow::<UniqueView<CaptureState>>().unwrap().is_captured() || !world.run_with_data(input, event) {
                    match event {
                        WindowEvent::RedrawRequested => { // TODO: check to ensure it's the same window
                            world.run_with_data(update_camera_from_input_manager, &last_render_time.elapsed());
                            last_render_time = Instant::now();


                            match world.run(render::render) {
                                Ok(_) => {}
                                // Reconfigure the surface if lost
                                Err(wgpu::SurfaceError::Lost) => world.run(reconfigure),
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
                        WindowEvent::Resized(physical_size) => world.run_with_data(resize, *physical_size),

                        WindowEvent::Focused(focused) => {
                            world.run(|mut capture_state: UniqueViewMut<CaptureState>, g_ctx: UniqueView<GraphicsContext>, mut input_manager: UniqueViewMut<InputManager>| {
                                if capture_state.set(&g_ctx.window, *focused).is_none() {
                                    error!("Unable to set capture/release mouse cursor.")
                                } else if !focused { // only reset action map if released cursor
                                    input_manager.action_map.reset_all();
                                }
                            });
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
                            world.run(|mut capture_state: UniqueViewMut<CaptureState>, g_ctx: UniqueView<GraphicsContext>, mut input: UniqueViewMut<InputManager>| {
                                match capture_state.toggle(&g_ctx.window) {
                                    Some(false) => input.action_map.reset_all(),
                                    None => error!("Unable to set capture/release mouse cursor."),
                                    _ => {}
                                }
                            });
                        }
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
        error!("Event loop encountered an error: {:?}", err);
    }
}

pub fn initialize_camera(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    storages.add_unique(Camera {
        position: Vec3::new(0.0, 0.0, 0.0),
        yaw: 90.0f32.to_radians(),
        pitch: -20.0f32.to_radians(),
        speed: 8.0,
        perspective: Perspective3::new(
            g_ctx.aspect(),
            45.0f32.to_radians(),
            0.01,
            1000.0
        )
    })
}

fn init_chunk_faces(storages: AllStoragesView) {
    let mut chunk = ChunkData::default();

    for i in 0..65536 {
        if rand::thread_rng().gen_bool(0.1) {
            chunk.blocks[i] = *[
                Block::Grass,
                Block::Dirt,
                Block::Cobblestone,
            ].choose(&mut rand::thread_rng())
                .expect("blocks exist");
        }
    }

    // TODO: move this elsewhere
    let baked = ChunkMesh::from_chunk(&chunk);
    storages.add_unique(baked);
}

fn input(event: &WindowEvent, mut input_manager: UniqueViewMut<InputManager>) -> bool {
    match event {
        WindowEvent::KeyboardInput {
            event:
            KeyEvent {
                physical_key: PhysicalKey::Code(key),
                state,
                ..
            },
            ..
        } => input_manager.action_map.process_input(key, *state == ElementState::Pressed),
        WindowEvent::MouseWheel { delta, .. } => {
            input_manager.mouse_manager.process_scroll(delta);
            true
        }
        _ => false, // returns false if the event hasn't been processed, so it can be further processed later
    }
}

fn resize(new_size: winit::dpi::PhysicalSize<u32>, mut g_ctx: UniqueViewMut<GraphicsContext>, mut camera: UniqueViewMut<Camera>) {
    if new_size.width > 0 && new_size.height > 0 {
        g_ctx.resize(new_size);
        camera.perspective.set_aspect(g_ctx.aspect());
    } else {
        tracing::warn!("Ignoring resize with non-positive width or height");
    }
}

fn reconfigure(g_ctx: UniqueViewMut<GraphicsContext>, camera: UniqueViewMut<Camera>) {
    let size = g_ctx.size;
    resize(size, g_ctx, camera);
}

fn update_camera_from_input_manager(delta_time: &Duration, mut camera: UniqueViewMut<Camera>, mut input_manager: UniqueViewMut<InputManager>) {
    camera.update_with_input(&mut input_manager, delta_time);
}