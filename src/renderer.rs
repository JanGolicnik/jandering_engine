use std::ops::Range;

use crate::{implementation::renderer::wgpu::WGPURenderer, types::UVec2, window::Window};

use super::{
    bind_group::BindGroup,
    object::Renderable,
    shader::ShaderDescriptor,
    texture::{sampler::SamplerDescriptor, Texture, TextureDescriptor},
};

#[derive(Copy, Clone, Debug)]
pub struct BufferHandle(pub usize);

#[derive(Copy, Clone, Debug)]
pub struct TextureHandle(pub usize);

#[derive(Copy, Clone, Debug)]
pub struct SamplerHandle(pub usize);

pub struct BindGroupHandle<T>(pub usize, pub std::marker::PhantomData<T>);

#[derive(Copy, Clone, Debug)]
pub struct UntypedBindGroupHandle(pub usize);

#[derive(Copy, Clone, Debug)]
pub struct ShaderHandle(pub usize);

cfg_if::cfg_if! {
    if #[cfg(feature = "wgpu")] {
        pub type Renderer = WGPURenderer;
    }
    // else if #[cfg(feature="nekinovga")] {
    //     pub type Renderer = NekiNovga;
    // }
}

pub trait Janderer {
    #[allow(async_fn_in_trait)]
    #[allow(opaque_hidden_inferred_bound)]
    async fn new(window: &Window) -> Self;

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

    fn create_bind_group_at(
        &mut self,
        bind_group: Box<dyn BindGroup>,
        handle: UntypedBindGroupHandle,
    );

    fn create_bind_group(&mut self, bind_group: Box<dyn BindGroup>) -> UntypedBindGroupHandle;

    fn get_bind_group(&self, handle: UntypedBindGroupHandle) -> Option<&dyn BindGroup>;

    fn get_bind_group_mut(&mut self, handle: UntypedBindGroupHandle) -> Option<&mut dyn BindGroup>;

    fn create_typed_bind_group_at<T: BindGroup>(
        &mut self,
        bind_group: T,
        handle: BindGroupHandle<T>,
    );

    fn create_typed_bind_group<T: BindGroup>(&mut self, bind_group: T) -> BindGroupHandle<T>;

    fn get_typed_bind_group<T: BindGroup>(&self, handle: BindGroupHandle<T>) -> Option<&T>;

    fn get_typed_bind_group_mut<T: BindGroup>(
        &mut self,
        handle: BindGroupHandle<T>,
    ) -> Option<&mut T>;

    fn write_bind_group(&mut self, handle: UntypedBindGroupHandle, data: &[u8]);

    fn max_texture_size(&self) -> UVec2;
}

pub trait RenderPass<'renderer> {
    fn render(
        self: Box<Self>,
        renderables: &[&dyn Renderable],
    ) -> Box<dyn RenderPass<'renderer> + 'renderer>;

    fn render_one(
        self: Box<Self>,
        renderable: &dyn Renderable,
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
    #[cfg(not(target_arch = "wasm32"))]
    fn with_target_texture_resolve(
        self: Box<Self>,
        target: TextureHandle,
        resolve: Option<TextureHandle>,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer>;
}
