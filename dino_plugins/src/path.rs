use std::any::Any;
use std::fmt;
use std::hash::{Hash, Hasher};
use shipyard::Label;
use crate::ident::Ident;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct IdentPath(Box<[&'static Ident]>);

impl IdentPath {
    pub fn new(path: &[&'static Ident]) -> Option<Self> {
        if !path.is_empty() {
            Some(Self(path.to_vec().into_boxed_slice()))
        } else {
            None
        }
    }

    pub const fn path(&self) -> &[&'static Ident] {
        &self.0
    }
}

impl fmt::Display for IdentPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter();

        let first = iter.next().expect("must have at least one segment");

        write!(f, "{first}")?;

        for ident in iter {
            write!(f, "::{ident}")?;
        }

        Ok(())
    }
}

impl Label for IdentPath {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn Label) -> bool {
        other.as_any()
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        Self::hash(self, &mut state)
    }

    fn dyn_clone(&self) -> Box<dyn Label> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[macro_export]
macro_rules! path {
    ($($segment:tt)::+) => {{
        $crate::path::IdentPath::new(&[
            $(
                path!(@internal $segment)
            ),*
        ]).expect("macro forces at least one ident")
    }};
    (@internal $segment:ident) => {
        <_ as ::strck::IntoCk>::ck(&stringify!($segment)).expect("invalid ident")
    };
    (@internal { $expr:expr }) => {{
        use $crate::Identifiable;

        $expr.identifier()
    }};
    () => {
        compile_error!("at least one segment required")
    }
}