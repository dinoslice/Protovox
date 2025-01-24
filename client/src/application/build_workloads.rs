use shipyard::{IntoWorkload, Workload, WorkloadModificator};
use dino_plugins::engine::{DinoEnginePlugin, EnginePhase};

pub fn build_startup<'a>(plugins: impl IntoIterator<Item = &'a dyn DinoEnginePlugin>) -> Workload {
    // TODO: macro/func to make the path idents?
    let mut early_startup = Workload::new("engine::early_startup");

    let mut late_startup = Workload::new("engine::late_startup");

    for plugin in plugins {
        if let Some(w) = plugin.instructions_renamed(EnginePhase::EarlyStartup) {
            early_startup = early_startup.with_workload(w);
        }

        if let Some(w) = plugin.instructions_renamed(EnginePhase::LateStartup) {
            late_startup = late_startup.with_workload(w);
        }
    }

    (early_startup, late_startup).into_sequential_workload().rename("engine::startup")
}