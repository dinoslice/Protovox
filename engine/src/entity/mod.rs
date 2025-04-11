use crate::components::Transform;
use shipyard::{AllStoragesViewMut, Component, IntoWorkload, Unique, Workload};
use std::ops::Index;

pub mod model;

#[derive(Component)]
pub struct ModelView(pub String);

pub fn initialize(mut storages: AllStoragesViewMut) {
    let model = ModelView("toy_car".to_string());
    let transform = Transform {
        position: [0.0, 50.0, 0.0].into(),
        scale: [100.0, 100.0, 100.0].into(),
        rotation: [0.0, 0.0, 0.0].into(),
    };

    storages.add_entity((model, transform));
}

pub fn update_entities() -> Workload {
    (
        model::update_dirty_models,
    ).into_sequential_workload()
}