use super::{engine::EngineContext, renderer::Renderer};

pub trait EventHandler {
    fn on_update(&mut self, engine: &mut EngineContext);

    fn on_render(&mut self, renderer: &mut Box<Renderer>);
}
