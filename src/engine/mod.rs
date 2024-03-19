use crate::{renderer::Renderer, types::Vec3};

mod definition;

pub struct Engine {
    pub window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
    pub renderer: Renderer,
    pub clear_color: Vec3,
}

pub struct EngineContext<'a> {
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub control_flow: &'a mut winit::event_loop::ControlFlow,
    pub surface_view: wgpu::TextureView,
    pub dt: f64,
    pub events: &'a [winit::event::WindowEvent<'a>],
    pub window: &'a winit::window::Window,
    pub resolution: (u32, u32),
}

pub struct EngineDescriptor {
    pub resolution: (u32, u32),
    pub clear_color: Vec3,
    pub show_cursor: bool,
}
