use crate::renderer::{Renderer, SamplerHandle, TextureHandle};

use super::{BindGroup, BindGroupLayout, BindGroupLayoutEntry};

pub struct TextureBindGroup {
    pub texture_handle: TextureHandle,
    pub sampler_handle: SamplerHandle,
}

impl BindGroup for TextureBindGroup {
    fn get_data(&self) -> Box<[u8]> {
        Box::new([])
    }

    fn get_layout(&self, _renderer: &mut Renderer) -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![
                BindGroupLayoutEntry::Texture(self.texture_handle),
                BindGroupLayoutEntry::Sampler(self.sampler_handle),
            ],
        }
    }
}

impl TextureBindGroup {
    pub fn new(
        _renderer: &mut Renderer,
        texture_handle: TextureHandle,
        sampler_handle: SamplerHandle,
    ) -> Self {
        Self {
            texture_handle,
            sampler_handle,
        }
    }

    pub fn get_layout() -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![
                BindGroupLayoutEntry::Texture(TextureHandle(0)),
                BindGroupLayoutEntry::Sampler(SamplerHandle(0)),
            ],
        }
    }
}
