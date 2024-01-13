use winit::window::Window;

use crate::{
    camera::CameraRenderData,
    object::{InstanceRaw, Object, VertexRaw},
};

use super::Renderer;

impl Renderer {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

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
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            config,
            size,
            queue,
            deafult_pipeline: None,
        }
    }

    pub fn add_default_pipeline(&mut self, camera: &CameraRenderData) {
        let deafult_pipeline = {
            let layout = self
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Default Shader Layout"),
                    bind_group_layouts: &[&camera.bind_group_layout],
                    push_constant_ranges: &[],
                });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Default Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("default_shader.wgsl").into()),
            };
            let shader = self.device.create_shader_module(shader);
            self.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[VertexRaw::desc(), InstanceRaw::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: self.config.format,
                            blend: Some(wgpu::BlendState {
                                alpha: wgpu::BlendComponent::REPLACE,
                                color: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                })
        };

        self.deafult_pipeline = Some(deafult_pipeline);
    }

    pub(crate) fn render(
        &self,
        objects: &[Object],
        camera: &CameraRenderData,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.5,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if let Some(default_pipeline) = &self.deafult_pipeline {
                render_pass.set_pipeline(default_pipeline);
            }
            self.queue
                .write_buffer(&camera.buffer, 0, bytemuck::cast_slice(&[camera.uniform]));
            render_pass.set_bind_group(0, &camera.bind_group, &[]);

            for object in objects {
                if object.pipeline.is_none() {
                    continue;
                }
                let pipeline = object.pipeline.as_ref().unwrap();

                self.queue.write_buffer(
                    &pipeline.instance_buffer,
                    0,
                    bytemuck::cast_slice(&object.instance_data),
                );

                render_pass.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, pipeline.instance_buffer.slice(..));
                render_pass
                    .set_index_buffer(pipeline.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(
                    0..object.indices.len() as u32,
                    0,
                    0..object.instances.len() as u32,
                );
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
