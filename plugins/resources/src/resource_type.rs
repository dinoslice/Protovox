use crate::Registry;

pub trait ResourceType: Send + Sync {
    fn resource_name() -> &'static str;

    fn is_valid(&self, _: &mut Registry) -> bool {
        true
    }
}