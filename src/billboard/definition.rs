use jandering_engine::{
    object::{ObjectRenderData, Renderable, VertexRaw},
    renderer::Renderer,
};
use wgpu::util::DeviceExt;

use super::{Billboard, BillboardInstance};

impl Renderable for Billboard {
    fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, queue: &mut wgpu::Queue) {
        queue.write_buffer(
            &self.render_data.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );

        render_pass.set_vertex_buffer(0, self.render_data.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.render_data.instance_buffer.slice(..));
        render_pass.set_index_buffer(
            self.render_data.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(
            0..self.indices.len() as u32,
            0,
            0..self.instances.len() as u32,
        );
    }
}

impl Billboard {
    pub fn new(renderer: &Renderer, instances: Vec<BillboardInstance>) -> Self {
        let vertices = vec![
            VertexRaw {
                position: [0.0, 1.0, 0.0],
                uv: [1.0, 1.0],
            },
            VertexRaw {
                position: [-1.0, -1.0, 0.0],
                uv: [1.0, 1.0],
            },
            VertexRaw {
                position: [1.0, -1.0, 0.0],
                uv: [1.0, 1.0],
            },
        ];
        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let indices = vec![0, 1, 2];
        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let instance_buffer =
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
        Self {
            vertices,
            indices,
            instances,
            render_data: ObjectRenderData {
                vertex_buffer,
                index_buffer,
                instance_buffer,
            },
            shader: 0,
        }
    }
}

impl Default for BillboardInstance {
    fn default() -> Self {
        Self {
            size: 1.0,
            position: [0.0; 3],
        }
    }
}

impl BillboardInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<BillboardInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}
