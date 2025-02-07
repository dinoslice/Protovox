use shipyard::{SystemModificator, Workload, WorkloadModificator, WorkloadSystem};
use dino_plugins::engine::EnginePhase;
use dino_plugins::path;
use crate::plugin::EguiSystemsPlugin;

pub trait DuringEgui {
    fn order_egui(self) -> Self;
}

impl DuringEgui for WorkloadSystem {
    fn order_egui(self) -> WorkloadSystem {
        self
            .before_all(path!({EguiSystemsPlugin}::{EnginePhase::Render}::render_start))
            .after_all(path!({EguiSystemsPlugin}::{EnginePhase::Render}::render_end))
    }
}

impl DuringEgui for Workload {
    fn order_egui(self) -> Workload {
        self
            .before_all(path!({EguiSystemsPlugin}::{EnginePhase::Render}::render_start))
            .after_all(path!({EguiSystemsPlugin}::{EnginePhase::Render}::render_end))
    }
}