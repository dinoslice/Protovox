mod startup;
mod update;
mod shutdown;

pub use startup::startup;
pub use update::update;
pub use shutdown::shutdown;

pub use update::process_input;