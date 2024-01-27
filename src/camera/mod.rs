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

pub struct FreeCameraController {
    pub right_pressed: bool,
    pub left_pressed: bool,
    pub forward_pressed: bool,
    pub backward_pressed: bool,
    pub is_shift_pressed: bool,
    pub speed_multiplier: f32,
    pub velocity: cgmath::Vector3<f32>,

    pub yaw: f32,
    pub pitch: f32,
}

pub struct PerspectiveCameraData {
    pub position: cgmath::Point3<f32>,
    pub direction: cgmath::Vector3<f32>,
    //
    pub fov: f32,
    pub znear: f32,
    pub zfar: f32,
    pub aspect: f32,
}

pub struct DefaultCameraPlugin {
    perspective: PerspectiveCameraData,
    //
    controller: FreeCameraController,
    //
    render_data: Option<CameraRenderData>,
    //
    #[cfg(target_arch = "wasm32")]
    last_mouse_position: Option<(f32, f32)>,
}
