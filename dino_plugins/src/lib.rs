pub trait DinoPlugin<P, I, M> {
    fn instructions(phase: P) -> Option<I> {
        None
    }

    fn metadata() -> M;
}