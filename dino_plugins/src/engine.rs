use shipyard::{Workload, WorkloadModificator};
use strck::IntoCk;
use crate::{path, DinoPlugin, Identifiable};
use crate::ident::Ident;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum EnginePhase {
    EarlyStartup,
    LateStartup,
    Input,
    EarlyUpdate,
    NetworkingClientPreRecv,
    NetworkingClientPostRecv,
    NetworkingServerPreRecv,
    NetworkingServerPostRecv,
    LateUpdate,
    PreRender,
    Render,
    RenderUi,
    PostRender,
    Shutdown,
}

impl Identifiable for EnginePhase {
    fn identifier(&self) -> &'static Ident {
        use EnginePhase as P;

        let res = match self {
            P::EarlyStartup => "early_startup".ck(),
            P::LateStartup => "late_startup".ck(),
            P::Input => "input".ck(),
            P::EarlyUpdate => "early_update".ck(),
            P::NetworkingClientPreRecv => "networking_client_pre_recv".ck(),
            P::NetworkingClientPostRecv => "networking_client_post_recv".ck(),
            P::NetworkingServerPreRecv => "networking_server_pre_recv".ck(),
            P::NetworkingServerPostRecv => "networking_server_post_recv".ck(),
            P::LateUpdate => "late_update".ck(),
            P::PreRender => "pre_render".ck(),
            P::Render => "render".ck(),
            P::RenderUi => "render_ui".ck(),
            P::PostRender => "post_render".ck(),
            P::Shutdown => "shutdown".ck(),
        };

        res.expect("valid identifier")
    }
}

pub struct EnginePluginMetadata {
    pub name: &'static Ident,
    pub version: &'static str, // TODO: use semver crate?
    pub dependencies: &'static [&'static dyn DinoEnginePlugin] // TODO: version information?
}

impl Identifiable for EnginePluginMetadata {
    fn identifier(&self) -> &'static Ident {
        self.name
    }
}

pub trait DinoEnginePlugin: DinoPlugin<&'static Ident, EnginePhase, Workload, EnginePluginMetadata> {
    fn early_startup(&self) -> Option<Workload> {
        None
    }

    fn late_startup(&self) -> Option<Workload> {
        None
    }

    fn input(&self) -> Option<Workload> {
        None
    }

    fn early_update(&self) -> Option<Workload> {
        None
    }

    fn networking_client_pre_recv(&self) -> Option<Workload> {
        None
    }

    fn networking_client_post_recv(&self) -> Option<Workload> {
        None
    }

    fn networking_server_pre_recv(&self) -> Option<Workload> {
        None
    }

    fn networking_server_post_recv(&self) -> Option<Workload> {
        None
    }

    fn late_update(&self) -> Option<Workload> {
        None
    }

    fn pre_render(&self) -> Option<Workload> {
        None
    }

    fn render(&self) -> Option<Workload> {
        None
    }

    fn render_ui(&self) -> Option<Workload> {
        None
    }

    fn post_render(&self) -> Option<Workload> {
        None
    }

    fn shutdown(&self) -> Option<Workload> {
        None
    }

    fn instructions_renamed(&self, phase: EnginePhase) -> Option<Workload> {
        self.instructions(phase).map(|workload|
            workload.rename(path!({self}::{phase}))
        )
    }

    fn plugin_metadata(&self) -> EnginePluginMetadata;
}

impl<T: DinoEnginePlugin> DinoPlugin<&'static Ident, EnginePhase, Workload, EnginePluginMetadata> for T {
    fn instructions(&self, phase: EnginePhase) -> Option<Workload> {
        use EnginePhase as P;

        match phase {
            P::EarlyStartup => self.early_startup(),
            P::LateStartup => self.late_startup(),
            P::Input => self.input(),
            P::EarlyUpdate => self.early_update(),
            P::NetworkingClientPreRecv => self.networking_client_pre_recv(),
            P::NetworkingClientPostRecv => self.networking_client_post_recv(),
            P::NetworkingServerPreRecv => self.networking_server_pre_recv(),
            P::NetworkingServerPostRecv => self.networking_server_post_recv(),
            P::LateUpdate => self.late_update(),
            P::PreRender => self.pre_render(),
            P::Render => self.render(),
            P::RenderUi => self.render_ui(),
            P::PostRender => self.post_render(),
            P::Shutdown => self.shutdown(),
        }
    }

    fn metadata(&self) -> EnginePluginMetadata {
        self.plugin_metadata()
    }
}

impl<T: DinoEnginePlugin> Identifiable for T {
    fn identifier(&self) -> &'static Ident {
        self.identifier()
    }
}