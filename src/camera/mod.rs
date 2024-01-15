pub mod constants;
mod definition;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

pub struct CameraRenderData {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
}

pub struct CameraControllerData {
    right_pressed: bool,
    left_pressed: bool,
    forward_pressed: bool,
    backward_pressed: bool,
    is_shift_pressed: bool,
    speed_multiplier: f32,
}

pub struct DefaultCameraPlugin {
    pub eye: cgmath::Point3<f32>,
    pub direction: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,
    //
    fov: f32,
    znear: f32,
    zfar: f32,
    aspect: f32,
    //
    velocity: cgmath::Vector3<f32>,
    controller: CameraControllerData,
    //
    render_data: Option<CameraRenderData>,
}
