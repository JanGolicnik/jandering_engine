use wgpu::util::DeviceExt;

use crate::renderer::Renderer;

use super::{Instance, InstanceRaw, Object, VertexRaw};

pub fn triangle(renderer: &Renderer, instances: Vec<Instance>) -> Object {
    let vertices = vec![
        VertexRaw {
            position: [0.0, 1.0, 0.0],
            uv: [1.0, 1.0],
        },
        VertexRaw {
            position: [1.0, -1.0, 0.0],
            uv: [1.0, 0.0],
        },
        VertexRaw {
            position: [-1.0, -1.0, 0.0],
            uv: [0.0, 1.0],
        },
    ];

    let instance_data: Vec<InstanceRaw> = instances.iter().map(|e| e.to_raw()).collect();

    let indices = vec![0, 2, 1];

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
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

    Object {
        vertices,
        indices,
        instances,
        instance_data,
        render_data: Some(super::ObjectRenderData {
            vertex_buffer,
            index_buffer,
            instance_buffer,
        }),
        shader: 0,
    }
}

pub fn quad(renderer: &Renderer, instances: Vec<Instance>) -> Object {
    let vertices = vec![
        VertexRaw {
            position: [-1.0, -1.0, 0.0],
            uv: [0.0, 0.0],
        },
        VertexRaw {
            position: [1.0, 1.0, 0.0],
            uv: [1.0, 1.0],
        },
        VertexRaw {
            position: [1.0, -1.0, 0.0],
            uv: [1.0, 0.0],
        },
        VertexRaw {
            position: [-1.0, 1.0, 0.0],
            uv: [0.0, 1.0],
        },
    ];

    let instance_data: Vec<InstanceRaw> = instances.iter().map(|e| e.to_raw()).collect();

    let indices = vec![0, 2, 1, 0, 1, 3];

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
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

    Object {
        vertices,
        indices,
        instances,
        instance_data,
        render_data: Some(super::ObjectRenderData {
            vertex_buffer,
            index_buffer,
            instance_buffer,
        }),
        shader: 0,
    }
}
