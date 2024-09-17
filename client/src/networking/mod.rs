use shipyard::{IntoWorkload, Workload, WorkloadModificator};
use crate::multiplayer::server_connection::process_network_events_multiplayer_client;
use crate::networking::server_socket::process_network_events_system;

pub mod types;
pub mod server_socket;

pub fn update_networking() -> Workload {
    (
        process_network_events_system, // internally, run_if(is_hosted)
        process_network_events_multiplayer_client, // internally, run_if(is_multiplayer_client)
    ).into_workload()
}