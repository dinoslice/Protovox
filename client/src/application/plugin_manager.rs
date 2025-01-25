use std::ops::Deref;
use shipyard::{IntoWorkload, IntoWorkloadSystem, Workload, WorkloadModificator, World};
use dino_plugins::engine::{DinoEnginePlugin, EnginePhase};
use crate::environment::{is_hosted, is_multiplayer_client};
use crate::networking::server_connection::client_process_network_events_multiplayer;
use crate::networking::server_handler::server_process_network_events;

pub struct PluginManager {
    plugins: Vec<&'static dyn DinoEnginePlugin>
}

impl PluginManager {
    pub fn new() -> Self {
        Self { plugins: vec![] }
    }

    pub fn add(&mut self, plugin: &'static dyn DinoEnginePlugin) {
        self.plugins.push(plugin);
    }

    // builder pattern version
    pub fn with(mut self, plugin: &'static dyn DinoEnginePlugin) -> Self {
        self.add(plugin);
        self
    }

    pub fn build_into(&self, world: &World) {
        let plugins = self.plugins.iter().map(Deref::deref);

        build_startup(plugins.clone())
            .add_to_world(&world)
            .expect("failed to add workload");

        build_update(plugins.clone(), client_process_network_events_multiplayer, server_process_network_events)
            .add_to_world(&world)
            .expect("failed to add workload");

        build_render(plugins.clone())
            .add_to_world(&world)
            .expect("failed to add workload");

        build_shutdown(plugins.clone())
            .add_to_world(&world)
            .expect("failed to add workload");
    }
}

fn build_startup<'a>(plugins: impl IntoIterator<Item = &'a dyn DinoEnginePlugin>) -> Workload {
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

    (early_startup, late_startup)
        .into_sequential_workload()
        .rename("engine::startup")
}

fn build_update<'a, CB, CR: 'static, SB, SR: 'static>(
    plugins: impl IntoIterator<Item = &'a dyn DinoEnginePlugin>,
    client_process: impl IntoWorkloadSystem<CB, CR> + 'static,
    server_process: impl IntoWorkloadSystem<SB, SR> + 'static,
) -> Workload {
    let mut input = Workload::new("engine::input");
    let mut early_update = Workload::new("engine::early_update");
    let mut networking_client_pre_recv = Workload::new("engine::networking_client_pre_recv");
    let mut networking_client_post_recv = Workload::new("engine::networking_client_post_recv");
    let mut networking_server_pre_recv = Workload::new("engine::networking_server_pre_recv");
    let mut networking_server_post_recv = Workload::new("engine::networking_server_post_recv");
    let mut late_update = Workload::new("engine::late_update");

    for plugin in plugins {
        if let Some(w) = plugin.instructions_renamed(EnginePhase::Input) {
            input = input.with_workload(w);
        }

        if let Some(w) = plugin.instructions_renamed(EnginePhase::EarlyUpdate) {
            early_update = early_update.with_workload(w);
        }

        if let Some(w) = plugin.instructions_renamed(EnginePhase::NetworkingClientPreRecv) {
            networking_client_pre_recv = networking_client_pre_recv.with_workload(w);
        }

        if let Some(w) = plugin.instructions_renamed(EnginePhase::NetworkingClientPostRecv) {
            networking_client_post_recv = networking_client_post_recv.with_workload(w);
        }

        if let Some(w) = plugin.instructions_renamed(EnginePhase::NetworkingServerPreRecv) {
            networking_server_pre_recv = networking_server_pre_recv.with_workload(w);
        }

        if let Some(w) = plugin.instructions_renamed(EnginePhase::NetworkingServerPostRecv) {
            networking_server_post_recv = networking_server_post_recv.with_workload(w);
        }

        if let Some(w) = plugin.instructions_renamed(EnginePhase::LateUpdate) {
            late_update = late_update.with_workload(w);
        }
    }

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
        .rename("engine::update")
}

fn build_render<'a>(plugins: impl IntoIterator<Item = &'a dyn DinoEnginePlugin>) -> Workload {
    let mut pre_render = Workload::new("engine::pre_render");
    let mut render = Workload::new("engine::render");
    let mut post_render = Workload::new("engine::post_render");

    for plugin in plugins {
        if let Some(w) = plugin.instructions_renamed(EnginePhase::PreRender) {
            pre_render = pre_render.with_workload(w);
        }

        if let Some(w) = plugin.instructions_renamed(EnginePhase::Render) {
            render = render.with_workload(w);
        }

        if let Some(w) = plugin.instructions_renamed(EnginePhase::PostRender) {
            post_render = post_render.with_workload(w);
        }
    }

    (pre_render, render, post_render)
        .into_sequential_workload()
        .rename("engine::render")
}

fn build_shutdown<'a>(plugins: impl IntoIterator<Item = &'a dyn DinoEnginePlugin>) -> Workload {
    let mut shutdown = Workload::new("engine::shutdown");

    for w in plugins.into_iter()
        .filter_map(|p| p.instructions_renamed(EnginePhase::Shutdown))
    {
        shutdown = shutdown.with_workload(w);
    }

    shutdown
}