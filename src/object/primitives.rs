use wgpu::util::DeviceExt;

use crate::renderer::Renderer;

use super::{InstanceRaw, Object, VertexRaw};

pub fn triangle(renderer: &Renderer, instances: &[InstanceRaw]) -> Object {
    let vertices = vec![
        VertexRaw {
            position: [0.0, 1.0, 0.0],
            color: [0.0, 1.0, 0.0],
        },
        VertexRaw {
            position: [1.0, -1.0, 0.0],
            color: [1.0, 0.0, 0.0],
        },
        VertexRaw {
            position: [-1.0, -1.0, 0.0],
            color: [0.0, 0.0, 1.0],
        },
    ];

    let indices = vec![0, 1, 2];

    let vertex_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
    let index_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
    let instance_buffer = renderer
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

    Object {
        vertices,
        indices,
        instances: Vec::new(),
        instance_data: Vec::new(),
        render_data: Some(super::ObjectRenderData {
            vertex_buffer,
            index_buffer,
            instance_buffer,
        }),
        shader: 0,
    }
}
