use shipyard::Component;

#[derive(Component, Debug)]
pub struct EventBus<T: Sync + Send + 'static>(pub Vec<T>);

impl<T: Sync + Send + 'static> Default for EventBus<T> {
    fn default() -> Self {
        Self(Vec::default())
    }
}