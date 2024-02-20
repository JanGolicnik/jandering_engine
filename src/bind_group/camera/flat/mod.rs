use super::CameraRenderData;

mod definition;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlatCameraUniform {
    view_position: [f32; 2],
    resolution: [f32; 2],
}

pub struct FlatCameraController {
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

pub struct FlatCameraData {
    pub position: cgmath::Point2<f32>,
    pub resolution: [f32; 2],
}

pub struct FlatCameraBindGroup {
    pub data: FlatCameraData,
    //
    controller: Option<FlatCameraController>,
    //
    uniform: FlatCameraUniform,
    render_data: CameraRenderData,
    //
    last_mouse_position: Option<(f32, f32)>,
    pressing: bool,
    mouse_is_inside: bool,
}
