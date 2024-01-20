use cgmath::{InnerSpace, SquareMatrix};
use jandering_engine::{
    camera::{
        constants::{CAMERA_UP, OPENGL_TO_WGPU_MATRIX},
        FreeCameraController, PerspectiveCameraData,
    },
    plugin::Plugin,
    renderer::Renderer,
};
use wgpu::{util::DeviceExt, BindGroupLayout};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    window::Window,
};

use super::{CustomCameraPlugin, CustomCameraRenderData, CustomCameraUniform};

impl Plugin for CustomCameraPlugin {
    fn event(
        &mut self,
        event: &WindowEvent,
        _control_flow: &mut winit::event_loop::ControlFlow,
        renderer: &mut Renderer,
        window: &Window,
    ) {
        self.controller.event(event);

        if let WindowEvent::CursorMoved { position, .. } = event {
            let dx = position.x as f32 - renderer.config.width as f32 / 2.0;
            let dy = position.y as f32 - renderer.config.height as f32 / 2.0;
            self.controller.cursor_moved(dx, dy);
            window
                .set_cursor_position(PhysicalPosition::new(
                    renderer.config.width / 2,
                    renderer.config.height / 2,
                ))
                .expect("failed to set cursor position");
        }
    }

    fn update(
        &mut self,
        _control_flow: &mut winit::event_loop::ControlFlow,
        renderer: &mut Renderer,
        dt: f64,
    ) {
        self.resize(PhysicalSize::new(
            renderer.config.width,
            renderer.config.height,
        ));

        self.controller.update(
            &mut self.perspective.position,
            &mut self.perspective.direction,
            dt,
        );
    }

    fn pre_render(&mut self, queue: &mut wgpu::Queue) -> Option<(u32, &wgpu::BindGroup)> {
        self.update_uniform();

        let render_data = self.render_data.as_ref().unwrap();

        queue.write_buffer(
            &render_data.buffer,
            0,
            bytemuck::cast_slice(&[render_data.uniform]),
        );
        Some((0, &render_data.bind_group))
    }

    fn initialize(&mut self, renderer: &mut Renderer) -> Option<Vec<&BindGroupLayout>> {
        self.perspective.aspect = renderer.config.width as f32 / renderer.config.height as f32;
        self.render_data = Some(CustomCameraRenderData::new(&renderer.device));
        let render_data = &self.render_data.as_ref().unwrap();
        Some(vec![&render_data.bind_group_layout])
    }
}

impl Default for CustomCameraPlugin {
    fn default() -> Self {
        Self {
            perspective: PerspectiveCameraData {
                position: cgmath::Point3 {
                    x: 2.0,
                    y: 2.0,
                    z: 2.0,
                },
                direction: cgmath::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: -1.0,
                },
                fov: 45.0,
                znear: 0.1,
                zfar: 100.0,
                aspect: 1.0,
            },
            controller: FreeCameraController {
                ..Default::default()
            },
            render_data: None,
        }
    }
}

impl CustomCameraPlugin {
    pub fn new() -> Self {
        let position = cgmath::Point3 {
            x: 2.0,
            y: 2.0,
            z: 2.0,
        };
        let target = cgmath::Point3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let direction = target - position;

        Self {
            perspective: super::PerspectiveCameraData {
                position,
                direction,
                fov: 90.0,
                znear: 0.1,
                zfar: 100.0,
                aspect: 1.0,
            },
            controller: FreeCameraController {
                ..Default::default()
            },
            render_data: None,
        }
    }

    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.perspective.aspect = physical_size.width as f32 / physical_size.height as f32;
    }

    pub fn update_uniform(&mut self) {
        let PerspectiveCameraData {
            position: eye,
            direction,
            aspect,
            znear,
            zfar,
            fov,
            ..
        } = &self.perspective;

        let render_data = self.render_data.as_mut().unwrap();

        let right = CAMERA_UP.cross(*direction).normalize();
        render_data.uniform.right = [right.x, right.y, right.z, 0.0];

        let up = direction.cross(right).normalize();
        render_data.uniform.up = [up.x, up.y, up.z, 0.0];

        render_data.uniform.view_proj = {
            let view = cgmath::Matrix4::look_at_rh(*eye, eye + direction, CAMERA_UP);
            let proj = cgmath::perspective(cgmath::Deg(*fov), *aspect, *znear, *zfar);
            OPENGL_TO_WGPU_MATRIX * proj * view
        }
        .into();
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
