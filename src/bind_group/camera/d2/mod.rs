use crate::{bind_group::BindGroupRenderData, types::Vec2};

mod definition;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlatCameraUniform {
    view_position: [f32; 2],
    resolution: [f32; 2],
}

pub struct D2CameraController {
    pub right_pressed: bool,
    pub left_pressed: bool,
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub is_mouse_pressed: bool,
    pub is_shift_pressed: bool,
    pub zoom: f32,
    pub velocity: cgmath::Vector2<f32>,
    pub pan_offset: cgmath::Vector2<f32>,
}

pub struct D2CameraData {
    pub position: Vec2,
    pub resolution: Vec2,
}

pub struct D2CameraBindGroup {
    pub data: D2CameraData,
    //
    controller: Option<D2CameraController>,
    //
    uniform: FlatCameraUniform,
    render_data: BindGroupRenderData,
    //
    last_mouse_position: Option<(f32, f32)>,
    pressing: bool,
    mouse_is_inside: bool,
}
