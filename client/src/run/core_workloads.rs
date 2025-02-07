use std::time::Instant;
use shipyard::{AllStoragesView, IntoWorkload, Unique, UniqueView, UniqueViewMut, Workload};
use winit::window::Fullscreen;
use engine::application::CaptureState;
use engine::application::delta_time::LastDeltaTime;
use engine::input::action_map::Action;
use engine::input::InputManager;
use engine::input::last_frame_events::LastFrameEvents;
use engine::input::mouse_manager::MouseManager;
use engine::rendering::graphics_context::GraphicsContext;

pub fn startup_core() -> Workload {
    (
        initialize_application_systems
    ).into_workload()
}

fn initialize_application_systems(storages: AllStoragesView) {
    storages.add_unique(InputManager::with_mouse_manager(MouseManager::new(0.75, 50.0)));
    storages.add_unique(CaptureState::default());
    storages.add_unique(LastDeltaTime::default());
    storages.add_unique(LastFrameEvents(Vec::new()));
    storages.add_unique(LastRenderInstant(Instant::now()));
}

pub fn update_core() -> Workload {
    (
        update_delta_time,
        update_input_manager,
        toggle_fullscreen,
    ).into_workload()
}

fn update_input_manager(mut input: UniqueViewMut<InputManager>) {
    input.process();
}

fn toggle_fullscreen(input: UniqueView<InputManager>, g_ctx: UniqueViewMut<GraphicsContext>) {
    if !input.just_pressed().get_action(Action::ToggleFullscreen) {
        return;
    }

    match g_ctx.window.fullscreen() {
        None => g_ctx.window.set_fullscreen(Some(Fullscreen::Borderless(None))),
        Some(_) => g_ctx.window.set_fullscreen(None)
    }
}

#[derive(Debug, Unique)]
pub struct LastRenderInstant(pub Instant);

pub fn update_delta_time(mut last_render_instant: UniqueViewMut<LastRenderInstant>, mut last_delta_time: UniqueViewMut<LastDeltaTime>) {
    let now = Instant::now();

    *last_delta_time = LastDeltaTime(now - last_render_instant.0);
    last_render_instant.0 = now;
}