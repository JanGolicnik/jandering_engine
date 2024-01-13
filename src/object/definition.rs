use cgmath::SquareMatrix;

use crate::object::VertexRaw;

use super::{Instance, InstanceRaw, Object};

impl VertexRaw {
    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
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
                    format: wgpu::VertexFormat::Float32x3,
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

impl Default for InstanceRaw {
    fn default() -> Self {
        Self {
            model: cgmath::Matrix4::identity().into(),
        }
    }
}

impl InstanceRaw {
    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
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

    pub(crate) fn update(&mut self, instance: &Instance) {
        let mut model = cgmath::Matrix4::<f32>::identity();
        if let Some(rotation) = instance.rotation {
            model = cgmath::Matrix4::from(rotation) * model;
        }

        if let Some(position) = instance.position {
            model = cgmath::Matrix4::from_translation(position) * model;
        }

        if let Some(scale) = instance.scale {
            model = cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z) * model;
        }

        self.model = model.into();
    }
}

impl Object {
    pub fn update(&mut self) {
        self.instance_data
            .resize(self.instances.len(), InstanceRaw::default());

        for (i, instance) in self
            .instances
            .iter_mut()
            .enumerate()
            .filter(|(_, e)| e.changed)
        {
            instance.changed = false;
            self.instance_data[i].update(instance);
        }
    }
}
