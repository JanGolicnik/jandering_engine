use std::ops::Range;

use wgpu::util::DeviceExt;

use crate::{implementation::renderer::wgpu::WGPURenderPass, types::UVec2};

use super::{
    bind_group::{BindGroup, BindGroupLayout, BindGroupLayoutEntry},
    object::Renderable,
    shader::{Shader, ShaderDescriptor},
    texture::Texture,
    window::Window,
};

pub struct Buffer {
    pub buffer: wgpu::Buffer,
}

pub struct BindGroupRenderData {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffer: Buffer,
}

pub struct TextureHandle(usize);

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    config: wgpu::SurfaceConfiguration,
    pub queue: wgpu::Queue,
    pub clear_color: (f32, f32, f32),
    current_frame: Option<(
        wgpu::CommandEncoder,
        wgpu::SurfaceTexture,
        wgpu::TextureView,
    )>,
    shaders: Vec<Shader>,
    bind_groups: Vec<Box<dyn BindGroup>>,
    bind_groups_render_data: Vec<BindGroupRenderData>,
    depth_texture: TextureHandle,
    textures: Vec<Texture>,
}

pub struct BindGroupHandle<T>(usize, std::marker::PhantomData<T>);

#[derive(Copy, Clone)]
pub struct UntypedBindGroupHandle(usize);

#[derive(Copy, Clone)]
pub struct ShaderHandle(usize);

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

        let textures = vec![Self::create_depth_texture(
            &device,
            UVec2::new(config.width, config.height),
        )];

        Self {
            surface,
            device,
            config,
            queue,
            textures,
            depth_texture: TextureHandle(0),
            clear_color: (0.0, 0.0, 0.0),
            current_frame: None,
            shaders: Vec::new(),
            bind_groups: Vec::new(),
            bind_groups_render_data: Vec::new(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 && (width != self.config.width || height != self.config.height) {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.textures[0] = Self::create_depth_texture(
                &self.device,
                UVec2::new(self.config.width, self.config.height),
            );
        }
    }

    pub fn set_width(&mut self, width: u32) {
        self.resize(width, self.height())
    }

    pub fn set_height(&mut self, height: u32) {
        self.resize(self.width(), height)
    }

    pub fn width(&self) -> u32 {
        self.config.width
    }

    pub fn height(&self) -> u32 {
        self.config.height
    }

    pub fn new_pass<'a>(&'a mut self, shader: ShaderHandle) -> Box<dyn RenderPass + 'a> {
        Box::new(WGPURenderPass::new(self))
    }

    pub fn create_uniform_buffer(&self, contents: &[u8]) -> Buffer {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        Buffer { buffer }
    }

    pub fn create_vertex_buffer(&self, contents: &[u8]) -> Buffer {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        Buffer { buffer }
    }

    pub fn create_index_buffer(&self, contents: &[u8]) -> Buffer {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

        Buffer { buffer }
    }

    pub fn write_buffer(&mut self, buffer: &Buffer, data: &[u8]) {
        self.queue.write_buffer(&buffer.buffer, 0, data);
    }

    pub fn create_shader(&mut self, desc: ShaderDescriptor) -> ShaderHandle {
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

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(desc.code.into()),
        };

        let targets = &[Some(wgpu::ColorTargetState {
            format: self.config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let shader = self.device.create_shader_module(shader);
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: desc.vs_entry,
                    buffers: desc.descriptors,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: desc.fs_entry,
                    targets,
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
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
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });
        self.shaders.push(Shader { pipeline });

        ShaderHandle(self.shaders.len() - 1)
    }

    pub fn create_bind_group<T: BindGroup>(&mut self, bind_group: T) -> BindGroupHandle<T> {
        {
            let buffer = self.create_uniform_buffer(&bind_group.get_data());

            let bind_group_layout =
                self.device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                        label: Some("bind_group_layout"),
                    });

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.buffer.as_entire_binding(),
                }],
                label: Some("bind_group"),
            });

            self.bind_groups_render_data.push(BindGroupRenderData {
                bind_group_layout,
                bind_group,
                buffer,
            });
        }

        self.bind_groups.push(Box::new(bind_group));
        BindGroupHandle(self.bind_groups.len() - 1, std::marker::PhantomData::<T>)
    }

    pub fn get_bind_group(&self, handle: UntypedBindGroupHandle) -> Option<&dyn BindGroup> {
        if let Some(b) = self.bind_groups.get(handle.0) {
            Some(b.as_ref())
        } else {
            None
        }
    }

    pub fn get_bind_group_mut(
        &mut self,
        handle: UntypedBindGroupHandle,
    ) -> Option<&mut dyn BindGroup> {
        if let Some(b) = self.bind_groups.get_mut(handle.0) {
            Some(b.as_mut())
        } else {
            None
        }
    }

    pub fn get_bind_group_t<T: BindGroup>(&self, handle: BindGroupHandle<T>) -> Option<&T> {
        if let Some(b) = self.bind_groups.get(handle.0) {
            let b = b.as_ref();
            let any = b.as_any();
            if let Some(bind_group) = any.downcast_ref::<T>() {
                return Some(bind_group);
            }
        }
        None
    }

    pub fn get_bind_group_t_mut<T: BindGroup>(
        &mut self,
        handle: BindGroupHandle<T>,
    ) -> Option<&mut T> {
        if let Some(b) = self.bind_groups.get_mut(handle.0) {
            let b = b.as_mut();
            let any = b.as_any_mut();
            if let Some(bind_group) = any.downcast_mut::<T>() {
                return Some(bind_group);
            }
        }
        None
    }

    pub fn get_layouts(
        device: &wgpu::Device,
        layouts: &[BindGroupLayout],
    ) -> Vec<wgpu::BindGroupLayout> {
        layouts
            .iter()
            .map(|e| {
                let entries: Vec<_> = e
                    .entries
                    .iter()
                    .map(|e| match e {
                        BindGroupLayoutEntry::Data => wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry::Texture => wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry::Sampler => wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    })
                    .collect();

                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &entries,
                    label: Some("bind_group_layout"),
                })
            })
            .collect()
    }

    pub fn write_bind_group(&mut self, handle: UntypedBindGroupHandle, data: &[u8]) {
        self.queue.write_buffer(
            &self.bind_groups_render_data[handle.0].buffer.buffer,
            0,
            data,
        );
    }

    fn create_depth_texture(device: &wgpu::Device, size: UVec2) -> Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Texture {
            texture,
            sampler,
            view,
            width: size.x,
            height: size.y,
        }
    }
}

pub trait RenderPass<'a> {
    fn render(&mut self, renderables: &[&dyn Renderable]) -> Box<Renderer>;

    fn bind(self: Box<Self>, bind_group: UntypedBindGroupHandle) -> Box<dyn RenderPass<'a>>;
}

impl<T> Clone for BindGroupHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for BindGroupHandle<T> {}

impl<T> From<BindGroupHandle<T>> for UntypedBindGroupHandle {
    fn from(value: BindGroupHandle<T>) -> Self {
        Self(value.0)
    }
}

impl<T> From<&BindGroupHandle<T>> for UntypedBindGroupHandle {
    fn from(value: &BindGroupHandle<T>) -> Self {
        Self(value.0)
    }
}

impl<T> std::fmt::Debug for BindGroupHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.0, std::any::type_name::<T>())
    }
}
