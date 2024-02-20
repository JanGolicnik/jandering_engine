use jandering_engine::bind_group::{BindGroup, BindGroupRenderData, BindGroupWriteData};
use jandering_engine::renderer::Renderer;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FactorBindGroupUniform {
    pub factor: f32,
    padding: [f32; 3],
}

pub struct FactorBindGroup {
    pub uniform: FactorBindGroupUniform,
    render_data: BindGroupRenderData,
}

impl BindGroup for FactorBindGroup {
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

impl FactorBindGroup {
    pub fn new(renderer: &Renderer) -> Self {
        let uniform = FactorBindGroupUniform {
            factor: 0.0,
            padding: [0.0; 3],
        };

        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("FactorBindGroup Buffer"),
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
                    label: Some("FactorBindGroup_bind_group_layout"),
                });

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("FactorBindGroup_bind_group"),
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
