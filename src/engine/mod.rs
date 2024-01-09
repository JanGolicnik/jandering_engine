use crate::renderer::Renderer;

mod definition;

pub struct Engine {
    window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
    renderer: Renderer,
}
