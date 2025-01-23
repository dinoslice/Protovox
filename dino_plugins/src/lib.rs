pub mod engine;
pub mod ident;

pub trait DinoPlugin<P, I, M> {
    fn instructions(&self, phase: P) -> Option<I> {
        None
    }

    fn metadata(&self) -> M;
}