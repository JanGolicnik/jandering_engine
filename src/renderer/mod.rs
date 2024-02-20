use crate::{bind_group::BindGroup, texture::Texture};

mod definition;

pub type TextureHandle = usize;

pub struct BindGroupHandle<T>(usize, std::marker::PhantomData<T>);

#[derive(Copy, Clone)]
pub struct UntypedBindGroupHandle(usize);

pub struct Renderer {
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pub queue: wgpu::Queue,
    surface_view: Option<wgpu::TextureView>,
    custom_view: Option<TextureHandle>,
    textures: Vec<Texture>,
    bind_groups: Vec<Box<dyn BindGroup>>,
}
