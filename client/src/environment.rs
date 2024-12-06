use std::net::SocketAddr;
use shipyard::{Unique, UniqueView};

#[derive(Unique, PartialEq, Eq, Debug, Hash, strum::Display)]
pub enum Environment {
    DedicatedServer,
    Singleplayer,
    HostedGame,
    #[strum(to_string = "MultiplayerClient of {0}")]
    MultiplayerClient(SocketAddr),
}

pub fn is_dedicated(env: UniqueView<Environment>) -> bool {
    matches!(*env, Environment::DedicatedServer)
}

pub fn is_singleplayer(env: UniqueView<Environment>) -> bool {
    matches!(*env, Environment::Singleplayer)
}

pub fn is_hosted(env: UniqueView<Environment>) -> bool {
    matches!(*env, Environment::HostedGame)
}

pub fn is_multiplayer_client(env: UniqueView<Environment>) -> bool {
    matches!(*env, Environment::MultiplayerClient(..))
}