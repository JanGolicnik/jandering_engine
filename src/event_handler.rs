use crate::window::{WindowHandle, WindowManager};

use super::{engine::EngineContext, renderer::Renderer};

#[async_trait::async_trait]
pub trait EventHandler {
    fn init(&mut self, renderer: &mut Renderer, window_manager: &mut WindowManager);

    fn on_update(&mut self, context: &mut EngineContext<'_>);

    fn on_render(
        &mut self,
        renderer: &mut Renderer,
        window_handle: WindowHandle,
        window_manager: &mut WindowManager,
    );
}
