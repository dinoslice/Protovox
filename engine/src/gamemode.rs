use shipyard::{Component, IntoIter, View};
use crate::components::LocalPlayer;

#[derive(Clone, Component, Debug, Default, Eq, PartialEq)]
pub enum Gamemode {
    #[default]
    Survival,
    Spectator,
}

pub fn local_player_is_gamemode_survival(v_local_player: View<LocalPlayer>, v_gamemode: View<Gamemode>) -> bool {
    let (_, gamemode) = (&v_local_player, &v_gamemode)
        .iter()
        .next()
        .expect("local player should have gamemode");
    
    *gamemode == Gamemode::Survival
}

pub fn local_player_is_gamemode_spectator(v_local_player: View<LocalPlayer>, v_gamemode: View<Gamemode>) -> bool {
    let (_, gamemode) = (&v_local_player, &v_gamemode)
        .iter()
        .next()
        .expect("local player should have gamemode");

    *gamemode == Gamemode::Spectator
}