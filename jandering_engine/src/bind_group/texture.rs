use crate::renderer::{Renderer, SamplerHandle, TextureHandle};

use super::{BindGroup, BindGroupLayout, BindGroupLayoutEntry, SamplerType};

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
                    sample_type: super::TextureSampleType::default(),
                },
                BindGroupLayoutEntry::Sampler {
                    handle: self.sampler_handle,
                    sampler_type: SamplerType::Filtering,
                },
            ],
        }
    }

    fn get_layout_descriptor() -> super::BindGroupLayoutDescriptor
    where
        Self: Sized,
    {
        super::BindGroupLayoutDescriptor {
            entries: vec![
                super::BindGroupLayoutDescriptorEntry::Texture {
                    sample_type: Default::default(),
                },
                super::BindGroupLayoutDescriptorEntry::Sampler {
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
                    sample_type: super::TextureSampleType::NonFilterable,
                },
                BindGroupLayoutEntry::Sampler {
                    handle: self.sampler_handle,
                    sampler_type: SamplerType::NonFiltering,
                },
            ],
        }
    }

    fn get_layout_descriptor() -> super::BindGroupLayoutDescriptor
    where
        Self: Sized,
    {
        super::BindGroupLayoutDescriptor {
            entries: vec![
                super::BindGroupLayoutDescriptorEntry::Texture {
                    sample_type: super::TextureSampleType::NonFilterable,
                },
                super::BindGroupLayoutDescriptorEntry::Sampler {
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
