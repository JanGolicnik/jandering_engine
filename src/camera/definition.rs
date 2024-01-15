use cgmath::{Rotation, Rotation3, SquareMatrix};
use wgpu::{util::DeviceExt, BindGroupLayout};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyboardInput, MouseScrollDelta, WindowEvent},
    window::Window,
};

use crate::{camera::DefaultCameraPlugin, plugin::Plugin, renderer::Renderer};

use super::{
    constants::{CAMERA_SENSITIVITY, CAMERA_SPEED, OPENGL_TO_WGPU_MATRIX},
    CameraControllerData, CameraRenderData, CameraUniform,
};

impl Plugin for DefaultCameraPlugin {
    fn event(
        &mut self,
        event: &WindowEvent,
        _control_flow: &mut winit::event_loop::ControlFlow,
        renderer: &mut Renderer,
        window: &Window,
    ) {
        match event {
            WindowEvent::ModifiersChanged(state) => {
                self.controller.is_shift_pressed = state.shift()
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, val),
                ..
            } => {
                self.controller.speed_multiplier += val / 10.0;
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
                    winit::event::VirtualKeyCode::A => self.controller.left_pressed = is_pressed,
                    winit::event::VirtualKeyCode::D => self.controller.right_pressed = is_pressed,
                    winit::event::VirtualKeyCode::S => self.controller.forward_pressed = is_pressed,
                    winit::event::VirtualKeyCode::W => {
                        self.controller.backward_pressed = is_pressed
                    }
                    _ => {}
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let dx = renderer.config.width as f32 / 2.0 - position.x as f32;
                let dy = renderer.config.height as f32 / 2.0 - position.y as f32;

                let rotationx =
                    cgmath::Quaternion::from_angle_y(cgmath::Deg(dx * CAMERA_SENSITIVITY));
                let rotationy =
                    cgmath::Quaternion::from_angle_x(cgmath::Deg(dy * CAMERA_SENSITIVITY));
                self.direction = (rotationx + rotationy).rotate_vector(self.direction);
                window
                    .set_cursor_position(PhysicalPosition::new(
                        renderer.config.width / 2,
                        renderer.config.height / 2,
                    ))
                    .expect("failed to set cursor position");
            }
            _ => {}
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

        let speed = CAMERA_SPEED
            * self.controller.speed_multiplier
            * if self.controller.is_shift_pressed {
                2.0
            } else {
                1.0
            };
        if self.controller.left_pressed {
            self.velocity.x = -speed;
        }
        if self.controller.right_pressed {
            self.velocity.x = speed;
        }
        if self.controller.forward_pressed {
            self.velocity.z = -speed;
        }
        if self.controller.backward_pressed {
            self.velocity.z = speed;
        }

        self.eye += self.velocity.z * self.direction * dt as f32;
        self.eye += self.velocity.x * self.direction.cross(self.up) * dt as f32;

        self.velocity += -self.velocity * (dt * 6.0) as f32;
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
        self.aspect = renderer.config.width as f32 / renderer.config.height as f32;
        self.render_data = Some(CameraRenderData::new(&renderer.device));
        let render_data = &self.render_data.as_ref().unwrap();
        Some(vec![&render_data.bind_group_layout])
    }
}

impl Default for DefaultCameraPlugin {
    fn default() -> Self {
        Self {
            eye: cgmath::Point3 {
                x: 2.0,
                y: 2.0,
                z: 2.0,
            },
            direction: cgmath::Vector3 {
                x: 0.0,
                y: 0.0,
                z: -1.0,
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
            velocity: cgmath::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            controller: CameraControllerData {
                ..Default::default()
            },
            render_data: None,
        }
    }
}

impl DefaultCameraPlugin {
    pub fn new() -> Self {
        Self {
            eye: cgmath::Point3 {
                x: 2.0,
                y: 2.0,
                z: 2.0,
            },
            direction: cgmath::Vector3 {
                x: 0.0,
                y: 0.0,
                z: -1.0,
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
            velocity: cgmath::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            controller: CameraControllerData {
                ..Default::default()
            },
            render_data: None,
        }
    }

    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.aspect = physical_size.width as f32 / physical_size.height as f32;
    }

    pub fn update_uniform(&mut self) {
        let Self {
            eye,
            direction,
            up,
            aspect,
            znear,
            zfar,
            fov,
            ..
        } = self;

        let render_data = self.render_data.as_mut().unwrap();

        render_data.uniform.view_proj = {
            let view = cgmath::Matrix4::look_at_rh(*eye, *eye + *direction, *up);
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

impl Default for CameraControllerData {
    fn default() -> Self {
        Self {
            right_pressed: false,
            left_pressed: false,
            forward_pressed: false,
            backward_pressed: false,
            is_shift_pressed: false,
            speed_multiplier: 1.0,
        }
    }
}
