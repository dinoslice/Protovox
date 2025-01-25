use std::sync::Arc;
use strck::{Check, Ck, Invariant, ident::unicode};
use crate::Identifiable;

pub struct DinoIdent;

pub type Ident = Ck<DinoIdent>; // TODO: eventually use wrapper ty?
pub type IdentBuf<B = Arc<str>> = Check<DinoIdent, B>;

#[derive(thiserror::Error, Debug)]
pub enum DinoIdentError {
    #[error("Identifier contained invalid character '{0}'")]
    InvalidChar(char),
    #[error("Cannot use wildcard as identifier")]
    Wildcard,
    #[error("Cannot have empty identifier")]
    Empty,
}

impl Invariant for DinoIdent {
    type Error = DinoIdentError;

    fn check(str: &str) -> Result<(), Self::Error> {
        fn check_inner(res: Result<&str, unicode::Error>) -> Result<(), DinoIdentError> {
            match res {
                Ok(str) => match str.chars().find(|&c| !c.is_ascii_alphanumeric() && c != '_') {
                    Some(invalid) => Err(DinoIdentError::InvalidChar(invalid)),
                    None => Ok(())
                }
                Err(unicode::Error::Start(c)) | Err(unicode::Error::Continue(c)) => Err(DinoIdentError::InvalidChar(c)),
                Err(unicode::Error::Empty) => Err(DinoIdentError::Empty),
            }
        }

        match unicode::UnicodeIdent::check(str) {
            Err(unicode::Error::Start('_')) => match str.len() {
                1 => Err(DinoIdentError::Wildcard), // `_` isn't ok
                _ => check_inner(Ok(str.trim_start_matches('_'))),
            }
            res => check_inner(res.map(|_| str)),
        }
    }
}

impl Identifiable for &'static Ident {
    fn identifier(&self) -> &'static Ident {
        self
    }
}