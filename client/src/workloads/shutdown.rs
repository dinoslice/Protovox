use shipyard::{IntoWorkload, Workload};

pub fn shutdown() -> Workload {
    (
        test_print_message
    ).into_sequential_workload()
}

fn test_print_message() {
    tracing::debug!("shutting down!");
}