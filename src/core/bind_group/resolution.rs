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
}

impl ResolutionBindGroup {
    pub fn new(renderer: &Renderer) -> Self {
        Self {
            data: ResolutionBindGroupData {
                resolution: [renderer.width(), renderer.height()],
                #[cfg(target_arch = "wasm32")]
                padding: [0.0; 2],
            },
        }
    }

    pub fn update(&mut self, resolution: UVec2) {
        self.data.resolution = resolution.into();
    }

    #[allow(dead_code)]
    fn layout() -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data],
        }
    }
}
