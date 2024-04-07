use crate::{
    core::{
        bind_group::{BindGroup, BindGroupLayout, BindGroupLayoutEntry},
        renderer::{BufferHandle, Renderer},
        window::{InputState, Key, Window, WindowEvent},
    },
    types::{Mat4, UVec2, Vec3, DEG_TO_RAD},
};

use self::constants::*;

pub mod constants;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraData {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
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
}

pub struct PerspectiveCameraData {
    pub position: Vec3,
    pub direction: Vec3,
    //
    pub fov: f32,
    pub znear: f32,
    pub zfar: f32,
    pub aspect: f32,
}

pub struct FreeCameraBindGroup {
    perspective: PerspectiveCameraData,
    //
    controller: FreeCameraController,
    //
    data: CameraData,
    //
    // #[allow(dead_code)]
    #[cfg(target_arch = "wasm32")]
    last_mouse_position: Option<(f32, f32)>,
}

impl BindGroup for FreeCameraBindGroup {
    fn get_data(&self) -> Box<[u8]> {
        bytemuck::cast_slice(&[self.data]).into()
    }

    fn get_layout(&self, renderer: &mut Renderer) -> BindGroupLayout {
        let buffer_handle = renderer.create_uniform_buffer(&self.get_data());
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data(buffer_handle)],
        }
    }
}

impl Default for FreeCameraBindGroup {
    fn default() -> Self {
        let data = CameraData {
            view_position: [0.0; 4],
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        };

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
            data,
            #[cfg(target_arch = "wasm32")]
            last_mouse_position: None,
        }
    }
}

impl FreeCameraBindGroup {
    pub fn resize(&mut self, physical_size: UVec2) {
        self.perspective.aspect = physical_size.x as f32 / physical_size.y as f32;
    }

    pub fn update_data(&mut self) {
        let PerspectiveCameraData {
            position,
            direction,
            aspect,
            znear,
            zfar,
            fov,
            ..
        } = &self.perspective;

        self.data.view_proj = {
            let view = Mat4::look_at_rh(*position, *position + *direction, CAMERA_UP);
            let proj = Mat4::perspective_rh(*fov, *aspect, *znear, *zfar);
            OPENGL_TO_WGPU_MATRIX * proj * view
        }
        .to_cols_array_2d();
        self.data.view_position = [position.x, position.y, position.z, 1.0];
    }

    pub fn update(
        &mut self,
        events: &[WindowEvent],
        window: &dyn Window,
        resolution: UVec2,
        dt: f32,
    ) {
        self.resize(resolution);
        for event in events.iter() {
            self.controller.event(event);

            if let WindowEvent::MouseMotion((x, y)) = event {
                let x = *x;
                let y = *y;
                cfg_if::cfg_if! {
                if #[cfg(target_arch = "wasm32")]{
                    let last_mouse_position = self.last_mouse_position.unwrap_or((x, y));
                    let dx = x - last_mouse_position.0;
                    let dy = y - last_mouse_position.1;
                    self.last_mouse_position = Some((x, y));
                }else{
                    let dx = x- resolution.x as f32 / 2.0;
                    let dy = y- resolution.y as f32 / 2.0;
                    window
                        .set_cursor_position(
                            resolution.x / 2,
                            resolution.y / 2,
                        );
                }
                }

                self.controller.cursor_moved(dx, dy);
            }
        }

        self.controller.update(
            &mut self.perspective.position,
            &mut self.perspective.direction,
            dt,
        );

        self.update_data()
    }

    pub fn get_layout() -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data(BufferHandle(0))],
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

    pub fn update(&mut self, object_position: &mut Vec3, object_direction: &mut Vec3, dt: f32) {
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
        let yaw_rad = yaw * DEG_TO_RAD;
        let pitch_rad = pitch * DEG_TO_RAD;
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
