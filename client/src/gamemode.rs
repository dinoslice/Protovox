use shipyard::Component;

#[derive(Clone, Component, Debug, Default, Eq, PartialEq)]
pub enum Gamemode {
    #[default]
    Survival,
    Spectator,
}