use cgmath::{Angle, InnerSpace, SquareMatrix};
use wgpu::{util::DeviceExt, BindGroupLayout};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyboardInput, MouseScrollDelta, WindowEvent},
    window::Window,
};

use crate::{camera::DefaultCameraPlugin, plugin::Plugin, renderer::Renderer};

use super::{
    constants::{CAMERA_SENSITIVITY, CAMERA_SPEED, CAMERA_UP, OPENGL_TO_WGPU_MATRIX},
    CameraRenderData, CameraUniform, FreeCameraController, PerspectiveCameraData,
};

impl Plugin for DefaultCameraPlugin {
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

        let render_data = &self.render_data.as_ref().unwrap();

        queue.write_buffer(
            &render_data.buffer,
            0,
            bytemuck::cast_slice(&[render_data.uniform]),
        );
        Some((0, &render_data.bind_group))
    }

    fn initialize(&mut self, renderer: &mut Renderer) -> Option<Vec<&BindGroupLayout>> {
        self.perspective.aspect = renderer.config.width as f32 / renderer.config.height as f32;
        self.render_data = Some(CameraRenderData::new(&renderer.device));
        let render_data = &self.render_data.as_ref().unwrap();
        Some(vec![&render_data.bind_group_layout])
    }
}

impl Default for DefaultCameraPlugin {
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

impl DefaultCameraPlugin {
    pub fn new() -> Self {
        Self {
            perspective: super::PerspectiveCameraData {
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

    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.perspective.aspect = physical_size.width as f32 / physical_size.height as f32;
    }

    pub fn update_uniform(&mut self) {
        let PerspectiveCameraData {
            position: eye,
            aspect,
            znear,
            zfar,
            fov,
            ..
        } = &self.perspective;

        let render_data = self.render_data.as_mut().unwrap();

        render_data.uniform.view_proj = {
            let view =
                cgmath::Matrix4::look_at_rh(*eye, cgmath::Point3::new(0.0, 0.0, 0.0), CAMERA_UP);
            let proj = cgmath::perspective(cgmath::Deg(*fov), *aspect, *znear, *zfar);
            OPENGL_TO_WGPU_MATRIX * proj * view
        }
        .into();
        render_data.uniform.view_position = eye.to_homogeneous().into();
    }
}

impl CameraRenderData {
    pub fn new(device: &wgpu::Device) -> Self {
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
            bind_group_layout,
            bind_group,
            uniform,
            buffer,
        }
    }
}

impl Default for FreeCameraController {
    fn default() -> Self {
        Self {
            right_pressed: false,
            left_pressed: false,
            forward_pressed: false,
            backward_pressed: false,
            is_shift_pressed: false,
            speed_multiplier: 1.0,
            velocity: cgmath::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl FreeCameraController {
    pub fn cursor_moved(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * CAMERA_SENSITIVITY;
        self.pitch -= dy * CAMERA_SENSITIVITY;
        self.pitch = self.pitch.clamp(-89.0, 89.0);
    }

    pub fn event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::ModifiersChanged(state) => self.is_shift_pressed = state.shift(),
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, val),
                ..
            } => {
                self.speed_multiplier += val / 10.0;
            }

            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = ElementState::Pressed == *state;
                match keycode {
                    winit::event::VirtualKeyCode::A => self.left_pressed = is_pressed,
                    winit::event::VirtualKeyCode::D => self.right_pressed = is_pressed,
                    winit::event::VirtualKeyCode::S => self.forward_pressed = is_pressed,
                    winit::event::VirtualKeyCode::W => self.backward_pressed = is_pressed,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn update(
        &mut self,
        object_position: &mut cgmath::Point3<f32>,
        object_direction: &mut cgmath::Vector3<f32>,
        dt: f64,
    ) {
        let Self {
            right_pressed,
            left_pressed,
            forward_pressed,
            backward_pressed,
            is_shift_pressed,
            speed_multiplier,
            velocity,
            yaw,
            pitch,
            ..
        } = *self;

        let mut direction = cgmath::Vector3::new(0.0, 0.0, 0.0);
        let yaw_rad = cgmath::Rad::from(cgmath::Deg(yaw));
        let pitch_rad = cgmath::Rad::from(cgmath::Deg(pitch));
        direction.x = cgmath::Rad::cos(yaw_rad) * cgmath::Rad::cos(pitch_rad);
        direction.y = cgmath::Rad::sin(pitch_rad);
        direction.z = cgmath::Rad::sin(yaw_rad) * cgmath::Rad::cos(pitch_rad);
        *object_direction = direction.normalize();

        let speed = CAMERA_SPEED * speed_multiplier * if is_shift_pressed { 2.0 } else { 1.0 };
        if left_pressed {
            self.velocity.x = -speed;
        }
        if right_pressed {
            self.velocity.x = speed;
        }
        if forward_pressed {
            self.velocity.z = -speed;
        }
        if backward_pressed {
            self.velocity.z = speed;
        }

        *object_position += velocity.z * *object_direction * dt as f32;
        *object_position +=
            velocity.x * object_direction.cross(cgmath::Vector3::unit_y()) * dt as f32;

        self.velocity += -self.velocity * (dt * 6.0) as f32;
    }
}
