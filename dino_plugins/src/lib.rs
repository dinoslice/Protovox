pub trait DinoPlugin<P, I, M> {
    fn instructions(game_phase: P) -> Option<I>;

    fn metadata() -> M;
}