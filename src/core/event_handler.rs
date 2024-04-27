use async_trait::async_trait;

use super::{engine::EngineContext, renderer::Renderer};

#[async_trait]
pub trait EventHandler {
    fn on_update(&mut self, engine: &mut EngineContext<'_>);

    fn on_render(&mut self, renderer: &mut Box<dyn Renderer>);
}
