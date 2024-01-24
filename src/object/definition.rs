use crate::{object::VertexRaw, renderer::Renderer};
use cgmath::{EuclideanSpace, SquareMatrix};
use wgpu::util::DeviceExt;

use super::{Instance, InstanceRaw, Object, ObjectRenderData, Renderable};

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
            scale: None,
            position: None,
            rotation: None,
            changed: true,
        }
    }
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        let mut model = cgmath::Matrix4::<f32>::identity();

        if let Some(scale) = self.scale {
            model = cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z) * model;
        }

        if let Some(rotation) = self.rotation {
            model = cgmath::Matrix4::from(rotation) * model;
        }

        if let Some(position) = self.position {
            model = cgmath::Matrix4::from_translation(position.to_vec()) * model;
        }

        InstanceRaw {
            model: model.into(),
        }
    }
}

impl Default for InstanceRaw {
    fn default() -> Self {
        Self {
            model: cgmath::Matrix4::identity().into(),
        }
    }
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
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

impl Object {
    pub fn new(renderer: &Renderer) -> Self {
        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::INDEX,
            });
        let instance_buffer =
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: &[],
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            instances: Vec::new(),
            instance_data: Vec::new(),
            render_data: Some(ObjectRenderData {
                vertex_buffer,
                index_buffer,
                instance_buffer,
            }),
            shader: 0,
        }
    }
    pub fn update(&mut self) {
        self.instance_data = self.instances.iter().map(|e| e.to_raw()).collect();
    }
}

impl Renderable for Object {
    fn bind<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        _queue: &mut wgpu::Queue,
        shaders: &'a [wgpu::RenderPipeline],
    ) {
        if self.render_data.is_none() {
            return;
        }

        if let Some(shader) = shaders.get(self.shader) {
            render_pass.set_pipeline(shader);
        }

        let render_data = self.render_data.as_ref().unwrap();

        render_pass.set_vertex_buffer(0, render_data.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, render_data.instance_buffer.slice(..));
        render_pass.set_index_buffer(
            render_data.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        render_pass.draw_indexed(
            0..self.indices.len() as u32,
            0,
            0..self.instances.len() as u32,
        );
    }
}
