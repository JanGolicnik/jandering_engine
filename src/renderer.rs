use crate::{
    engine::EngineConfig,
    implementation::renderer::wgpu::{compute_pass::WGPUComputePass, WGPURenderer},
    render_pass::RenderPass,
    shader::ComputeShaderDescriptor,
    window::{WindowHandle, WindowManager},
};

use super::{
    bind_group::BindGroup,
    shader::ShaderDescriptor,
    texture::{sampler::SamplerDescriptor, Texture, TextureDescriptor},
};

#[derive(Copy, Clone, Debug)]
pub(crate) enum BufferType {
    Uniform,
    Storage,
}

#[derive(Copy, Clone, Debug)]
pub struct BufferHandle {
    pub(crate) buffer_type: BufferType,
    pub(crate) index: usize,
}

impl BufferHandle {
    pub fn uniform(index: usize) -> Self {
        Self {
            buffer_type: BufferType::Uniform,
            index,
        }
    }

    pub fn storage(index: usize) -> Self {
        Self {
            buffer_type: BufferType::Storage,
            index,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextureHandle(pub usize);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SamplerHandle(pub usize);

pub struct BindGroupHandle<T>(pub usize, pub std::marker::PhantomData<T>);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UntypedBindGroupHandle(pub usize);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ShaderHandle(pub usize);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ComputeShaderHandle(pub usize);

cfg_if::cfg_if! {
    if #[cfg(feature = "wgpu")] {
        pub type Renderer = WGPURenderer;
        pub type ComputePass<'renderer> = WGPUComputePass<'renderer>;
    }
    // else if #[cfg(feature="nekinovga")] {
    //     pub type Renderer = NekiNovga;
    // }
}

pub trait Janderer {
    #[allow(async_fn_in_trait)]
    #[allow(opaque_hidden_inferred_bound)]
    async fn new(config: EngineConfig) -> Self;

    fn register_window(&mut self, handle: WindowHandle, window_manager: &mut WindowManager);

    fn resize(&mut self, handle: WindowHandle, width: u32, height: u32);

    // rendering
    fn new_pass(&mut self, window_handle: WindowHandle) -> RenderPass;

    fn new_compute_pass(&mut self) -> ComputePass;

    fn present(&mut self);

    // buffers
    fn create_uniform_buffer(&mut self, contents: &[u8]) -> BufferHandle;

    fn create_storage_buffer_with_size(&mut self, size: usize) -> BufferHandle;

    fn create_storage_buffer(&mut self, contents: &[u8]) -> BufferHandle;

    fn create_vertex_buffer(&mut self, contents: &[u8]) -> BufferHandle;

    fn create_index_buffer(&mut self, contents: &[u8]) -> BufferHandle;

    fn write_buffer(&mut self, buffer: BufferHandle, data: &[u8]);

    //shaders
    fn create_shader_at(&mut self, desc: ShaderDescriptor, handle: ShaderHandle);

    fn create_shader(&mut self, desc: ShaderDescriptor) -> ShaderHandle;

    fn re_create_shader(&mut self, handle: ShaderHandle);

    fn re_create_shaders(&mut self);

    // compute
    fn create_compute_shader_at(
        &mut self,
        desc: ComputeShaderDescriptor,
        handle: ComputeShaderHandle,
    );

    fn create_compute_shader(&mut self, desc: ComputeShaderDescriptor) -> ComputeShaderHandle;

    fn re_create_compute_shader(&mut self, handle: ComputeShaderHandle);

    fn re_create_compute_shaders(&mut self);

    //textures
    fn create_texture_at(&mut self, desc: TextureDescriptor, handle: TextureHandle);

    fn create_texture(&mut self, desc: TextureDescriptor) -> TextureHandle;

    fn re_create_texture(&mut self, desc: TextureDescriptor, handle: TextureHandle);

    fn add_texture(&mut self, texture: Texture) -> TextureHandle;

    fn get_texture(&self, handle: TextureHandle) -> Option<&Texture>;

    fn create_sampler(&mut self, desc: SamplerDescriptor) -> SamplerHandle;

    // bind group
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
}
