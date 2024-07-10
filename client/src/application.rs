use std::time::Instant;
use glm::Vec2;
use tracing::error;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoopBuilder;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowBuilder};
use server::chunk::pos::ChunkPos;
use crate::rendering::face_data::{FaceData, FaceType};
use crate::state::State;

pub fn run() {
    let event_loop = EventLoopBuilder::new().build()
        .expect("event loop built successfully");

    let window = WindowBuilder::new()
        .with_title("voxel game")
        .build(&event_loop)
        .expect("window built successfully");

    let mut state = State::new(&window);
    let mut last_render_time = Instant::now();
    let mut capture_state = CaptureState(false);

    // TODO: move this elsewhere
    let instances = vec![
        FaceData::new(ChunkPos::new_unchecked(0, 0, 1), FaceType::Top),
        FaceData::new(ChunkPos::new_unchecked(0, 0, 1), FaceType::Bottom),
        FaceData::new(ChunkPos::new_unchecked(0, 0, 1), FaceType::Left),
        FaceData::new(ChunkPos::new_unchecked(0, 0, 1), FaceType::Right),
        FaceData::new(ChunkPos::new_unchecked(0, 0, 1), FaceType::Front),
        // FaceData::new(ChunkPos::new_unchecked(0, 0, 1), FaceType::Back),

        FaceData::new(ChunkPos::new_unchecked(0, 0, 0), FaceType::Top),
        FaceData::new(ChunkPos::new_unchecked(0, 0, 0), FaceType::Bottom),
        FaceData::new(ChunkPos::new_unchecked(0, 0, 0), FaceType::Left),
        FaceData::new(ChunkPos::new_unchecked(0, 0, 0), FaceType::Right),
        // FaceData::new(ChunkPos::new_unchecked(0, 0, 0), FaceType::Front),
        FaceData::new(ChunkPos::new_unchecked(0, 0, 0), FaceType::Back),
    ];

    let res = event_loop.run(move |event, control_flow| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta }, ..
            } => if capture_state.is_captured() {
                state.input_manager.mouse_manager.rotate = Vec2::new(delta.0 as f32, delta.1 as f32);
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.renderer.graphics_context.window.id() => if !capture_state.is_captured() || !state.input(event) {
                match event {
                    WindowEvent::RedrawRequested => { // TODO: check to ensure it's the same window
                        state.update(&last_render_time.elapsed());
                        last_render_time = Instant::now();
                        match state.renderer.render(&state.camera, &instances) {
                            Ok(_) => {}
                            // Reconfigure the surface if lost
                            Err(wgpu::SurfaceError::Lost) => state.renderer.reconfigure(),
                            // Quit if the system is out of memory
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                error!("System is out of memory!");
                                panic!("System is out of memory!");
                            },
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => error!("{:?}", e),
                        }
                    }
                    WindowEvent::CloseRequested => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::Focused(focused) => {
                        if capture_state.set(state.renderer.graphics_context.window, *focused).is_none() {
                            error!("Unable to set capture/release mouse cursor.")
                        }
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
                        if capture_state.toggle(state.renderer.graphics_context.window).is_none() {
                            error!("Unable to set capture/release mouse cursor.")
                        }
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
    });

    if let Err(err) = res {
        error!("Event loop encountered an error: {:?}", err);
    }
}

struct CaptureState(bool);

impl CaptureState {
    pub fn set(&mut self, window: &Window, captured: bool) -> Option<()> {
        self.0 = captured;

        let cursor_grab = match self.0 {
            true => CursorGrabMode::Confined,
            false => CursorGrabMode::None,
        };

        window.set_cursor_visible(!self.0);
        window.set_cursor_grab(cursor_grab).ok()
    }

    pub fn is_captured(&self) -> bool {
        self.0
    }

    pub fn toggle(&mut self, window: &Window) -> Option<bool> {
        self.set(window, !self.0).map(|_| self.0)
    }
}