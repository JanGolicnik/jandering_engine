use std::marker::PhantomData;

use render_pass::WGPURenderPass;
use wgpu::{util::DeviceExt, PresentMode};

use crate::{
    bind_group::{BindGroup, BindGroupLayoutEntry},
    renderer::{
        BindGroupHandle, BufferHandle, Janderer, RenderPass, SamplerHandle, ShaderHandle,
        TextureHandle, UntypedBindGroupHandle,
    },
    shader::ShaderDescriptor,
    texture::{
        sampler::{
            SamplerAddressMode, SamplerCompareFunction, SamplerDescriptor, SamplerFilterMode,
        },
        texture_usage, Texture, TextureDescriptor, TextureFormat,
    },
    types::UVec2,
    window::{Window, WindowTrait},
};

mod bind_groups;
pub mod render_pass;

struct WGPUBindGroupRenderData {
    pub bind_group: wgpu::BindGroup,
    pub buffer_handle: BufferHandle,
}

pub struct WGPUShader {
    pub pipeline: wgpu::RenderPipeline,
}

pub struct WGPURenderer {
    pub(crate) surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    config: wgpu::SurfaceConfiguration,
    pub queue: wgpu::Queue,
    pub(crate) shaders: Vec<WGPUShader>,
    shader_descriptors: Vec<ShaderDescriptor>,
    bind_groups: Vec<Box<dyn BindGroup>>,
    bind_groups_render_data: Vec<WGPUBindGroupRenderData>,
    pub(crate) textures: Vec<Texture>,
    pub(crate) samplers: Vec<wgpu::Sampler>,
    pub(crate) buffers: Vec<wgpu::Buffer>,
    pub(crate) surface_data: Option<(wgpu::SurfaceTexture, wgpu::TextureView)>,
    limits: wgpu::Limits,
}

impl Drop for WGPURenderer {
    fn drop(&mut self) {
        log::info!("HEYY")
    }
}

impl Janderer for WGPURenderer {
    async fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };

        let (width, height) = {
            let size = window.size();
            (size.0, size.1)
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: limits.clone(),
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

        let present_mode = match window.get_fps_prefrence() {
            crate::window::FpsPreference::Vsync => PresentMode::AutoVsync,
            crate::window::FpsPreference::Exact(_) | crate::window::FpsPreference::Uncapped => {
                PresentMode::Immediate
            }
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
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
            shader_descriptors: Vec::new(),
            bind_groups: Vec::new(),
            bind_groups_render_data: Vec::new(),
            buffers: Vec::new(),
            surface_data: None,
            limits,
        }
    }
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn set_width(&mut self, width: u32) {
        self.resize(width, self.size().y)
    }

    fn set_height(&mut self, height: u32) {
        self.resize(self.size().x, height)
    }

    fn size(&self) -> UVec2 {
        UVec2::new(self.config.width, self.config.height)
    }

    fn create_uniform_buffer(&mut self, contents: &[u8]) -> BufferHandle {
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

    fn create_vertex_buffer(&mut self, contents: &[u8]) -> BufferHandle {
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

    fn create_index_buffer(&mut self, contents: &[u8]) -> BufferHandle {
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

    fn write_buffer(&mut self, buffer: BufferHandle, data: &[u8]) {
        self.queue.write_buffer(&self.buffers[buffer.0], 0, data);
    }

    fn begin_frame(&mut self) -> bool {
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

    fn present(&mut self) {
        let surface_data = self.surface_data.take();
        surface_data.unwrap().0.present();
    }

    fn new_pass<'renderer>(&'renderer mut self) -> Box<dyn RenderPass + 'renderer> {
        Box::new(WGPURenderPass::new(self))
    }

    fn create_shader_at(&mut self, desc: ShaderDescriptor, handle: ShaderHandle) {
        //ugly ass code fuck off
        let bind_group_layouts = Self::get_layouts(&self.device, &desc.bind_group_layouts);
        let bind_group_ref = bind_group_layouts.iter().collect::<Vec<_>>();

        let layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shader Layout"),
                bind_group_layouts: &bind_group_ref,
                push_constant_ranges: &[],
            });

        let crate::shader::ShaderSource::Code(code) = &desc.source;

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(code.clone().into()),
        };

        let targets = &[Some(wgpu::ColorTargetState {
            format: self.config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let attributes = desc
            .descriptors
            .iter()
            .map(Self::get_buffer_attributes)
            .collect::<Vec<_>>();
        let buffers = Self::get_buffer_layouts(&attributes, &desc.descriptors);

        let shader = self.device.create_shader_module(shader);
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: desc.vs_entry,
                    buffers: &buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: desc.fs_entry,
                    targets,
                }),
                primitive: wgpu::PrimitiveState {
                    topology: if desc.stripped {
                        wgpu::PrimitiveTopology::TriangleStrip
                    } else {
                        wgpu::PrimitiveTopology::TriangleList
                    },
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: if desc.backface_culling {
                        Some(wgpu::Face::Back)
                    } else {
                        None
                    },
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: if desc.depth {
                    Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    })
                } else {
                    None
                },
                multisample: wgpu::MultisampleState {
                    count: desc.multisample,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        let shader = WGPUShader { pipeline };

        if handle.0 >= self.shaders.len() {
            self.shaders.push(shader);
            self.shader_descriptors.push(desc);
        } else {
            self.shaders[handle.0] = shader;
        }
    }

    fn create_shader(&mut self, desc: ShaderDescriptor) -> ShaderHandle {
        self.create_shader_at(desc, ShaderHandle(self.shaders.len()));
        ShaderHandle(self.shaders.len() - 1)
    }

    fn re_create_shader(&mut self, handle: ShaderHandle) {
        let descriptor = &self.shader_descriptors[handle.0];
        self.create_shader_at(descriptor.clone(), handle);
    }

    fn re_create_shaders(&mut self) {
        for i in 0..self.shaders.len() {
            self.re_create_shader(ShaderHandle(i));
        }
    }

    fn create_texture_at(&mut self, desc: TextureDescriptor, handle: TextureHandle) {
        let size = wgpu::Extent3d {
            width: desc.size.x,
            height: desc.size.y,
            depth_or_array_layers: 1,
        };

        let (format, channels) = match desc.format {
            TextureFormat::Rgba8U => (wgpu::TextureFormat::Rgba8UnormSrgb, 4),
            TextureFormat::Bgra8U => (wgpu::TextureFormat::Bgra8UnormSrgb, 4),
            TextureFormat::Depth32F => (wgpu::TextureFormat::Depth32Float, 1),
        };

        let mut usage = wgpu::TextureUsages::empty();
        if desc.usage & texture_usage::BIND != texture_usage::NONE {
            usage |= wgpu::TextureUsages::TEXTURE_BINDING;
        }
        if desc.usage & texture_usage::COPY_SRC != texture_usage::NONE {
            usage |= wgpu::TextureUsages::COPY_SRC;
        }
        if desc.usage & texture_usage::COPY_TARGET != texture_usage::NONE {
            usage |= wgpu::TextureUsages::COPY_DST;
        }
        if desc.usage & texture_usage::TARGET != texture_usage::NONE {
            usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        }

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size,
            mip_level_count: 1,
            sample_count: desc.sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        if let Some(data) = desc.data {
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(channels * size.width),
                    rows_per_image: Some(size.height),
                },
                size,
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture = Texture {
            texture,
            view,
            width: desc.size.x,
            height: desc.size.y,
        };

        if handle.0 >= self.textures.len() {
            self.textures.push(texture);
        } else {
            self.textures[handle.0] = texture;
        }
    }

    fn create_texture(&mut self, desc: TextureDescriptor) -> TextureHandle {
        self.create_texture_at(desc, TextureHandle(self.textures.len()));
        TextureHandle(self.textures.len() - 1)
    }

    fn re_create_texture(&mut self, desc: TextureDescriptor, handle: TextureHandle) {
        self.create_texture_at(desc, handle);
    }

    fn add_texture(&mut self, texture: Texture) -> TextureHandle {
        self.textures.push(texture);
        TextureHandle(self.textures.len() - 1)
    }

    fn get_texture(&self, handle: TextureHandle) -> Option<&Texture> {
        self.textures.get(handle.0)
    }

    fn create_sampler(&mut self, desc: SamplerDescriptor) -> SamplerHandle {
        let address_mode = match desc.address_mode {
            SamplerAddressMode::Repeat => wgpu::AddressMode::Repeat,
            SamplerAddressMode::RepeatMirrored => wgpu::AddressMode::MirrorRepeat,
            SamplerAddressMode::Clamp => wgpu::AddressMode::ClampToEdge,
        };
        let filter = match desc.filter {
            SamplerFilterMode::Linear => wgpu::FilterMode::Linear,
            SamplerFilterMode::Nearest => wgpu::FilterMode::Nearest,
        };
        let lod_min_clamp = desc.lod_min_clamp;
        let lod_max_clamp = desc.lod_max_clamp;
        let compare = desc.compare.map(|e| match e {
            SamplerCompareFunction::Less => wgpu::CompareFunction::Less,
            SamplerCompareFunction::LessEqual => wgpu::CompareFunction::LessEqual,
            SamplerCompareFunction::Equal => wgpu::CompareFunction::Equal,
            SamplerCompareFunction::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            SamplerCompareFunction::Greater => wgpu::CompareFunction::Greater,
        });

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: filter,
            lod_min_clamp,
            lod_max_clamp,
            compare,
            ..Default::default()
        });

        self.samplers.push(sampler);
        SamplerHandle(self.samplers.len() - 1)
    }

    fn get_bind_group(&self, handle: UntypedBindGroupHandle) -> Option<&dyn BindGroup> {
        if let Some(b) = self.bind_groups.get(handle.0) {
            Some(b.as_ref())
        } else {
            None
        }
    }

    fn get_bind_group_mut(&mut self, handle: UntypedBindGroupHandle) -> Option<&mut dyn BindGroup> {
        if let Some(b) = self.bind_groups.get_mut(handle.0) {
            Some(b.as_mut())
        } else {
            None
        }
    }

    fn write_bind_group(&mut self, handle: UntypedBindGroupHandle, data: &[u8]) {
        let render_data = &self.bind_groups_render_data[handle.0];
        self.queue
            .write_buffer(&self.buffers[render_data.buffer_handle.0], 0, data);
    }

    fn create_bind_group_at(
        &mut self,
        bind_group: Box<dyn BindGroup>,
        handle: UntypedBindGroupHandle,
    ) {
        {
            let layout = bind_group.get_layout(self);
            let bind_group_layout = Self::get_layout(&self.device, &layout);
            let mut first_handle = BufferHandle(0); // TODO FIX THIS LMAO
            let entries = layout
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: match entry {
                        BindGroupLayoutEntry::Data(handle) => {
                            first_handle = *handle;
                            self.buffers[handle.0].as_entire_binding()
                        }
                        BindGroupLayoutEntry::Texture(handle) => {
                            wgpu::BindingResource::TextureView(&self.textures[handle.0].view)
                        }
                        BindGroupLayoutEntry::Sampler(handle) => {
                            wgpu::BindingResource::Sampler(&self.samplers[handle.0])
                        }
                    },
                })
                .collect::<Vec<_>>();

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &entries[..],
                label: Some("bind_group"),
            });

            let data = WGPUBindGroupRenderData {
                bind_group,
                buffer_handle: first_handle,
            };

            if handle.0 >= self.bind_groups.len() {
                self.bind_groups_render_data.push(data);
            } else {
                self.bind_groups_render_data[handle.0] = data;
            }
        }

        if handle.0 >= self.bind_groups.len() {
            self.bind_groups.push(bind_group);
        } else {
            self.bind_groups[handle.0] = bind_group;
        }
    }

    fn create_bind_group(&mut self, bind_group: Box<dyn BindGroup>) -> UntypedBindGroupHandle {
        let handle = UntypedBindGroupHandle(self.bind_groups.len());
        self.create_bind_group_at(bind_group, handle);
        handle
    }

    fn create_typed_bind_group_at<T: BindGroup>(
        &mut self,
        bind_group: T,
        handle: BindGroupHandle<T>,
    ) {
        self.create_bind_group_at(Box::new(bind_group), handle.into());
    }

    fn create_typed_bind_group<T: BindGroup>(&mut self, bind_group: T) -> BindGroupHandle<T> {
        let handle = self.create_bind_group(Box::new(bind_group));
        BindGroupHandle(handle.0, PhantomData::<T>)
    }

    fn get_typed_bind_group<T: BindGroup>(&self, handle: BindGroupHandle<T>) -> Option<&T> {
        if let Some(b) = self.get_bind_group(handle.into()) {
            let any = b.as_any();
            if let Some(bind_group) = any.downcast_ref::<T>() {
                return Some(bind_group);
            }
        }
        None
    }

    fn get_typed_bind_group_mut<T: BindGroup>(
        &mut self,
        handle: BindGroupHandle<T>,
    ) -> Option<&mut T> {
        if let Some(b) = self.get_bind_group_mut(handle.into()) {
            let any = b.as_any_mut();
            if let Some(bind_group) = any.downcast_mut::<T>() {
                return Some(bind_group);
            }
        }
        None
    }

    fn max_texture_size(&self) -> UVec2 {
        UVec2::splat(self.limits.max_texture_dimension_2d)
    }
}
