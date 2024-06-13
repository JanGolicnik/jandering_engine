use crate::{
    core::{
        bind_group::{BindGroup, BindGroupLayout, BindGroupLayoutEntry},
        engine::Events,
        renderer::{BufferHandle, Renderer},
        window::{InputState, Key, WindowEvent},
    },
    types::{Mat4, Vec2, Vec3},
};

use self::constants::*;

pub mod constants;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraData {
    up: Vec3,
    up_padding: f32,
    right: Vec3,
    right_padding: f32,
    position: Vec3,
    position_padding: f32,
    direction: Vec3,
    direction_padding: f32,
    view_proj: Mat4,
}

pub struct FreeCameraController {
    pub right_pressed: bool,
    pub left_pressed: bool,
    pub forward_pressed: bool,
    pub backward_pressed: bool,
    pub is_shift_pressed: bool,
    pub speed_multiplier: f32,
    pub velocity: Vec3,

    pub yaw: f32,
    pub pitch: f32,

    pub last_mouse_position: Option<Vec2>,
}

pub trait CameraController {
    fn event(&mut self, event: WindowEvent);
    fn update(&mut self, position: &mut Vec3, direction: &mut Vec3, dt: f32);
}

pub struct MatrixCameraBindGroup {
    data: CameraData,
    proj: Mat4,
    pub controller: Option<Box<dyn CameraController>>,
}

impl BindGroup for MatrixCameraBindGroup {
    fn get_data(&self) -> Box<[u8]> {
        bytemuck::cast_slice(&[self.data]).into()
    }

    fn get_layout(&self, renderer: &mut dyn Renderer) -> BindGroupLayout {
        let buffer_handle = renderer.create_uniform_buffer(&self.get_data());
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data(buffer_handle)],
        }
    }
}

impl Default for MatrixCameraBindGroup {
    fn default() -> Self {
        let data = CameraData {
            up: Vec3::ZERO,
            up_padding: 0.0,
            right: Vec3::ZERO,
            right_padding: 0.0,
            position: Vec3::ZERO,
            position_padding: 0.0,
            direction: -Vec3::Z,
            direction_padding: 0.0,
            view_proj: Mat4::IDENTITY,
        };

        Self {
            controller: None,
            proj: Mat4::IDENTITY,
            data,
        }
    }
}

impl MatrixCameraBindGroup {
    pub fn with_controller(controller: Box<dyn CameraController>) -> Self {
        let mut this = Self::default();
        this.attach_controller(controller);
        this
    }

    pub fn make_perspective(&mut self, fov: f32, aspect: f32, znear: f32, zfar: f32) {
        self.proj = Mat4::perspective_rh(fov.to_radians(), aspect, znear, zfar);
    }

    pub fn make_ortho(
        &mut self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) {
        self.proj = Mat4::orthographic_rh(left, right, bottom, top, near, far);
    }

    pub fn update_data(&mut self) {
        self.data.right = CAMERA_UP.cross(self.data.direction).normalize();
        self.data.up = self.data.direction.cross(self.data.right).normalize();

        self.data.view_proj = {
            let view = Mat4::look_at_rh(
                self.data.position,
                self.data.position + self.data.direction,
                CAMERA_UP,
            );
            OPENGL_TO_WGPU_MATRIX * self.proj * view
        };
    }

    pub fn update(&mut self, events: &Events, dt: f32) {
        if let Some(controller) = &mut self.controller {
            let controller = controller.as_mut();

            for event in events.iter() {
                controller.event(*event);
            }

            controller.update(&mut self.data.position, &mut self.data.direction, dt);
        }

        self.update_data()
    }

    pub fn get_layout() -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data(BufferHandle(0))],
        }
    }

    pub fn attach_controller(&mut self, controller: Box<dyn CameraController>) -> &mut Self {
        self.controller = Some(controller);
        self
    }

    pub fn position_mut(&mut self) -> &mut Vec3 {
        &mut self.data.position
    }
    pub fn position(&self) -> Vec3 {
        self.data.position
    }

    pub fn direction_mut(&mut self) -> &mut Vec3 {
        &mut self.data.direction
    }
    pub fn direction(&self) -> Vec3 {
        self.data.direction
    }
    pub fn controller(&self) -> &Option<Box<dyn CameraController>> {
        &self.controller
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
            last_mouse_position: None,
        }
    }
}

impl CameraController for FreeCameraController {
    fn event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::MouseMotion(position) => {
                let position = Vec2::from(position);
                if let Some(last_mouse_position) = &mut self.last_mouse_position {
                    let (dx, dy) = (*last_mouse_position - position).into();
                    self.yaw -= dx * CAMERA_SENSITIVITY;
                    self.pitch += dy * CAMERA_SENSITIVITY;
                    self.pitch = self.pitch.clamp(-89.0, 89.0);
                    *last_mouse_position = position;
                } else {
                    self.last_mouse_position = Some(position);
                }
            }
            WindowEvent::MouseLeft => {
                self.last_mouse_position = None;
            }
            WindowEvent::Scroll((_, val)) => {
                if val.is_sign_positive() {
                    self.speed_multiplier += (CAMERA_SPEED_MAX - self.speed_multiplier) / 100.0;
                } else {
                    self.speed_multiplier += (CAMERA_SPEED_MIN - self.speed_multiplier) / 20.0;
                }
            }

            WindowEvent::KeyInput { key, state } => {
                let is_pressed = matches!(state, InputState::Pressed);
                match key {
                    Key::A => self.left_pressed = is_pressed,
                    Key::D => self.right_pressed = is_pressed,
                    Key::S => self.forward_pressed = is_pressed,
                    Key::W => self.backward_pressed = is_pressed,
                    Key::Shift => self.is_shift_pressed = is_pressed,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn update(&mut self, object_position: &mut Vec3, object_direction: &mut Vec3, dt: f32) {
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
        let yaw_rad = yaw.to_radians();
        let pitch_rad = pitch.to_radians();
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

        *object_position += velocity.z * *object_direction * dt;
        *object_position += velocity.x * object_direction.cross(Vec3::Y) * dt;

        self.velocity += -self.velocity * (dt * 6.0);
    }
}
