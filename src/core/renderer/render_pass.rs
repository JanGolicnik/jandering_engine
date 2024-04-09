use std::ops::Range;

use crate::{core::object::Renderable, implementation::renderer::wgpu::WGPURenderPass};

use super::{Renderer, ShaderHandle, TextureHandle, UntypedBindGroupHandle};

pub trait RenderPass<'renderer> {
    fn render(
        self: Box<Self>,
        renderables: &[&dyn Renderable],
    ) -> Box<dyn RenderPass<'renderer> + 'renderer>;

    fn render_range(
        self: Box<Self>,
        renderables: &dyn Renderable,
        range: Range<u32>,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer>;

    fn bind(
        self: Box<Self>,
        slot: u32,
        bind_group: UntypedBindGroupHandle,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer>;

    fn unbind(self: Box<Self>, slot: u32) -> Box<dyn RenderPass<'renderer> + 'renderer>;

    fn submit(self: Box<Self>);

    fn set_shader(
        self: Box<Self>,
        shader: ShaderHandle,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer>;

    fn with_depth(
        self: Box<Self>,
        texture: TextureHandle,
        value: Option<f32>,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer>;

    fn with_clear_color(
        self: Box<Self>,
        r: f32,
        g: f32,
        b: f32,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer>;

    //  None for resolve target means use canvas
    fn with_target_texture_resolve(
        self: Box<Self>,
        target: TextureHandle,
        resolve: Option<TextureHandle>,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer>;
}

impl Renderer {
    pub fn new_pass<'renderer>(&'renderer mut self) -> Box<dyn RenderPass + 'renderer> {
        Box::new(WGPURenderPass::new(self))
    }
}
