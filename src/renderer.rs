use crate::{
    implementation::renderer::wgpu::{render_pass::WGPURenderPass, WGPURenderer},
    window::{WindowHandle, WindowManager},
};

use super::{
    bind_group::BindGroup,
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
        pub type RenderPass<'renderer> = WGPURenderPass<'renderer>;
    }
    // else if #[cfg(feature="nekinovga")] {
    //     pub type Renderer = NekiNovga;
    // }
}

pub trait Janderer {
    #[allow(async_fn_in_trait)]
    #[allow(opaque_hidden_inferred_bound)]
    async fn new() -> Self;

    fn register_window(&mut self, handle: WindowHandle, window_manager: &mut WindowManager);

    fn resize(&mut self, handle: WindowHandle, width: u32, height: u32);

    fn create_uniform_buffer(&mut self, contents: &[u8]) -> BufferHandle;

    fn create_vertex_buffer(&mut self, contents: &[u8]) -> BufferHandle;

    fn create_index_buffer(&mut self, contents: &[u8]) -> BufferHandle;

    fn write_buffer(&mut self, buffer: BufferHandle, data: &[u8]);

    fn new_pass(&mut self, window_handle: WindowHandle) -> RenderPass;

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

    fn present(&mut self);
}
