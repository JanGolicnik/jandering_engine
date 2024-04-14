use std::{marker::PhantomData, ops::Range};

use crate::types::UVec2;

use super::{
    bind_group::BindGroup,
    object::Renderable,
    shader::ShaderDescriptor,
    texture::{sampler::SamplerDescriptor, Texture, TextureDescriptor},
};

#[derive(Copy, Clone)]
pub struct BufferHandle(pub usize);

#[derive(Copy, Clone)]
pub struct TextureHandle(pub usize);

#[derive(Copy, Clone)]
pub struct SamplerHandle(pub usize);

pub struct BindGroupHandle<T>(pub usize, pub std::marker::PhantomData<T>);

#[derive(Copy, Clone)]
pub struct UntypedBindGroupHandle(pub usize);

#[derive(Copy, Clone)]
pub struct ShaderHandle(pub usize);

pub trait Renderer {
    fn resize(&mut self, width: u32, height: u32);

    fn set_width(&mut self, width: u32);

    fn set_height(&mut self, height: u32);

    fn size(&self) -> UVec2;

    fn create_uniform_buffer(&mut self, contents: &[u8]) -> BufferHandle;

    fn create_vertex_buffer(&mut self, contents: &[u8]) -> BufferHandle;

    fn create_index_buffer(&mut self, contents: &[u8]) -> BufferHandle;

    fn write_buffer(&mut self, buffer: BufferHandle, data: &[u8]);

    fn begin_frame(&mut self) -> bool;

    fn present(&mut self);

    fn new_pass<'renderer>(&'renderer mut self) -> Box<dyn RenderPass + 'renderer>;

    fn create_shader_at(&mut self, desc: ShaderDescriptor, handle: ShaderHandle);

    fn create_shader(&mut self, desc: ShaderDescriptor) -> ShaderHandle;

    fn re_create_shader(&mut self, handle: ShaderHandle);

    fn re_create_shaders(&mut self);

    fn create_texture_at(&mut self, desc: TextureDescriptor, handle: TextureHandle);

    fn create_texture(&mut self, desc: TextureDescriptor) -> TextureHandle;

    fn re_create_texture(&mut self, desc: TextureDescriptor, handle: TextureHandle);

    fn add_texture(&mut self, texture: Texture) -> TextureHandle;

    fn get_texture(&self, handle: TextureHandle) -> Option<&Texture>;

    fn create_sampler(&mut self, desc: SamplerDescriptor) -> SamplerHandle;

    fn create_bind_group(&mut self, bind_group: Box<dyn BindGroup>) -> UntypedBindGroupHandle;

    fn get_bind_group(&self, handle: UntypedBindGroupHandle) -> Option<&dyn BindGroup>;

    fn get_bind_group_mut(&mut self, handle: UntypedBindGroupHandle) -> Option<&mut dyn BindGroup>;

    fn write_bind_group(&mut self, handle: UntypedBindGroupHandle, data: &[u8]);
}

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

pub fn create_typed_bind_group<T: BindGroup>(
    renderer: &mut dyn Renderer,
    bind_group: T,
) -> BindGroupHandle<T> {
    let handle = renderer.create_bind_group(Box::new(bind_group));
    BindGroupHandle(handle.0, PhantomData::<T>)
}

pub fn get_typed_bind_group<T: BindGroup>(
    renderer: &dyn Renderer,
    handle: BindGroupHandle<T>,
) -> Option<&T> {
    if let Some(b) = renderer.get_bind_group(handle.into()) {
        let any = b.as_any();
        if let Some(bind_group) = any.downcast_ref::<T>() {
            return Some(bind_group);
        }
    }
    None
}

pub fn get_typed_bind_group_mut<T: BindGroup>(
    renderer: &mut dyn Renderer,
    handle: BindGroupHandle<T>,
) -> Option<&mut T> {
    if let Some(b) = renderer.get_bind_group_mut(handle.into()) {
        let any = b.as_any_mut();
        if let Some(bind_group) = any.downcast_mut::<T>() {
            return Some(bind_group);
        }
    }
    None
}
