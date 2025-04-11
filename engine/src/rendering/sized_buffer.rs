use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct SizedBuffer {
    pub buffer: wgpu::Buffer,
    pub size: u32,
}

impl SizedBuffer {
    pub fn new(buffer: wgpu::Buffer, size: u32) -> Self {
        Self { buffer, size }
    }
}

impl Deref for SizedBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for SizedBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}