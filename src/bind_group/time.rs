use wgpu::util::DeviceExt;

use crate::{bind_group::BindGroup, renderer::Renderer};

use super::{BindGroupRenderData, BindGroupWriteData};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TimeBindGroupUniform {
    dt: f32,
    pub time: f32,
    #[cfg(target_arch = "wasm32")]
    padding: [f32; 2],
}

pub struct TimeBindGroup {
    pub uniform: TimeBindGroupUniform,
    render_data: BindGroupRenderData,
}

impl BindGroup for TimeBindGroup {
    fn get_bind_group_layout(&self) -> Option<&wgpu::BindGroupLayout> {
        Some(&self.render_data.bind_group_layout)
    }

    fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        Some(&self.render_data.bind_group)
    }

    fn write(&mut self, data: &BindGroupWriteData) {
        data.queue.write_buffer(
            &self.render_data.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}

impl TimeBindGroup {
    pub fn new(renderer: &Renderer) -> Self {
        let uniform = TimeBindGroupUniform {
            dt: 0.0,
            time: 0.0,
            #[cfg(target_arch = "wasm32")]
            padding: [0.0; 2],
        };

        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Time Buffer"),
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
                    label: Some("time_bind_group_layout"),
                });

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("time_bind_group"),
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

    pub fn update(&mut self, dt: f32) {
        self.uniform.time += dt;
        self.uniform.dt = dt;
    }
}
