use crate::plugins::Plugin;
use crate::renderer::Renderer;

mod default_plugins;
mod definition;

pub struct Engine {
    pub window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
    pub renderer: Renderer,
    pub plugins: Vec<Box<dyn Plugin>>,
    pub shaders: Vec<wgpu::RenderPipeline>,
}
