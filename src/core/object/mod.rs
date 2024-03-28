use std::ops::Range;

use crate::types::*;

use super::renderer::{Buffer, RenderPass, Renderer};

pub mod primitives;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

pub struct ObjectRenderData {
    pub vertex_buffer: Buffer,
    //
    pub index_buffer: Buffer,
    //
    pub instance_buffer: Buffer,
}

pub struct Object<T> {
    pub vertices: Vec<Vertex>,
    //
    pub indices: Vec<u32>,
    //
    pub instances: Vec<T>,
    //
    pub render_data: ObjectRenderData,

    previous_instances_len: usize,
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub model: Mat4,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            model: Mat4::IDENTITY,
        }
    }
}

impl Instance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }

    pub fn with_position(pos: Vec3) -> Self {
        let model = Mat4::from_translation(pos);
        Self { model }
    }
}

impl<T: bytemuck::Pod> Object<T> {
    pub fn update(&mut self, renderer: &mut Renderer) {
        if self.previous_instances_len != self.instances.len() {
            self.render_data.instance_buffer =
                renderer.create_vertex_buffer(bytemuck::cast_slice(&self.instances));
            self.previous_instances_len = self.instances.len();
        } else {
            renderer.write_buffer(
                &self.render_data.instance_buffer,
                bytemuck::cast_slice(&self.instances),
            );
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct D2Instance {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
}

impl Default for D2Instance {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            scale: Vec2::ONE,
            rotation: 0.0,
        }
    }
}

impl D2Instance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<D2Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

pub trait Renderable {
    // fn bind(&self, render_pass: Box<dyn RenderPass>, range: Range<u32>);

    fn num_instances(&self) -> u32;
}

impl<T: std::any::Any> Renderable for Object<T> {
    // fn bind<'a, R: RenderPass<'a>>(&'a self, mut render_pass: R, range: Range<u32>) {
    //     // render_pass.draw_instanced(
    //     //     0..self.indices.len() as u32,
    //     //     range,
    //     //     &self.render_data.vertex_buffer,
    //     //     &self.render_data.instance_buffer,
    //     //     &self.render_data.index_buffer,
    //     // );
    // }

    fn num_instances(&self) -> u32 {
        self.previous_instances_len as u32
    }
}
