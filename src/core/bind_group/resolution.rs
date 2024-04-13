use crate::{core::renderer::Renderer, types::UVec2};

use super::{BindGroup, BindGroupLayout, BindGroupLayoutEntry};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]

struct ResolutionBindGroupData {
    resolution: [u32; 2],
    #[cfg(target_arch = "wasm32")]
    padding: [f32; 2],
}

pub struct ResolutionBindGroup {
    data: ResolutionBindGroupData,
}

impl BindGroup for ResolutionBindGroup {
    fn get_data(&self) -> Box<[u8]> {
        bytemuck::cast_slice(&[self.data]).into()
    }

    fn get_layout(&self, renderer: &mut dyn Renderer) -> BindGroupLayout {
        let buffer_handle = renderer.create_uniform_buffer(&self.get_data());
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data(buffer_handle)],
        }
    }
}

impl ResolutionBindGroup {
    pub fn new(renderer: &dyn Renderer) -> Self {
        Self {
            data: ResolutionBindGroupData {
                resolution: [renderer.size().x, renderer.size().y],
                #[cfg(target_arch = "wasm32")]
                padding: [0.0; 2],
            },
        }
    }

    pub fn update(&mut self, resolution: UVec2) {
        self.data.resolution = resolution.into();
    }
}
