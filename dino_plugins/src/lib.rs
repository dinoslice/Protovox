use crate::ident::Ident;

pub mod engine;
pub mod ident;

pub trait DinoPlugin<P: Identifiable, I, M: Identifiable> {
    fn instructions(&self, phase: P) -> Option<I> {
        None
    }

    fn metadata(&self) -> M;
}

pub trait Identifiable<I = &'static Ident> {
    fn identifier(&self) -> I;
}