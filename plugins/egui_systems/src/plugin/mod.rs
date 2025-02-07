
use shipyard::{AllStoragesView, IntoWorkload, SystemModificator, UniqueView, Workload, WorkloadModificator};
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePhase, EnginePluginMetadata};
use dino_plugins::path;
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
            .after_all(path!({VoxelEngine}::{EnginePhase::EarlyStartup}::rendering::initialize))
            .into_workload()
            .into()
    }

    fn render(&self) -> Option<Workload> {
        pub use render::*;

        (
            render_start.tag(path!({self}::{EnginePhase::Render}::render_start)),
            render_end.tag(path!({self}::{EnginePhase::Render}::render_end)),
        )
            .into_sequential_workload()
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