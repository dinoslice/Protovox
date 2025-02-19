use crate::ident::Ident;

pub mod engine;
pub mod ident;
pub mod path;

pub trait DinoPlugin<I, P: Identifiable<I>, W, M: Identifiable<I>> {
    fn instructions(&self, #[allow(unused_variables)] phase: P) -> Option<W> {
        None
    }

    fn metadata(&self) -> M;

    fn identifier(&self) -> I {
        self.metadata().identifier()
    }
}

pub trait Identifiable<I = &'static Ident> {
    fn identifier(&self) -> I;
}