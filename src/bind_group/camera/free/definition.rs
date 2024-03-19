use wgpu::{util::DeviceExt, BindGroupLayout};
#[allow(unused_imports)]
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyboardInput, MouseScrollDelta, WindowEvent},
    window::Window,
};

use super::FreeCameraBindGroup;
use super::{
    constants::{CAMERA_SENSITIVITY, CAMERA_SPEED, CAMERA_UP, OPENGL_TO_WGPU_MATRIX},
    BindGroupRenderData, FreeCameraController, PerspectiveCameraData,
};
use crate::{
    bind_group::{camera::CameraUniform, BindGroup, BindGroupWriteData},
    engine::EngineContext,
    types::{Mat4, UVec2, Vec3, RAD_TO_DEG},
};

#[allow(unused_variables)]
impl BindGroup for FreeCameraBindGroup {
    fn get_bind_group_layout(&self) -> Option<&BindGroupLayout> {
        Some(&self.render_data.bind_group_layout)
    }

    fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        Some(&self.render_data.bind_group)
    }

    fn write(&mut self, data: &BindGroupWriteData) {
        self.update_uniform();

        data.queue.write_buffer(
            &self.render_data.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}

impl FreeCameraBindGroup {
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

        self.uniform.view_proj = {
            let view = Mat4::look_at_rh(*position, *position + *direction, CAMERA_UP);
            let proj = Mat4::perspective_rh(*fov, *aspect, *znear, *zfar);
            OPENGL_TO_WGPU_MATRIX * proj * view
        }
        .to_cols_array_2d();
        self.uniform.view_position = [position.x, position.y, position.z, 1.0];
    }

    pub fn new(renderer: &crate::renderer::Renderer) -> Self {
        let uniform = CameraUniform {
            view_position: [0.0; 4],
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
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
                position: Vec3 {
                    x: 2.0,
                    y: 2.0,
                    z: 2.0,
                },
                direction: Vec3 {
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
            render_data: BindGroupRenderData {
                bind_group_layout,
                bind_group,
                buffer,
            },
            uniform,
            #[cfg(target_arch = "wasm32")]
            last_mouse_position: None,
        }
    }

    pub fn update(&mut self, context: &EngineContext, resolution: UVec2) {
        self.resize(PhysicalSize::new(resolution.x, resolution.y));
        for event in context.events.iter() {
            self.controller.event(event);

            if let winit::event::WindowEvent::CursorMoved { position, .. } = event {
                cfg_if::cfg_if! {
                    if #[cfg(target_arch = "wasm32")]{
                        let last_mouse_position = self.last_mouse_position.unwrap_or((position.x as f32, position.y as f32));
                        let dx = position.x as f32 - last_mouse_position.0;
                        let dy = position.y as f32 - last_mouse_position.1;
                        self.last_mouse_position = Some((position.x as f32, position.y as f32));
                    }else{
                        let dx = position.x as f32 - resolution.x as f32 / 2.0;
                        let dy = position.y as f32 - resolution.y as f32 / 2.0;
                        context
                            .window
                            .set_cursor_position(winit::dpi::PhysicalPosition::new(
                                resolution.x / 2,
                                resolution.y / 2,
                            ))
                            .expect("failed to set cursor position");
                    }
                }

                self.controller.cursor_moved(dx, dy);
            }
        }

        self.controller.update(
            &mut self.perspective.position,
            &mut self.perspective.direction,
            context.dt,
        );
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
            velocity: Vec3 {
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

    pub fn update(&mut self, object_position: &mut Vec3, object_direction: &mut Vec3, dt: f64) {
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

        let mut direction = Vec3::new(0.0, 0.0, 0.0);
        let yaw_rad = yaw * RAD_TO_DEG;
        let pitch_rad = pitch * RAD_TO_DEG;
        direction.x = yaw_rad.cos() * pitch_rad.cos();
        direction.y = pitch_rad.sin();
        direction.z = yaw_rad.sin() * pitch_rad.cos();
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
        *object_position += velocity.x * object_direction.cross(Vec3::Y) * dt as f32;

        self.velocity += -self.velocity * (dt * 6.0) as f32;
    }
}
