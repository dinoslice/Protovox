pub trait ResourceType: Send + Sync {
    fn resource_name() -> &'static str;
}