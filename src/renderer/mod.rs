mod definition;

pub struct Renderer {
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pub queue: wgpu::Queue,
}
