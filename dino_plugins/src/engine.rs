use shipyard::Workload;
use crate::DinoPlugin;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum EnginePhase { // TODO: eventually add all these phases
    Startup,
    Input,
    EarlyUpdate,
    // NetworkingClientPreRecv,
    // NetworkingClientPostRecv,
    // NetworkingServerPreRecv,
    // NetworkingServerPostRecv,
    // LateUpdate,
    // PreRender,
    Render,
    // RenderUi,
    // PostRender,
    Shutdown,
}

pub struct EnginePluginMetadata {
    name: &'static str,
}

pub trait DinoEnginePlugin {
    fn startup() -> Option<Workload> {
        None
    }

    fn input() -> Option<Workload> {
        None
    }

    fn early_update() -> Option<Workload> {
        None
    }

    fn render() -> Option<Workload> {
        None
    }

    fn shutdown() -> Option<Workload> {
        None
    }

    fn metadata() -> EnginePluginMetadata;
}

impl<T: DinoEnginePlugin> DinoPlugin<EnginePhase, Workload, EnginePluginMetadata> for T {
    fn instructions(phase: EnginePhase) -> Option<Workload> {
        match phase {
            EnginePhase::Startup => T::startup(),
            EnginePhase::Input => T::input(),
            EnginePhase::EarlyUpdate => T::early_update(),
            EnginePhase::Render => T::render(),
            EnginePhase::Shutdown => T::shutdown(),
        }
    }

    fn metadata() -> EnginePluginMetadata {
        T::metadata()
    }
}