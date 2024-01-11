mod constants;
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

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    //
    fov: f32,
    znear: f32,
    zfar: f32,
    aspect: f32,
    //
    render_data: Option<CameraRenderData>,
}
