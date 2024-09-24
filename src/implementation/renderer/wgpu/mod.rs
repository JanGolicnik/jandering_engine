use std::{collections::HashMap, marker::PhantomData};

use compute_pass::WGPUComputePass;
use wgpu::{util::DeviceExt, ComputePipelineDescriptor, Features, PresentMode, SurfaceTexture};

use crate::{
    bind_group::{BindGroup, BindGroupLayoutEntry},
    engine::EngineConfig,
    render_pass::{RenderPass, RenderStep},
    renderer::{
        BindGroupHandle, BufferHandle, ComputeShaderHandle, Janderer, SamplerHandle, ShaderHandle,
        TextureHandle, UntypedBindGroupHandle,
    },
    shader::{ComputeShaderDescriptor, ShaderDescriptor},
    texture::{
        sampler::{
            SamplerAddressMode, SamplerCompareFunction, SamplerDescriptor, SamplerFilterMode,
        },
        texture_usage, Texture, TextureDescriptor, TextureFormat,
    },
    window::{WindowHandle, WindowManager, WindowManagerTrait, WindowTrait},
};

mod bind_groups;
pub mod compute_pass;

struct WGPUBindGroupRenderData {
    pub bind_group: wgpu::BindGroup,
    pub buffer_handle: BufferHandle,
}

pub struct WGPUShader {
    pub pipeline: wgpu::RenderPipeline,
}

pub struct WGPUComputeShader {
    pub pipeline: wgpu::ComputePipeline,
}

#[derive(Debug)]
pub(crate) struct Surface {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    surface_texture: Option<SurfaceTexture>,
}

pub struct WGPURenderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,

    pub(crate) surfaces: HashMap<WindowHandle, Surface>,

    pub(crate) shaders: Vec<WGPUShader>,
    shader_descriptors: Vec<ShaderDescriptor>,

    pub(crate) compute_shaders: Vec<WGPUComputeShader>,
    compute_shader_descriptors: Vec<ComputeShaderDescriptor>,

    bind_groups: Vec<Box<dyn BindGroup>>,
    bind_groups_render_data: Vec<WGPUBindGroupRenderData>,

    pub(crate) textures: Vec<Texture>,
    pub(crate) samplers: Vec<wgpu::Sampler>,

    pub(crate) buffers: Vec<wgpu::Buffer>,
}

impl Janderer for WGPURenderer {
    async fn new(config: EngineConfig) -> Self {
        // inits wgpu
        let instance = wgpu::Instance::default();

        // actual physical graphics card
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();

        let limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };

        let device_descriptor = {
            let mut features = Features::empty();
            if config.enable_compute {
                features.insert(Features::VERTEX_WRITABLE_STORAGE)
            };
            &wgpu::DeviceDescriptor {
                limits,
                features,
                ..Default::default()
            }
        };

        // device is logical graphics card and queue is used for executing command buffers
        let (device, queue) = adapter
            .request_device(device_descriptor, None)
            .await
            .unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,

            surfaces: HashMap::new(),

            textures: Vec::new(),
            samplers: Vec::new(),

            shaders: Vec::new(),
            shader_descriptors: Vec::new(),

            compute_shaders: Vec::new(),
            compute_shader_descriptors: Vec::new(),

            bind_groups: Vec::new(),
            bind_groups_render_data: Vec::new(),

            buffers: Vec::new(),
        }
    }

    fn register_window(&mut self, handle: WindowHandle, window_manager: &mut WindowManager) {
        self.surfaces.entry(handle).or_insert({
            let window = window_manager.get_window(handle).unwrap();
            let surface = unsafe { self.instance.create_surface(window) }.unwrap();
            let config = {
                let surface_capabilities = surface.get_capabilities(&self.adapter);

                let surface_format = surface_capabilities
                    .formats
                    .iter()
                    .copied()
                    .find(|f| f.is_srgb())
                    .unwrap_or(surface_capabilities.formats[0]);

                let present_mode = match window.get_fps_prefrence() {
                    crate::window::FpsPreference::Vsync => PresentMode::AutoVsync,
                    crate::window::FpsPreference::Exact(_)
                    | crate::window::FpsPreference::Uncapped => PresentMode::Immediate,
                };

                let (width, height) = {
                    let size = window.size();
                    (size.0, size.1)
                };
                wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: surface_format,
                    width,
                    height,
                    present_mode,
                    alpha_mode: surface_capabilities.alpha_modes[0],
                    view_formats: vec![],
                }
            };

            surface.configure(&self.device, &config);
            Surface {
                surface,
                config,
                surface_texture: None,
            }
        });
    }

    fn resize(&mut self, handle: WindowHandle, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.surfaces.entry(handle).and_modify(|surface| {
            if surface.config.width == width && surface.config.height == height {
                return;
            }

            surface.config.width = width;
            surface.config.height = height;
            surface.surface.configure(&self.device, &surface.config);
        });
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
        BufferHandle::uniform(self.buffers.len() - 1)
    }

    fn create_storage_buffer(&mut self, contents: &[u8]) -> BufferHandle {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents,
                usage: wgpu::BufferUsages::STORAGE,
            });

        self.buffers.push(buffer);
        BufferHandle::storage(self.buffers.len() - 1)
    }

    fn create_storage_buffer_with_size(&mut self, size: usize) -> BufferHandle {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        self.buffers.push(buffer);
        BufferHandle::storage(self.buffers.len() - 1)
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
        BufferHandle::uniform(self.buffers.len() - 1)
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
        BufferHandle::uniform(self.buffers.len() - 1)
    }

    fn write_buffer(&mut self, buffer: BufferHandle, data: &[u8]) {
        self.queue
            .write_buffer(&self.buffers[buffer.index], 0, data);
    }

    fn new_pass(&mut self, window_handle: WindowHandle) -> RenderPass {
        RenderPass::new(self, window_handle)
    }

    fn new_compute_pass(&mut self) -> WGPUComputePass {
        WGPUComputePass::new(self)
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
        let shader = self.device.create_shader_module(shader);

        let format = match desc.target_texture_format {
            TextureFormat::Rgba8U => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Bgra8U => wgpu::TextureFormat::Bgra8UnormSrgb,
            TextureFormat::F32 => wgpu::TextureFormat::R32Float,
            TextureFormat::Depth32F => wgpu::TextureFormat::Depth32Float,
            TextureFormat::Depth16U => wgpu::TextureFormat::Depth16Unorm,
        };

        let targets = &[Some(wgpu::ColorTargetState {
            format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let attributes = desc
            .descriptors
            .iter()
            .map(Self::get_buffer_attributes)
            .collect::<Vec<_>>();
        let buffers = Self::get_buffer_layouts(&attributes, &desc.descriptors);

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
        self.create_shader_at(self.shader_descriptors[handle.0].clone(), handle);
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
            TextureFormat::F32 => (wgpu::TextureFormat::R32Float, 1),
            TextureFormat::Depth32F => (wgpu::TextureFormat::Depth32Float, 1),
            TextureFormat::Depth16U => (wgpu::TextureFormat::Depth16Unorm, 1),
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
            label: Some(desc.name),
            size,
            mip_level_count: 1,
            sample_count: desc.sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
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
            .write_buffer(&self.buffers[render_data.buffer_handle.index], 0, data);
    }

    fn create_bind_group_at(
        &mut self,
        bind_group: Box<dyn BindGroup>,
        handle: UntypedBindGroupHandle,
    ) {
        {
            let layout = bind_group.get_layout();
            let bind_group_layout = Self::get_layout(&self.device, &layout);
            let mut first_handle = BufferHandle::uniform(0); // TODO FIX THIS LMAO
            let entries = layout
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: match entry {
                        BindGroupLayoutEntry::Data(handle) => {
                            first_handle = *handle;
                            self.buffers[first_handle.index].as_entire_binding()
                        }
                        BindGroupLayoutEntry::Texture { handle, .. } => {
                            wgpu::BindingResource::TextureView(&self.textures[handle.0].view)
                        }
                        BindGroupLayoutEntry::Sampler { handle, .. } => {
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

    fn present(&mut self) {
        self.surfaces.iter_mut().for_each(|(_, surface)| {
            if let Some(e) = surface.surface_texture.take() {
                e.present()
            }
        });
    }

    fn create_compute_shader_at(
        &mut self,
        desc: crate::shader::ComputeShaderDescriptor,
        handle: ComputeShaderHandle,
    ) {
        let crate::shader::ShaderSource::Code(code) = &desc.source;

        let shader_desc = wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(code.clone().into()),
        };
        let shader = self.device.create_shader_module(shader_desc);

        //ugly ass code fuck off
        let bind_group_layouts = Self::get_layouts(&self.device, &desc.bind_group_layouts);
        let bind_group_ref = bind_group_layouts.iter().collect::<Vec<_>>();

        let layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &bind_group_ref,
                push_constant_ranges: &[],
            });

        let pipeline = self
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("Compute Pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: desc.entry,
            });

        let shader = WGPUComputeShader { pipeline };

        if handle.0 >= self.compute_shaders.len() {
            self.compute_shaders.push(shader);
            self.compute_shader_descriptors.push(desc);
        } else {
            self.compute_shaders[handle.0] = shader;
        }
    }

    fn create_compute_shader(
        &mut self,
        desc: crate::shader::ComputeShaderDescriptor,
    ) -> ComputeShaderHandle {
        self.create_compute_shader_at(desc, ComputeShaderHandle(self.compute_shaders.len()));
        ComputeShaderHandle(self.compute_shaders.len() - 1)
    }

    fn re_create_compute_shader(&mut self, handle: ComputeShaderHandle) {
        let descriptor = &self.compute_shader_descriptors[handle.0];
        self.create_compute_shader_at(descriptor.clone(), handle);
    }

    fn re_create_compute_shaders(&mut self) {
        for i in 0..self.compute_shaders.len() {
            self.re_create_compute_shader(ComputeShaderHandle(i));
        }
    }
}

impl WGPURenderer {
    pub fn submit_pass(pass: RenderPass) {
        let RenderPass {
            renderer,
            window_handle,
            steps,
        } = pass;

        let mut encoder = renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let surface_texture_view = {
            let surface = renderer.surfaces.get_mut(&window_handle).unwrap();

            let surface_texture = if let Some(surface_texture) = &surface.surface_texture {
                surface_texture
            } else {
                let surface_texture = match surface.surface.get_current_texture() {
                    Ok(surface) => surface,
                    Err(e) => {
                        panic!("{e}");
                    }
                };
                surface.surface_texture = Some(surface_texture);
                surface.surface_texture.as_ref().unwrap()
            };

            surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default())
        };

        let mut render_pass = None;
        let previous_target = None;

        let len = steps.len() - 1;
        for (i, step) in steps.iter().enumerate().take(len) {
            let RenderStep {
                action,
                shader,
                bind_groups,
                target,
                depth_tex,
                resolve_target,
                alpha,
                depth,
                clear_color,
            } = step;

            let mut changed = render_pass.is_none() || previous_target != *target;

            if i > 0 {
                if let Some(prev) = steps.get(i - 1) {
                    changed |= *depth_tex != prev.depth_tex;
                }
            }

            if changed {
                drop(render_pass);

                let (view, resolve_target) = if let Some(tex) = target {
                    let view = &renderer.textures[tex.0].view;
                    let resolve_target = Some(
                        resolve_target
                            .map(|tex| &renderer.textures[tex.0].view)
                            .unwrap_or(&surface_texture_view),
                    );
                    (view, resolve_target)
                } else {
                    (&surface_texture_view, None)
                };

                let depth_stencil_attachment =
                    depth_tex.map(|tex| wgpu::RenderPassDepthStencilAttachment {
                        view: &renderer.textures[tex.0].view,
                        depth_ops: Some(wgpu::Operations {
                            load: if let Some(depth) = depth {
                                wgpu::LoadOp::Clear(*depth)
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    });

                let new_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target,
                        ops: wgpu::Operations {
                            load: if let Some(color) = clear_color {
                                wgpu::LoadOp::Clear(wgpu::Color {
                                    r: color.x as f64 * *alpha as f64,
                                    g: color.y as f64 * *alpha as f64,
                                    b: color.z as f64 * *alpha as f64,
                                    a: *alpha as f64,
                                })
                            } else {
                                wgpu::LoadOp::Load
                            },
                            ..Default::default()
                        },
                    })],
                    depth_stencil_attachment,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                render_pass = Some(new_pass);
            }

            let render_pass = render_pass.as_mut().unwrap();

            if let Some(shader) = shader {
                let shader = renderer.shaders.get(shader.0).unwrap();
                render_pass.set_pipeline(&shader.pipeline);
            }

            for (index, handle) in bind_groups.iter() {
                render_pass.set_bind_group(
                    *index,
                    &renderer.bind_groups_render_data[handle.0].bind_group,
                    &[],
                );
            }

            match action {
                crate::render_pass::RenderAction::Mesh {
                    vertex_buffer_handle,
                    index_buffer_handle,
                    instance_buffer_handle,
                    range,
                    num_indices,
                } => {
                    render_pass.set_vertex_buffer(
                        0,
                        renderer.buffers[vertex_buffer_handle.index].slice(..),
                    );
                    if let Some(instance_buffer_handle) = instance_buffer_handle {
                        render_pass.set_vertex_buffer(
                            1,
                            renderer.buffers[instance_buffer_handle.index].slice(..),
                        );
                    }
                    render_pass.set_index_buffer(
                        renderer.buffers[index_buffer_handle.index].slice(..),
                        wgpu::IndexFormat::Uint32,
                    );

                    render_pass.draw_indexed(0..*num_indices, 0, range.clone());
                }
                crate::render_pass::RenderAction::Empty => {}
            }
        }

        if render_pass.is_some() {
            drop(render_pass);
            renderer.queue.submit(std::iter::once(encoder.finish()));
        }
    }
}
