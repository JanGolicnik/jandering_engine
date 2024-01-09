mod definition;

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    queue: wgpu::Queue,
}
