use super::CameraUniform;
use crate::{bind_group::BindGroupRenderData, types::Vec3};

pub mod constants;
mod definition;

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
    uniform: CameraUniform,
    render_data: BindGroupRenderData,
    //
    #[allow(dead_code)]
    #[cfg(target_arch = "wasm32")]
    last_mouse_position: Option<(f32, f32)>,
}
