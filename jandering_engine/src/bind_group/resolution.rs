use crate::{
    renderer::{BufferHandle, Janderer, Renderer},
    types::UVec2,
};

use super::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutDescriptorEntry,
    BindGroupLayoutEntry,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]

struct ResolutionBindGroupData {
    resolution: [u32; 2],
    #[cfg(target_arch = "wasm32")]
    padding: [f32; 2],
}

pub struct ResolutionBindGroup {
    data: ResolutionBindGroupData,
    buffer_handle: BufferHandle,
}

impl BindGroup for ResolutionBindGroup {
    fn get_layout(&self) -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data(self.buffer_handle)],
        }
    }

    fn get_layout_descriptor() -> super::BindGroupLayoutDescriptor
    where
        Self: Sized,
    {
        BindGroupLayoutDescriptor {
            entries: vec![BindGroupLayoutDescriptorEntry::Data { is_uniform: true }],
        }
    }
}

impl ResolutionBindGroup {
    pub fn new(renderer: &mut Renderer, resolution: UVec2) -> Self {
        let data = ResolutionBindGroupData {
            resolution: resolution.into(),
            #[cfg(target_arch = "wasm32")]
            padding: [0.0; 2],
        };
        let buffer_handle = renderer.create_uniform_buffer(bytemuck::cast_slice(&[data]));
        Self {
            data,
            buffer_handle,
        }
    }

    pub fn update(&mut self, resolution: UVec2) {
        self.data.resolution = resolution.into();
    }
}
