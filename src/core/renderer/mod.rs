use wgpu::util::DeviceExt;

use crate::types::UVec2;

use super::{bind_group::BindGroup, shader::Shader, texture::Texture, window::Window};

mod bind_groups;
pub mod render_pass;
mod shaders;
mod textures;

#[derive(Copy, Clone)]
pub struct BufferHandle(pub usize);

pub struct BindGroupRenderData {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffer_handle: BufferHandle,
}

#[derive(Copy, Clone)]
pub struct TextureHandle(pub usize);

#[derive(Copy, Clone)]
pub struct SamplerHandle(pub usize);

pub struct Renderer {
    pub(crate) surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    config: wgpu::SurfaceConfiguration,
    pub queue: wgpu::Queue,
    pub(crate) shaders: Vec<Shader>,
    bind_groups: Vec<Box<dyn BindGroup>>,
    pub(crate) bind_groups_render_data: Vec<BindGroupRenderData>,
    pub(crate) textures: Vec<Texture>,
    pub(crate) samplers: Vec<wgpu::Sampler>,
    pub(crate) buffers: Vec<wgpu::Buffer>,
    pub(crate) surface_data: Option<(wgpu::SurfaceTexture, wgpu::TextureView)>,
}

pub struct BindGroupHandle<T>(usize, std::marker::PhantomData<T>);

#[derive(Copy, Clone)]
pub struct UntypedBindGroupHandle(pub usize);

#[derive(Copy, Clone)]
pub struct ShaderHandle(pub usize);

impl Renderer {
    #[allow(clippy::borrowed_box)]
    pub async fn new(window: &Box<dyn Window>) -> Self {
        let (width, height) = {
            let size = window.size();
            (size.0, size.1)
        };

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window.as_ref()) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            config,
            queue,
            textures: Vec::new(),
            samplers: Vec::new(),
            shaders: Vec::new(),
            bind_groups: Vec::new(),
            bind_groups_render_data: Vec::new(),
            buffers: Vec::new(),
            surface_data: None,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            dbg!("configured");
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn set_width(&mut self, width: u32) {
        self.resize(width, self.size().y)
    }

    pub fn set_height(&mut self, height: u32) {
        self.resize(self.size().x, height)
    }

    pub fn size(&self) -> UVec2 {
        UVec2::new(self.config.width, self.config.height)
    }

    pub fn create_uniform_buffer(&mut self, contents: &[u8]) -> BufferHandle {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        self.buffers.push(buffer);
        BufferHandle(self.buffers.len() - 1)
    }

    pub fn create_vertex_buffer(&mut self, contents: &[u8]) -> BufferHandle {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        self.buffers.push(buffer);
        BufferHandle(self.buffers.len() - 1)
    }

    pub fn create_index_buffer(&mut self, contents: &[u8]) -> BufferHandle {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        self.buffers.push(buffer);
        BufferHandle(self.buffers.len() - 1)
    }

    pub fn write_buffer(&mut self, buffer: BufferHandle, data: &[u8]) {
        self.queue.write_buffer(&self.buffers[buffer.0], 0, data);
    }

    pub fn begin_frame(&mut self) -> bool {
        let surface = match self.surface.get_current_texture() {
            Ok(surface) => surface,
            Err(e) => {
                panic!("{e}");
            }
        };

        let surface_view = surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.surface_data = Some((surface, surface_view));

        true
    }

    pub fn present(&mut self) {
        let surface_data = self.surface_data.take();
        surface_data.unwrap().0.present();
    }
}
