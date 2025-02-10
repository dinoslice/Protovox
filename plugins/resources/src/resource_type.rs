use crate::{Registry, ResourceKey};

pub trait ResourceType: Send + Sync + Default {
    fn resource_name() -> &'static str;

    fn is_valid(&self, _: &mut Registry) -> bool {
        true
    }

    fn default_resource() -> ResourceKey<Self> where Self: Sized;
}