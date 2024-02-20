use wgpu::util::DeviceExt;

use super::{Object, VertexRaw};
use crate::renderer::Renderer;
use crate::types::*;

pub fn triangle<T>(renderer: &Renderer, instances: Vec<T>) -> Object<T>
where
    T: bytemuck::Pod,
{
    let vertices = vec![
        VertexRaw {
            position: Vec3::new(0.0, 1.0, 0.0),
            uv: Vec2::new(1.0, 1.0),
        },
        VertexRaw {
            position: Vec3::new(1.0, -1.0, 0.0),
            uv: Vec2::new(1.0, 0.0),
        },
        VertexRaw {
            position: Vec3::new(-1.0, -1.0, 0.0),
            uv: Vec2::new(0.0, 1.0),
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
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

    Object {
        vertices,
        indices,
        instances,
        render_data: Some(super::ObjectRenderData {
            vertex_buffer,
            index_buffer,
            instance_buffer,
        }),
    }
}

pub fn quad<T>(renderer: &Renderer, instances: Vec<T>) -> Object<T>
where
    T: bytemuck::Pod,
{
    let vertices = vec![
        VertexRaw {
            position: Vec3::new(-1.0, -1.0, 0.0),
            uv: Vec2::new(0.0, 0.0),
        },
        VertexRaw {
            position: Vec3::new(1.0, 1.0, 0.0),
            uv: Vec2::new(1.0, 1.0),
        },
        VertexRaw {
            position: Vec3::new(1.0, -1.0, 0.0),
            uv: Vec2::new(1.0, 0.0),
        },
        VertexRaw {
            position: Vec3::new(-1.0, 1.0, 0.0),
            uv: Vec2::new(0.0, 1.0),
        },
    ];

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
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

    Object {
        vertices,
        indices,
        instances,
        render_data: Some(super::ObjectRenderData {
            vertex_buffer,
            index_buffer,
            instance_buffer,
        }),
    }
}
