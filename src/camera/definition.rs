use cgmath::{Angle, InnerSpace, SquareMatrix};
use wgpu::{util::DeviceExt, BindGroupLayout};
#[allow(unused_imports)]
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyboardInput, MouseScrollDelta, WindowEvent},
    window::Window,
};

use crate::{camera::DefaultCameraPlugin, plugins::Plugin};

use super::{
    constants::{CAMERA_SENSITIVITY, CAMERA_SPEED, CAMERA_UP, OPENGL_TO_WGPU_MATRIX},
    CameraRenderData, CameraUniform, FreeCameraController, PerspectiveCameraData,
};

#[allow(unused_variables)]
impl Plugin for DefaultCameraPlugin {
    fn get_bind_group_layout(&self) -> Option<&BindGroupLayout> {
        Some(&self.render_data.bind_group_layout)
    }

    fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        Some(&self.render_data.bind_group)
    }

    fn update(
        &mut self,
        context: &mut crate::engine::EngineContext,
        renderer: &mut crate::renderer::Renderer,
    ) {
        self.resize(PhysicalSize::new(
            renderer.config.width,
            renderer.config.height,
        ));

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

impl DefaultCameraPlugin {
    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.perspective.aspect = physical_size.width as f32 / physical_size.height as f32;
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

        self.render_data.uniform.view_proj = {
            let view = cgmath::Matrix4::look_at_rh(*position, position + direction, CAMERA_UP);
            let proj = cgmath::perspective(cgmath::Deg(*fov), *aspect, *znear, *zfar);
            OPENGL_TO_WGPU_MATRIX * proj * view
        }
        .into();
        self.render_data.uniform.view_position = position.to_homogeneous().into();
    }

    pub fn new(renderer: &crate::renderer::Renderer) -> Self {
        let uniform = CameraUniform {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        };

        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });

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
            render_data: CameraRenderData {
                bind_group_layout,
                bind_group,
                uniform,
                buffer,
            },
            #[cfg(target_arch = "wasm32")]
            last_mouse_position: None,
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
