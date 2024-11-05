use shipyard::{AllStoragesView, Unique};

#[derive(Unique)]
pub struct ExitRequested;

pub fn request_exit(storages: AllStoragesView) {
    storages.add_unique(ExitRequested);
}