use jandering_engine::bind_group::{BindGroup, BindGroupWriteData};
use jandering_engine::renderer::Renderer;
use jandering_engine::types::Vec2;
use wgpu::util::DeviceExt;

use super::TILE_SIZE;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MapBindGroupUniform {
    pub position: Vec2,
    tile_size: f32,
    pub hue: f32,
    padding: [f32; 4],
}

struct MapBindGroupRenderData {
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

pub struct MapBindGroup {
    pub uniform: MapBindGroupUniform,
    render_data: MapBindGroupRenderData,
}

impl BindGroup for MapBindGroup {
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

impl MapBindGroup {
    pub fn new(renderer: &Renderer) -> Self {
        let uniform = MapBindGroupUniform {
            position: Vec2::new(0.0, 0.0),
            tile_size: TILE_SIZE,
            hue: 0.0,
            padding: [0.0; 4],
        };

        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("MapBindGroup Buffer"),
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
                    label: Some("MapBindGroup_bind_group_layout"),
                });

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("MapBindGroup_bind_group"),
            });

        Self {
            uniform,
            render_data: MapBindGroupRenderData {
                buffer,
                bind_group_layout,
                bind_group,
            },
        }
    }
}
