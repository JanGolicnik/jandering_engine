use jandering_engine::camera::{FreeCameraController, PerspectiveCameraData};

mod definition;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CustomCameraUniform {
    up: [f32; 4],
    right: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

pub struct CustomCameraPlugin {
    perspective: PerspectiveCameraData,
    //
    controller: FreeCameraController,
    //
    render_data: Option<CustomCameraRenderData>,
}

pub struct CustomCameraRenderData {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub uniform: CustomCameraUniform,
    pub buffer: wgpu::Buffer,
}
