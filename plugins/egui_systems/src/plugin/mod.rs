use shipyard::{AllStoragesView, IntoWorkload, SystemModificator, UniqueView, UniqueViewMut, Workload, WorkloadModificator};
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePhase, EnginePluginMetadata, RenderUiStartMarker};
use dino_plugins::path;
use engine::application::CaptureState;
use engine::input::last_frame_events::LastFrameEvents;
use engine::rendering::graphics_context::GraphicsContext;
use engine::VoxelEngine;
use crate::renderer::EguiRenderer;

pub mod frame;
pub mod order;
mod render;

pub struct EguiSystemsPlugin;

impl DinoEnginePlugin for EguiSystemsPlugin {
    fn early_startup(&self) -> Option<Workload> {
        initialize_renderer
            .tag(path!({self}::{EnginePhase::EarlyStartup}::initialize_renderer))
            .after_all(path!({VoxelEngine}::{EnginePhase::EarlyStartup}::rendering::initialize))
            .into_workload()
            .into()
    }

    fn late_update(&self) -> Option<Workload> {
        process_events
            .into_workload()
            .into()
    }

    fn render(&self) -> Option<Workload> {
        pub use render::*;

        (
            render_start
                .tag(path!({self}::{EnginePhase::Render}::render_start))
                .tag(path!({RenderUiStartMarker})),
            render_end.tag(path!({self}::{EnginePhase::Render}::render_end)),
        )
            .into_sequential_workload()
            .after_all(path!({VoxelEngine}::{EnginePhase::Render}::render_world))
            .before_all(path!({VoxelEngine}::{EnginePhase::Render}::submit_rendered_frame))
            .into()
    }

    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "egui_systems".ck().expect("should be a valid identifier"),
            version: env!("CARGO_PKG_VERSION"),
            dependencies: &[&VoxelEngine],
        }
    }
}

fn initialize_renderer(g_ctx: UniqueView<GraphicsContext>, all_storages: AllStoragesView) {
    all_storages.add_unique(EguiRenderer::new(&g_ctx.device, g_ctx.config.format, None, 1, &g_ctx.window))
}

fn process_events(g_ctx: UniqueView<GraphicsContext>, last_frame_events: UniqueView<LastFrameEvents>, mut egui_renderer: UniqueViewMut<EguiRenderer>, capture_state: UniqueView<CaptureState>) {
    let window = &g_ctx.window;
    
    if !capture_state.is_captured() {
        for evt in &last_frame_events.0 {
            egui_renderer.handle_input(window, evt);
        }
    }
}