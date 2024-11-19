use crate::{
    renderer::{BufferHandle, BufferType},
    texture::TextureFormat,
};

use super::renderer::{SamplerHandle, TextureHandle};

#[derive(Clone, Default)]
pub enum TextureSampleType {
    #[default]
    Filterable,
    NonFilterable,
    Depth,
}

#[derive(Clone, Copy)]
pub enum StorageTextureAccessType {
    Read,
    Write,
    ReadWrite,
}

#[derive(Clone)]
pub enum BindGroupLayoutDescriptorEntry {
    Data {
        is_uniform: bool,
    },
    Texture {
        sample_type: TextureSampleType,
    },
    StorageTexture {
        access_type: StorageTextureAccessType,
        format: TextureFormat,
    },
    Sampler {
        sampler_type: SamplerType,
    },
}

#[derive(Clone)]
pub struct BindGroupLayoutDescriptor {
    pub entries: Vec<BindGroupLayoutDescriptorEntry>,
}

#[derive(Clone)]
pub enum BindGroupLayoutEntry {
    Data(BufferHandle),
    Texture {
        handle: TextureHandle,
        sample_type: TextureSampleType,
    },

    StorageTexture {
        handle: TextureHandle,
        access_type: StorageTextureAccessType,
        format: TextureFormat,
    },
    Sampler {
        handle: SamplerHandle,
        sampler_type: SamplerType,
    },
}

impl From<BindGroupLayoutEntry> for BindGroupLayoutDescriptorEntry {
    fn from(val: BindGroupLayoutEntry) -> Self {
        match val {
            BindGroupLayoutEntry::Data(buffer_handle) => BindGroupLayoutDescriptorEntry::Data {
                is_uniform: matches!(buffer_handle.buffer_type, BufferType::Uniform),
            },
            BindGroupLayoutEntry::Texture { sample_type, .. } => {
                BindGroupLayoutDescriptorEntry::Texture { sample_type }
            }
            BindGroupLayoutEntry::StorageTexture {
                access_type,
                format,
                ..
            } => BindGroupLayoutDescriptorEntry::StorageTexture {
                access_type,
                format,
            },
            BindGroupLayoutEntry::Sampler { sampler_type, .. } => {
                BindGroupLayoutDescriptorEntry::Sampler { sampler_type }
            }
        }
    }
}

#[derive(Clone)]
pub struct BindGroupLayout {
    pub entries: Vec<BindGroupLayoutEntry>,
}

#[derive(Clone)]
pub enum SamplerType {
    Filtering,
    NonFiltering,
    Comparison,
}
