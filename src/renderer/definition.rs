use std::{fmt, ops::Range};

use winit::window::Window;

use crate::{
    bind_group::{BindGroup, BindGroupWriteData},
    engine::EngineContext,
    object::{self, Renderable},
    shader::Shader,
    texture::Texture,
};

use super::{BindGroupHandle, Renderer, TextureHandle, UntypedBindGroupHandle};

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
            surface_view: None,
            custom_view: None,
            textures: Vec::new(),
            bind_groups: Vec::new(),
        }
    }

    pub fn begin_frame(
        &mut self,
    ) -> Result<
        (
            wgpu::CommandEncoder,
            wgpu::TextureView,
            wgpu::SurfaceTexture,
        ),
        wgpu::SurfaceError,
    > {
        let surface = self.surface.get_current_texture()?;

        let surface_view = surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let view = if let Some(view) = &self.surface_view {
            view
        } else {
            &surface_view
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.015,
                        g: 0.007,
                        b: 0.045,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        Ok((encoder, surface_view, surface))
    }

    pub fn new_pass(
        &mut self,
        view: &wgpu::TextureView,
    ) -> Result<wgpu::CommandEncoder, wgpu::SurfaceError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.015,
                        g: 0.007,
                        b: 0.045,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        Ok(encoder)
    }

    pub fn render_with_range<T: object::Renderable>(
        &mut self,
        renderable: &T,
        context: &mut EngineContext,
        shader: &Shader,
        bind_groups: &[UntypedBindGroupHandle],
        mut range: Range<u32>,
    ) {
        let data = BindGroupWriteData {
            queue: &self.queue,
            config: &self.config,
            context,
        };
        bind_groups
            .iter()
            .for_each(|i| self.bind_groups[i.0].write(&data));

        let view = if let Some(handle) = self.custom_view {
            &self.get_texture(handle).unwrap().view
        } else {
            &context.surface_view
        };

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        ..Default::default()
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

        render_pass.set_pipeline(&shader.pipeline);

        for (index, handle) in bind_groups.iter().enumerate() {
            let bind_group = self.get_bind_group(*handle).unwrap();
            render_pass.set_bind_group(index as u32, bind_group.get_bind_group().unwrap(), &[]);
        }

        if range.start + range.len() as u32 > renderable.num_instances() {
            range.end = renderable.num_instances();
        }
        renderable.bind(&mut render_pass, range);
    }

    pub fn render<T: Renderable>(
        &mut self,
        renderables: &[&T],
        context: &mut EngineContext,
        shader: &Shader,
        bind_groups: &[UntypedBindGroupHandle],
    ) {
        for renderable in renderables {
            self.render_with_range(
                *renderable,
                context,
                shader,
                bind_groups,
                0..renderable.num_instances(),
            )
        }
    }

    pub fn submit(&mut self, encoder: wgpu::CommandEncoder, surface: wgpu::SurfaceTexture) {
        self.queue.submit(std::iter::once(encoder.finish()));
        surface.present();
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn get_texture_mut(&mut self, handle: TextureHandle) -> Option<&mut Texture> {
        self.textures.get_mut(handle)
    }

    pub fn get_texture(&self, handle: TextureHandle) -> Option<&Texture> {
        self.textures.get(handle)
    }

    pub fn add_texture(&mut self, texture: Texture) -> TextureHandle {
        self.textures.push(texture);
        self.textures.len() - 1
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

    pub fn get_bind_group_t<T>(&self, handle: BindGroupHandle<T>) -> Option<&T>
    where
        T: BindGroup,
    {
        if let Some(b) = self.bind_groups.get(handle.0) {
            let b = b.as_ref();
            let any = b.as_any();
            match any.downcast_ref::<T>() {
                Some(bind_group) => Some(bind_group),
                None => None,
            }
        } else {
            None
        }
    }

    pub fn get_bind_group_t_mut<T>(&mut self, handle: BindGroupHandle<T>) -> Option<&mut T>
    where
        T: BindGroup,
    {
        if let Some(b) = self.bind_groups.get_mut(handle.0) {
            let b = b.as_mut();
            let any = b.as_any_mut();
            match any.downcast_mut::<T>() {
                Some(bind_group) => Some(bind_group),
                None => None,
            }
        } else {
            None
        }
    }

    pub fn add_bind_group<T: BindGroup>(&mut self, bind_group: T) -> BindGroupHandle<T>
    where
        T: BindGroup,
    {
        self.bind_groups.push(Box::new(bind_group));
        BindGroupHandle(self.bind_groups.len() - 1, std::marker::PhantomData::<T>)
    }

    pub fn clear_texture(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        handle: TextureHandle,
        color: wgpu::Color,
    ) {
        let view = &self.get_texture(handle).unwrap().view;
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
    }

    pub fn set_render_target(&mut self, handle: TextureHandle) {
        self.custom_view = Some(handle);
    }

    pub fn set_target_surface(&mut self) {
        self.custom_view = None;
    }
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

impl<T> fmt::Debug for BindGroupHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.0, std::any::type_name::<T>())
    }
}
