use crate::{
    bind_group::BindGroupLayout,
    engine::EngineConfig,
    implementation::renderer::wgpu::{compute_pass::WGPUComputePass, WGPURenderer},
    render_pass::RenderPass,
    shader::ComputeShaderDescriptor,
    window::Window,
};

use super::{
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BindGroupHandle(pub usize);

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

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum TargetTexture {
    #[default]
    Screen,
    Handle(TextureHandle),
    None,
}

pub trait Janderer {
    #[allow(async_fn_in_trait)]
    #[allow(opaque_hidden_inferred_bound)]
    async fn new(config: EngineConfig) -> Self;

    fn register_window(&mut self, window: &Window);

    fn resize(&mut self, window: &Window, width: u32, height: u32);

    // rendering
    fn new_compute_pass(&mut self) -> ComputePass;

    fn submit_pass(&mut self, pass: RenderPass);

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

    fn reload_shader(&mut self, handle: ShaderHandle);

    fn reload_shaders(&mut self);

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

    fn clear_texture(&mut self, texture: TextureHandle);

    // bind group
    fn create_bind_group_at(&mut self, layout: BindGroupLayout, handle: BindGroupHandle);

    fn create_bind_group(&mut self, layout: BindGroupLayout) -> BindGroupHandle;
}
