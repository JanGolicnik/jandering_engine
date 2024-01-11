use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use crate::camera::Camera;

use super::{constants::OPENGL_TO_WGPU_MATRIX, CameraRenderData, CameraUniform};

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: cgmath::Point3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            target: cgmath::Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            up: cgmath::Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            fov: 45.0,
            znear: 0.1,
            zfar: 100.0,
            aspect: 1.0,
            render_data: None,
        }
    }
}

impl Camera {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let uniform = CameraUniform {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("model_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            eye: cgmath::Point3 {
                x: 2.0,
                y: 2.0,
                z: 2.0,
            },
            target: cgmath::Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            up: cgmath::Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            fov: 45.0,
            znear: 0.1,
            zfar: 100.0,
            aspect: config.width as f32 / config.height as f32,
            render_data: Some(super::CameraRenderData {
                bind_group,
                uniform,
                buffer,
                bind_group_layout,
            }),
        }
    }

    pub fn update_uniform(&mut self) {
        if let Some(CameraRenderData { uniform, .. }) = &mut self.render_data {
            uniform.view_proj = {
                let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
                let proj =
                    cgmath::perspective(cgmath::Deg(self.fov), self.aspect, self.znear, self.zfar);
                OPENGL_TO_WGPU_MATRIX * proj * view
            }
            .into();
            uniform.view_position = self.eye.to_homogeneous().into();
        }
    }

    pub fn get_render_data(&self) -> Option<&CameraRenderData> {
        self.render_data.as_ref()
    }
}
