use crate::renderer::{Renderer, SamplerHandle, TextureHandle};

use super::{BindGroup, BindGroupLayout, BindGroupLayoutEntry, SamplerType};

pub struct TextureBindGroup {
    pub texture_handle: TextureHandle,
    pub sampler_handle: SamplerHandle,
}

impl BindGroup for TextureBindGroup {
    fn get_data(&self) -> Box<[u8]> {
        Box::new([])
    }

    fn get_layout(&self) -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![
                BindGroupLayoutEntry::Texture {
                    handle: self.texture_handle,
                    depth: false,
                },
                BindGroupLayoutEntry::Sampler {
                    handle: self.sampler_handle,
                    sampler_type: SamplerType::Filtering,
                },
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
}
