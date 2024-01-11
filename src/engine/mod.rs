use crate::camera::Camera;
use crate::object::Object;
use crate::renderer::Renderer;

mod definition;

pub struct Engine {
    window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
    renderer: Renderer,
    objects: Vec<Object>,
    camera: Camera,
}
