use crate::{Registry, ResourceKey};
use crate::key::ResourceIdent;

pub trait ResourceType: Send + Sync {
    fn resource_name()  -> &'static str where Self: Sized;

    fn is_valid(&self, _: &mut Registry) -> bool {
        true
    }

    fn default_resource() -> ResourceIdent<Self> where Self: Sized;
}