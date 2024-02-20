use std::any::Any;
use std::ops::Range;

use super::{D2Instance, Instance, Object, Renderable, Vec2};
use crate::types::Mat4;
use crate::{engine::EngineContext, object::VertexRaw, renderer::Renderer};
use cgmath::{SquareMatrix, Zero};

impl VertexRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<VertexRaw>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            model: Mat4::identity(),
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
}

impl Default for D2Instance {
    fn default() -> Self {
        Self {
            position: Vec2::zero(),
            scale: Vec2::zero(),
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

impl<T: bytemuck::Pod> Object<T> {
    pub fn update(&mut self, _context: &EngineContext, renderer: &Renderer) {
        renderer.queue.write_buffer(
            &self.render_data.as_ref().unwrap().instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );
    }
}

impl<T: Any> Renderable for Object<T> {
    fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, range: Range<u32>) {
        if self.render_data.is_none() {
            return;
        }

        let render_data = self.render_data.as_ref().unwrap();

        render_pass.set_vertex_buffer(0, render_data.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, render_data.instance_buffer.slice(..));
        render_pass.set_index_buffer(
            render_data.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        render_pass.draw_indexed(0..self.indices.len() as u32, 0, range);
    }

    fn num_instances(&self) -> u32 {
        self.instances.len() as u32
    }
}
