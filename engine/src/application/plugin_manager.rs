use std::ops::Deref;
use itertools::Itertools;
use shipyard::{IntoWorkload, IntoWorkloadSystem, Workload, WorkloadModificator, World};
use dino_plugins::engine::{DinoEnginePlugin, EnginePhase};
use dino_plugins::{path, Identifiable};
use dino_plugins::ident::Ident;
use crate::environment::{is_hosted, is_multiplayer_client};
use crate::networking::server_connection::client_process_network_events_multiplayer;
use crate::networking::server_handler::server_process_network_events;

#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<&'static dyn DinoEnginePlugin>
}

impl PluginManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, plugin: &'static dyn DinoEnginePlugin) {
        self.plugins.push(plugin);
    }

    // builder pattern version
    pub fn with(mut self, plugin: &'static dyn DinoEnginePlugin) -> Self {
        self.add(plugin);
        self
    }

    pub fn build_into(&self, world: &World, ident: impl Identifiable + Clone) {
        let plugins = self.plugins.iter().map(Deref::deref);

        build_startup(plugins.clone(), ident.clone())
            .add_to_world(world)
            .expect("failed to add workload");

        build_update(plugins.clone(), ident.clone(), client_process_network_events_multiplayer, server_process_network_events)
            .add_to_world(world)
            .expect("failed to add workload");

        build_render(plugins.clone(), ident.clone())
            .add_to_world(world)
            .expect("failed to add workload");

        build_shutdown(plugins, ident)
            .add_to_world(world)
            .expect("failed to add workload");
    }

    pub fn first_unmet_dependency(&self) -> Option<&'static Ident> { // TODO: use something like type_id?
        let loaded = self.plugins
            .iter()
            .map(|pl| pl.identifier())
            .collect_vec();

        self.plugins
            .iter()
            .flat_map(|pl|
                pl.metadata()
                    .dependencies
                    .iter()
                    .map(|pl| pl.identifier())
            )
            .find(|id| !loaded.contains(id))
    }
}

fn build_helper<'a>(plugins: impl IntoIterator<Item = &'a dyn DinoEnginePlugin>, ident: impl Identifiable, phase: EnginePhase) -> Workload {
    plugins.into_iter()
        .filter_map(|p| p.instructions_renamed(phase))
        .fold(
            Workload::new(path!({ident}::{phase})),
            |build, w| build.with_workload(w)
        )
}

fn build_startup<'a>(plugins: impl IntoIterator<Item = &'a dyn DinoEnginePlugin> + Clone, ident: impl Identifiable + Clone) -> Workload {
    let early_startup = build_helper(plugins.clone(), ident.clone(), EnginePhase::EarlyStartup);
    let late_startup = build_helper(plugins, ident.clone(), EnginePhase::LateStartup);

    (early_startup, late_startup)
        .into_sequential_workload()
        .rename(path!({ident}::startup))
}

fn build_update<'a, CB, CR: 'static, SB, SR: 'static>(
    plugins: impl IntoIterator<Item = &'a dyn DinoEnginePlugin> + Clone,
    ident: impl Identifiable + Clone,
    client_process: impl IntoWorkloadSystem<CB, CR> + 'static,
    server_process: impl IntoWorkloadSystem<SB, SR> + 'static,
) -> Workload {
    let input = build_helper(plugins.clone(), ident.clone(), EnginePhase::Input);
    let early_update = build_helper(plugins.clone(), ident.clone(), EnginePhase::EarlyUpdate);
    let networking_client_pre_recv = build_helper(plugins.clone(), ident.clone(), EnginePhase::NetworkingClientPreRecv);
    let networking_client_post_recv = build_helper(plugins.clone(), ident.clone(), EnginePhase::NetworkingClientPostRecv);
    let networking_server_pre_recv = build_helper(plugins.clone(), ident.clone(), EnginePhase::NetworkingServerPreRecv);
    let networking_server_post_recv = build_helper(plugins.clone(), ident.clone(), EnginePhase::NetworkingServerPostRecv);
    let late_update = build_helper(plugins, ident.clone(), EnginePhase::LateUpdate);

    (
        input,
        early_update,
        (
            networking_client_pre_recv,
            client_process,
            networking_client_post_recv,
        ).into_sequential_workload().run_if(is_multiplayer_client),
        (
            networking_server_pre_recv,
            server_process,
            networking_server_post_recv,
        ).into_sequential_workload().run_if(is_hosted),
        late_update,
    ).into_sequential_workload()
        .rename(path!({ident}::update))
}

fn build_render<'a>(plugins: impl IntoIterator<Item = &'a dyn DinoEnginePlugin> + Clone, ident: impl Identifiable + Clone) -> Workload {
    let pre_render = build_helper(plugins.clone(), ident.clone(), EnginePhase::PreRender);
    let render = build_helper(plugins.clone(), ident.clone(), EnginePhase::Render);
    let post_render = build_helper(plugins, ident.clone(), EnginePhase::PostRender);

    (pre_render, render, post_render)
        .into_sequential_workload()
        .rename(path!({ident}::render))
}

fn build_shutdown<'a>(plugins: impl IntoIterator<Item = &'a dyn DinoEnginePlugin> + Clone, ident: impl Identifiable) -> Workload {
    build_helper(plugins.clone(), ident, EnginePhase::Shutdown)
}