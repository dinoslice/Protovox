use std::net::SocketAddr;
use clap::Parser;
use shipyard::AllStoragesView;
use crate::environment::Environment;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, value_name = "SERVER_IP", help = "Run as a client connecting to the server at SERVER_IP")]
    client: Option<String>,
}

pub fn parse_env(storages: AllStoragesView) {
    let args = Args::parse();

    let env = match args.client {
        None => Environment::HostedGame,
        Some(addr) => Environment::MultiplayerClient(
            addr
                .parse::<SocketAddr>()
                .expect("valid server addr")
        )
    };

    tracing::debug!("Set env to {env:?}");

    storages.add_unique(env);
}