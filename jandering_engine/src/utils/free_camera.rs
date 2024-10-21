use crate::{
    bind_group::{
        BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutDescriptorEntry,
        BindGroupLayoutEntry,
    },
    renderer::{BindGroupHandle, BufferHandle, Janderer, Renderer, UntypedBindGroupHandle},
    types::{Mat4, Vec2, Vec3},
};

use je_windowing::{Events, InputState, Key, WindowEvent};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(
    &[1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,]
);

pub const CAMERA_SPEED: f32 = 16.5;
pub const CAMERA_SENSITIVITY: f32 = 0.2;

pub const CAMERA_SPEED_MAX: f32 = 10.0;
pub const CAMERA_SPEED_MIN: f32 = 0.01;

pub const CAMERA_UP: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

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
    inverse_view: Mat4,
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
    fn set_direction(&mut self, _direction: Vec3) {}
}

struct MatrixCameraBindGroup {
    buffer_handle: BufferHandle,
}

pub struct MatrixCamera {
    bind_group: BindGroupHandle<MatrixCameraBindGroup>,
    data: CameraData,
    proj: Mat4,
    controller: Option<Box<dyn CameraController>>,
}

impl std::fmt::Debug for MatrixCamera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point").field("data", &self.data).finish()
    }
}

impl BindGroup for MatrixCameraBindGroup {
    fn get_layout_descriptor() -> BindGroupLayoutDescriptor {
        BindGroupLayoutDescriptor {
            entries: vec![BindGroupLayoutDescriptorEntry::Data { is_uniform: true }],
        }
    }

    fn get_layout(&self) -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data(self.buffer_handle)],
        }
    }
}

impl MatrixCamera {
    pub fn new(renderer: &mut Renderer) -> Self {
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
            inverse_view: Mat4::IDENTITY,
        };

        let buffer_handle = renderer.create_uniform_buffer(bytemuck::cast_slice(&[data]));

        let bind_group = renderer.create_typed_bind_group(MatrixCameraBindGroup { buffer_handle });

        Self {
            bind_group,
            proj: Mat4::IDENTITY,
            controller: None,
            data,
        }
    }

    pub fn get_layout_descriptor(&self) -> BindGroupLayoutDescriptor {
        MatrixCameraBindGroup::get_layout_descriptor()
    }

    pub fn with_controller(
        renderer: &mut Renderer,
        controller: impl CameraController + 'static,
    ) -> Self {
        let mut this = Self::new(renderer);
        this.attach_controller(Box::new(controller));
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

    pub fn matrix(&self) -> Mat4 {
        self.data.view_proj
    }

    pub fn update_data(&mut self) {
        self.data.right = CAMERA_UP.cross(self.data.direction).normalize();
        self.data.up = self.data.direction.cross(self.data.right).normalize();

        let view = Mat4::look_at_rh(
            self.data.position,
            self.data.position + self.data.direction,
            CAMERA_UP,
        );

        self.data.view_proj = OPENGL_TO_WGPU_MATRIX * self.proj * view;
        self.data.inverse_view = view.inverse();
    }

    pub fn update(&mut self, renderer: &mut Renderer, events: &Events, dt: f32) {
        if let Some(controller) = &mut self.controller {
            let controller = controller.as_mut();

            for event in events.iter() {
                controller.event(*event);
            }

            controller.update(&mut self.data.position, &mut self.data.direction, dt);
        }

        self.update_data();

        let bind_group = renderer.get_typed_bind_group(self.bind_group).unwrap();
        renderer.write_buffer(bind_group.buffer_handle, bytemuck::cast_slice(&[self.data]));
    }

    pub fn attach_controller(&mut self, mut controller: Box<dyn CameraController>) -> &mut Self {
        controller.set_direction(self.direction());
        self.controller = Some(controller);
        self
    }

    pub fn take_controller(&mut self) -> Option<Box<dyn CameraController>> {
        self.controller.take()
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.data.position = pos;
    }

    pub fn position(&self) -> Vec3 {
        self.data.position
    }

    pub fn set_direction(&mut self, direction: Vec3) {
        self.data.direction = direction;
        if let Some(controller) = &mut self.controller {
            controller.set_direction(direction);
        }
    }
    pub fn direction(&self) -> Vec3 {
        self.data.direction
    }
    pub fn controller(&self) -> &Option<Box<dyn CameraController>> {
        &self.controller
    }
    pub fn up(&self) -> Vec3 {
        self.data.up
    }
    pub fn right(&self) -> Vec3 {
        self.data.right
    }
    pub fn bind_group(&self) -> UntypedBindGroupHandle {
        self.bind_group.into()
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

    fn set_direction(&mut self, direction: Vec3) {
        let length = direction.length();
        self.pitch = (direction.y / length).asin().to_degrees();
        let cos_pitch = self.pitch.cos();
        if cos_pitch != 0.0 {
            self.yaw = (direction.x / (cos_pitch * length)).asin().to_degrees();
        } else {
            self.yaw = 0.0;
        }
    }
}
