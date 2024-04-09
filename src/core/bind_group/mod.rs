use std::any::Any;

use super::renderer::{BufferHandle, Renderer, SamplerHandle, TextureHandle};

pub mod camera;
pub mod resolution;
pub mod texture;
pub trait BindGroup: Any + BindGroupToAny {
    fn get_data(&self) -> Box<[u8]>;
    fn get_layout(&self, renderer: &mut Renderer) -> BindGroupLayout;
}

pub trait BindGroupToAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> BindGroupToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Clone)]
pub enum BindGroupLayoutEntry {
    Data(BufferHandle),
    Texture(TextureHandle),
    Sampler(SamplerHandle),
}

#[derive(Clone)]
pub struct BindGroupLayout {
    pub entries: Vec<BindGroupLayoutEntry>,
}
