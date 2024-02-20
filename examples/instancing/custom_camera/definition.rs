use cgmath::{InnerSpace, SquareMatrix};
use jandering_engine::{
    bind_group::camera::free::{
        constants::{CAMERA_UP, OPENGL_TO_WGPU_MATRIX},
        FreeCameraController, PerspectiveCameraData,
    },
    bind_group::BindGroup,
    engine::EngineContext,
    renderer::Renderer,
};
use wgpu::{util::DeviceExt, BindGroupLayout};

use super::{CustomCameraPlugin, CustomCameraRenderData, CustomCameraUniform};

impl BindGroup for CustomCameraPlugin {
    fn get_bind_group_layout(&self) -> Option<&BindGroupLayout> {
        Some(&self.render_data.bind_group_layout)
    }

    fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        Some(&self.render_data.bind_group)
    }

    fn update(&mut self, context: &mut EngineContext, renderer: &mut Renderer) {
        self.handle_events(context, renderer);

        self.controller.update(
            &mut self.perspective.position,
            &mut self.perspective.direction,
            context.dt,
        );

        self.update_uniform();

        renderer.queue.write_buffer(
            &self.render_data.buffer,
            0,
            bytemuck::cast_slice(&[self.render_data.uniform]),
        );
    }
}

impl CustomCameraPlugin {
    pub fn new(renderer: &mut Renderer) -> Self {
        let position = cgmath::Point3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };
        let target = cgmath::Point3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let direction = (target - position).normalize();

        Self {
            perspective: super::PerspectiveCameraData {
                position,
                direction,
                fov: 45.0,
                znear: 0.1,
                zfar: 100.0,
                aspect: renderer.config.width as f32 / renderer.config.height as f32,
            },
            controller: FreeCameraController {
                ..Default::default()
            },
            render_data: CustomCameraRenderData::new(&renderer.device),
        }
    }

    pub fn update_uniform(&mut self) {
        let PerspectiveCameraData {
            position,
            direction,
            aspect,
            znear,
            zfar,
            fov,
            ..
        } = &self.perspective;

        let right = CAMERA_UP.cross(*direction).normalize();
        self.render_data.uniform.right = [right.x, right.y, right.z, 0.0];

        let up = direction.cross(right).normalize();
        self.render_data.uniform.up = [up.x, up.y, up.z, 0.0];

        self.render_data.uniform.view_proj = {
            let view = cgmath::Matrix4::look_at_rh(*position, position + direction, CAMERA_UP);
            let proj = cgmath::perspective(cgmath::Deg(*fov), *aspect, *znear, *zfar);
            OPENGL_TO_WGPU_MATRIX * proj * view
        }
        .into();
    }

    fn handle_events(&mut self, context: &EngineContext, renderer: &mut Renderer) {
        for event in context.events.iter() {
            self.controller.event(event);

            if let winit::event::WindowEvent::CursorMoved { position, .. } = event {
                let dx = position.x as f32 - renderer.config.width as f32 / 2.0;
                let dy = position.y as f32 - renderer.config.height as f32 / 2.0;
                self.controller.cursor_moved(dx, dy);
                context
                    .window
                    .set_cursor_position(winit::dpi::PhysicalPosition::new(
                        renderer.config.width / 2,
                        renderer.config.height / 2,
                    ))
                    .expect("failed to set cursor position");
            }
        }
    }
}

impl CustomCameraRenderData {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = CustomCameraUniform {
            up: [0.0, 1.0, 0.0, 0.0],
            right: [1.0, 0.0, 0.0, 0.0],
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
            label: Some("camera_bind_group_layout"),
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
            bind_group_layout,
            bind_group,
            uniform,
            buffer,
        }
    }
}
