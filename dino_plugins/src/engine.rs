use shipyard::Workload;
use crate::DinoPlugin;
use strck::IntoCk;
use crate::{DinoPlugin, Identifiable};
use crate::ident::Ident;

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

impl Identifiable for EnginePhase {
    fn identifier(&self) -> &'static Ident {
        let res = match self {
            EnginePhase::Startup => "startup".ck(),
            EnginePhase::Input => "input".ck(),
            EnginePhase::EarlyUpdate => "early_update".ck(),
            EnginePhase::Render => "render".ck(),
            EnginePhase::Shutdown => "shutdown".ck(),
        };

        res.expect("valid identifier")
    }
}

pub struct EnginePluginMetadata {
    pub name: &'static Ident,
    pub version: &'static str, // TODO: ues semver crate?
}

impl Identifiable for EnginePluginMetadata {
    fn identifier(&self) -> &'static Ident {
        self.name
    }
}

    fn startup(&self) -> Option<Workload> {
        None
    }

    fn input(&self) -> Option<Workload> {
        None
    }

    fn early_update(&self) -> Option<Workload> {
        None
    }

    fn render(&self) -> Option<Workload> {
        None
    }

    fn shutdown(&self) -> Option<Workload> {
        None
    }

    fn metadata(&self) -> EnginePluginMetadata;
}

impl<T: DinoEnginePlugin> DinoPlugin<EnginePhase, Workload, EnginePluginMetadata> for T {
    fn instructions(&self, phase: EnginePhase) -> Option<Workload> {
        match phase {
            EnginePhase::Startup => self.startup(),
            EnginePhase::Input => self.input(),
            EnginePhase::EarlyUpdate => self.early_update(),
            EnginePhase::Render => self.render(),
            EnginePhase::Shutdown => self.shutdown(),
        }
    }

    fn metadata(&self) -> EnginePluginMetadata {
        DinoEnginePlugin::metadata(self)
    }
}