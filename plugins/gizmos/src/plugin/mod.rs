use shipyard::{IntoWorkload, SystemModificator, Workload, WorkloadModificator};
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePhase, EnginePluginMetadata};
use dino_plugins::path;
use engine::VoxelEngine;
use line_render_state::initialize_line_gizmos_render_state;
use settings::read_settings;
use update::{decompose_box_gizmos, process_line_gizmos};
use crate::plugin::render::render_line_gizmos;

pub mod line_render_state;
pub mod settings;
pub mod vertex;
mod render;
mod update;

pub struct GizmosPlugin;

impl DinoEnginePlugin for GizmosPlugin {
    fn early_startup(&self) -> Option<Workload> {
        (
            read_settings,
            initialize_line_gizmos_render_state,
        ).into_sequential_workload()
            .after_all(path!({VoxelEngine}::{EnginePhase::EarlyStartup}))
            .into()
    }

    fn late_update(&self) -> Option<Workload> {
        (
            process_line_gizmos,
            decompose_box_gizmos,
        ).into_workload()
            .into()
    }

    fn render(&self) -> Option<Workload> {
        (
            render_line_gizmos
                .after_all(path!({VoxelEngine}::{EnginePhase::Render}::render_world))
                .before_all(path!({VoxelEngine}::{EnginePhase::Render}::render_egui)),
        ).into_workload()
            .into()
    }

    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "gizmos".ck().expect("valid identifier"),
            version: env!("CARGO_PKG_VERSION"),
            dependencies: &[&VoxelEngine]
        }
    }
}