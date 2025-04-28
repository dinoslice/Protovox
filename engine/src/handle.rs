use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait AsHandle {
    fn as_handle(&self) -> Handle<Self> where Self: Sized;
}

impl<T: Sized> AsHandle for T {
    fn as_handle(&self) -> Handle<Self> {
        Handle::new(self)
    }
}

#[derive(Copy, Clone)]
pub struct Handle<'a, T> {
    _inner: &'a T,
}

impl<'a, T> Handle<'a, T> {
    pub fn new(_inner: &'a T) -> Self {
        Self { _inner }
    }
}

impl<T> Deref for Handle<'_, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self._inner
    }
}

#[derive(Clone)]
pub struct HandleMut<T> {
    _inner: Arc<RwLock<T>>,
}

impl<T> HandleMut<T> {
    pub fn new(_inner: Arc<RwLock<T>>) -> Self {
        Self { _inner }
    }
    
    pub fn get(&self) -> RwLockReadGuard<'_, T> {
        self._inner.read().unwrap()
    }
    
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, T> {
        self._inner.write().unwrap()
    }
}