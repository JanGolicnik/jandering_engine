use wgpu::util::DeviceExt;

use crate::bind_group::BindGroup;

use super::{BindGroupRenderData, BindGroupWriteData};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ResolutionBindGroupUniform {
    resolution: [f32; 2],
    #[cfg(target_arch = "wasm32")]
    padding: [f32; 2],
}

pub struct ResolutionBindGroup {
    uniform: ResolutionBindGroupUniform,
    render_data: BindGroupRenderData,
}

impl BindGroup for ResolutionBindGroup {
    fn get_bind_group_layout(&self) -> Option<&wgpu::BindGroupLayout> {
        Some(&self.render_data.bind_group_layout)
    }

    fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        Some(&self.render_data.bind_group)
    }

    fn write(&mut self, data: &BindGroupWriteData) {
        self.uniform.resolution[0] = data.config.width as f32;
        self.uniform.resolution[1] = data.config.height as f32;

        data.queue.write_buffer(
            &self.render_data.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}

impl ResolutionBindGroup {
    pub fn new(renderer: &crate::renderer::Renderer) -> Self {
        let uniform = ResolutionBindGroupUniform {
            resolution: [renderer.config.width as f32, renderer.config.height as f32],
            #[cfg(target_arch = "wasm32")]
            padding: [0.0; 2],
        };

        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Resolution Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("resolution_bind_group_layout"),
                });

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("resolution_bind_group"),
            });

        Self {
            uniform,
            render_data: BindGroupRenderData {
                buffer,
                bind_group_layout,
                bind_group,
            },
        }
    }
}
