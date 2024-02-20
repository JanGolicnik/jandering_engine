use cgmath::Zero;
use wgpu::{util::DeviceExt, BindGroupLayout};
use winit::event::MouseButton;
#[allow(unused_imports)]
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyboardInput, MouseScrollDelta, WindowEvent},
    window::Window,
};

use super::{BindGroupRenderData, D2CameraData};
use super::{D2CameraBindGroup, D2CameraController};
use crate::{
    bind_group::{BindGroup, BindGroupWriteData},
    engine::EngineContext,
    types::Vec2,
};

#[allow(unused_variables)]
impl BindGroup for D2CameraBindGroup {
    fn get_bind_group_layout(&self) -> Option<&BindGroupLayout> {
        Some(&self.render_data.bind_group_layout)
    }

    fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        Some(&self.render_data.bind_group)
    }

    fn write(&mut self, data: &BindGroupWriteData) {
        self.resize(PhysicalSize::new(data.config.width, data.config.height));

        self.handle_events(data.context);
        if let Some(controller) = self.controller.as_mut() {
            controller.update(&mut self.data.position, data.context.dt);
        }
        self.update_uniform();

        data.queue.write_buffer(
            &self.render_data.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}

impl D2CameraBindGroup {
    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.data.resolution = Vec2::new(physical_size.width as f32, physical_size.height as f32);
    }

    pub fn update_uniform(&mut self) {
        let D2CameraData {
            position,
            resolution,
            ..
        } = &self.data;

        self.uniform.view_position = [position.x, position.y];
        let zoom = if let Some(controller) = self.controller.as_ref() {
            controller.zoom
        } else {
            1.0
        };
        self.uniform.resolution = [resolution[0] * zoom, resolution[1] * zoom];
    }

    pub fn new(renderer: &crate::renderer::Renderer, with_controller: bool) -> Self {
        let uniform = super::FlatCameraUniform {
            view_position: [0.0; 2],
            resolution: [renderer.config.width as f32, renderer.config.height as f32],
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

        let controller = if with_controller {
            Some(D2CameraController::default())
        } else {
            None
        };

        Self {
            data: D2CameraData {
                position: Vec2::zero(),
                resolution: Vec2::new(renderer.config.width as f32, renderer.config.height as f32),
            },
            controller,
            uniform,
            render_data: BindGroupRenderData {
                bind_group_layout,
                bind_group,
                buffer,
            },
            last_mouse_position: None,
            pressing: false,
            mouse_is_inside: true,
        }
    }

    #[allow(unused_variables)]
    fn handle_events(&mut self, context: &EngineContext) {
        for event in context.events.iter() {
            if let Some(controller) = self.controller.as_mut() {
                controller.event(event);
            };

            match event {
                WindowEvent::CursorMoved {
                    device_id,
                    position,
                    ..
                } => {
                    if let Some(controller) = self.controller.as_mut() {
                        let (dx, dy) = if self.mouse_is_inside && self.pressing {
                            let last_mouse_position = self
                                .last_mouse_position
                                .unwrap_or((position.x as f32, position.y as f32));
                            let dx = position.x as f32 - last_mouse_position.0;
                            let dy = position.y as f32 - last_mouse_position.1;
                            self.last_mouse_position = Some((position.x as f32, position.y as f32));
                            (dx, -dy)
                        } else {
                            self.last_mouse_position = None;
                            (0.0, 0.0)
                        };

                        controller.cursor_moved(dx, dy);
                    };
                }
                WindowEvent::CursorEntered { device_id } => self.mouse_is_inside = true,
                WindowEvent::CursorLeft { device_id } => self.mouse_is_inside = false,
                WindowEvent::MouseInput {
                    device_id,
                    state,
                    button: MouseButton::Left,
                    ..
                } => {
                    self.pressing = matches!(state, ElementState::Pressed);
                }
                _ => {}
            }
        }
    }
}

impl Default for D2CameraController {
    fn default() -> Self {
        Self {
            right_pressed: false,
            left_pressed: false,
            up_pressed: false,
            down_pressed: false,
            is_shift_pressed: false,
            zoom: 1.0,
            is_mouse_pressed: false,
            velocity: cgmath::Vector2 { x: 0.0, y: 0.0 },
            pan_offset: cgmath::Vector2 { x: 0.0, y: 0.0 },
        }
    }
}

impl D2CameraController {
    pub fn cursor_moved(&mut self, dx: f32, dy: f32) {
        self.pan_offset.x += dx * self.zoom * 2.0;
        self.pan_offset.y += dy * self.zoom * 2.0;
    }

    pub fn event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::ModifiersChanged(state) => self.is_shift_pressed = state.shift(),
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, val),
                ..
            } => {
                self.zoom = (self.zoom - val / 10.0).max(0.2);
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
                    winit::event::VirtualKeyCode::S => self.up_pressed = is_pressed,
                    winit::event::VirtualKeyCode::W => self.down_pressed = is_pressed,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self, object_position: &mut Vec2, dt: f64) {
        let Self {
            right_pressed,
            left_pressed,
            up_pressed,
            down_pressed,
            is_shift_pressed,
            ..
        } = *self;

        let speed = if is_shift_pressed { 2.0 } else { 1.0 };
        if left_pressed {
            self.velocity.x = speed;
        }
        if right_pressed {
            self.velocity.x = -speed;
        }
        if up_pressed {
            self.velocity.y = speed;
        }
        if down_pressed {
            self.velocity.y = -speed;
        }

        *object_position += self.velocity * dt as f32 * 2000.0;
        self.velocity += -self.velocity * (dt * 6.0) as f32;

        object_position.x += self.pan_offset.x;
        object_position.y += self.pan_offset.y;

        self.pan_offset.x = 0.0;
        self.pan_offset.y = 0.0;
    }
}
