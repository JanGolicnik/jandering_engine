use crate::core::renderer::{Renderer, TextureHandle};

use super::{BindGroup, BindGroupLayout, BindGroupLayoutEntry};

pub struct TextureBindGroup {
    pub texture_handle: TextureHandle,
}

impl BindGroup for TextureBindGroup {
    fn get_data(&self) -> Box<[u8]> {
        Box::new([])
    }

    fn get_layout(&self, _renderer: &mut Renderer) -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![
                BindGroupLayoutEntry::Texture(self.texture_handle),
                BindGroupLayoutEntry::Sampler(self.texture_handle),
            ],
        }
    }
}

impl TextureBindGroup {
    pub fn new(_renderer: &mut Renderer, texture_handle: TextureHandle) -> Self {
        Self { texture_handle }
    }

    pub fn get_layout() -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![
                BindGroupLayoutEntry::Texture(TextureHandle(0)),
                BindGroupLayoutEntry::Sampler(TextureHandle(0)),
            ],
        }
    }
}
