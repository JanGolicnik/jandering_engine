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
    pub velocity: Vec2,
    pub pan_offset: Vec2,
}

pub struct D2CameraBindGroup {
    pub position: Vec2,
    pub resolution: Vec2,
    //
    pub controller: Option<D2CameraController>,
    //
    uniform: FlatCameraUniform,
    render_data: BindGroupRenderData,
    //
    last_mouse_position: Option<(f32, f32)>,
    pressing: bool,
    mouse_is_inside: bool,
    pub right_click_move: bool,
}
