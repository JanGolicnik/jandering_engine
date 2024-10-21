use crate::renderer::{Renderer, SamplerHandle, TextureHandle};

use crate::bind_group::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutDescriptorEntry,
    BindGroupLayoutEntry, SamplerType, TextureSampleType,
};

pub struct TextureBindGroup {
    pub texture_handle: TextureHandle,
    pub sampler_handle: SamplerHandle,
}

impl BindGroup for TextureBindGroup {
    fn get_layout(&self) -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![
                BindGroupLayoutEntry::Texture {
                    handle: self.texture_handle,
                    sample_type: TextureSampleType::default(),
                },
                BindGroupLayoutEntry::Sampler {
                    handle: self.sampler_handle,
                    sampler_type: SamplerType::Filtering,
                },
            ],
        }
    }

    fn get_layout_descriptor() -> BindGroupLayoutDescriptor
    where
        Self: Sized,
    {
        BindGroupLayoutDescriptor {
            entries: vec![
                BindGroupLayoutDescriptorEntry::Texture {
                    sample_type: Default::default(),
                },
                BindGroupLayoutDescriptorEntry::Sampler {
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

pub struct UnfilteredTextureBindGroup {
    pub texture_handle: TextureHandle,
    pub sampler_handle: SamplerHandle,
}

impl BindGroup for UnfilteredTextureBindGroup {
    fn get_layout(&self) -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![
                BindGroupLayoutEntry::Texture {
                    handle: self.texture_handle,
                    sample_type: TextureSampleType::NonFilterable,
                },
                BindGroupLayoutEntry::Sampler {
                    handle: self.sampler_handle,
                    sampler_type: SamplerType::NonFiltering,
                },
            ],
        }
    }

    fn get_layout_descriptor() -> BindGroupLayoutDescriptor
    where
        Self: Sized,
    {
        BindGroupLayoutDescriptor {
            entries: vec![
                BindGroupLayoutDescriptorEntry::Texture {
                    sample_type: TextureSampleType::NonFilterable,
                },
                BindGroupLayoutDescriptorEntry::Sampler {
                    sampler_type: SamplerType::NonFiltering,
                },
            ],
        }
    }
}

impl UnfilteredTextureBindGroup {
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
