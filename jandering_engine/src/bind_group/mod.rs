use std::any::Any;

use crate::renderer::{BufferHandle, BufferType};

use super::renderer::{SamplerHandle, TextureHandle};

pub mod camera;
pub mod resolution;
pub mod texture;

pub trait BindGroup: Any + BindGroupToAny {
    fn get_layout_descriptor() -> BindGroupLayoutDescriptor
    where
        Self: Sized;
    fn get_layout(&self) -> BindGroupLayout;
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
pub enum BindGroupLayoutDescriptorEntry {
    Data { is_uniform: bool },
    Texture { depth: bool },
    Sampler { sampler_type: SamplerType },
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
        depth: bool,
    },
    Sampler {
        handle: SamplerHandle,
        sampler_type: SamplerType,
    },
}

impl Into<BindGroupLayoutDescriptorEntry> for BindGroupLayoutEntry {
    fn into(self) -> BindGroupLayoutDescriptorEntry {
        match self {
            BindGroupLayoutEntry::Data(buffer_handle) => BindGroupLayoutDescriptorEntry::Data {
                is_uniform: matches!(buffer_handle.buffer_type, BufferType::Uniform),
            },
            BindGroupLayoutEntry::Texture { depth, .. } => {
                BindGroupLayoutDescriptorEntry::Texture { depth }
            }
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
