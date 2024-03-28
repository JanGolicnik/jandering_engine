use crate::core::{
    object::Renderable,
    renderer::{RenderPass, Renderer},
};

pub struct WGPURenderPass<'a> {
    renderer: &'a mut Renderer,
}

impl<'a> WGPURenderPass<'a> {
    pub fn new(renderer: &'a mut Renderer) -> Self {
        Self { renderer }
    }
}

impl<'a> RenderPass<'a> for WGPURenderPass<'a> {
    fn render(&mut self, renderables: &[&dyn Renderable]) -> Box<Renderer> {
        todo!()
    }

    fn bind(
        self: Box<Self>,
        bind_group: crate::core::renderer::UntypedBindGroupHandle,
    ) -> Box<dyn RenderPass<'a>> {
        self
    }
}
