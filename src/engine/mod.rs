use crate::renderer::Renderer;

mod definition;

pub struct Engine {
    pub window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
    pub renderer: Renderer,
}

pub struct EngineContext<'a> {
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub surface: &'a mut wgpu::SurfaceTexture,
    pub control_flow: &'a mut winit::event_loop::ControlFlow,
    pub dt: f64,
    pub events: &'a [winit::event::WindowEvent<'a>],
    pub window: &'a winit::window::Window,
}

pub struct EngineDescriptor {
    pub resolution: (u32, u32),
}
