use crate::{
    core::{
        object::Renderable,
        renderer::{RenderPass, ShaderHandle, TextureHandle, UntypedBindGroupHandle},
    },
    types::Vec3,
};

use std::collections::HashMap;

use super::WGPURenderer;

pub struct WGPURenderPass<'renderer> {
    renderer: &'renderer mut WGPURenderer,
    encoder: wgpu::CommandEncoder,
    shader: ShaderHandle,
    bind_groups: HashMap<u32, UntypedBindGroupHandle>,
    clear_color: Option<Vec3>,
    depth: Option<f32>,
    depth_tex: Option<TextureHandle>,
    target: Option<TextureHandle>,
    resolve_target: Option<TextureHandle>,
}

impl<'renderer> WGPURenderPass<'renderer> {
    pub fn new(renderer: &'renderer mut WGPURenderer) -> Self {
        let encoder = renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        Self {
            renderer,
            encoder,
            shader: ShaderHandle(0),
            bind_groups: HashMap::new(),
            clear_color: None,
            depth: None,
            depth_tex: None,
            target: None,
            resolve_target: None,
        }
    }
}

impl<'renderer> RenderPass<'renderer> for WGPURenderPass<'renderer> {
    fn render(
        self: Box<Self>,
        renderables: &[&dyn Renderable],
    ) -> Box<dyn RenderPass<'renderer> + 'renderer> {
        let mut ret: Box<dyn RenderPass<'renderer> + 'renderer> = self;
        for renderable in renderables {
            ret = ret.render_range(*renderable, 0..renderable.num_instances());
        }
        ret
    }

    fn render_range(
        mut self: Box<Self>,
        renderable: &dyn Renderable,
        mut range: std::ops::Range<u32>,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer> {
        if range.start + range.len() as u32 > renderable.num_instances() {
            range.end = renderable.num_instances();
        }

        let (view, resolve_target) = if let Some(tex) = self.target {
            let view = &self.renderer.textures[tex.0].view;
            let resolve_target = Some(
                self.resolve_target
                    .map(|tex| &self.renderer.textures[tex.0].view)
                    .unwrap_or(&self.renderer.surface_data.as_ref().unwrap().1),
            );
            (view, resolve_target)
        } else {
            (&self.renderer.surface_data.as_ref().unwrap().1, None)
        };

        let depth_stencil_attachment =
            self.depth_tex
                .map(|tex| wgpu::RenderPassDepthStencilAttachment {
                    view: &self.renderer.textures[tex.0].view,
                    depth_ops: Some(wgpu::Operations {
                        load: if let Some(depth) = self.depth {
                            wgpu::LoadOp::Clear(depth)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                });

        let mut render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target,
                ops: wgpu::Operations {
                    load: if let Some(color) = self.clear_color {
                        wgpu::LoadOp::Clear(wgpu::Color {
                            r: color.x as f64,
                            g: color.y as f64,
                            b: color.z as f64,
                            a: 1.0,
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

        let shader = self.renderer.shaders.get(self.shader.0).unwrap();
        render_pass.set_pipeline(&shader.pipeline);

        for (index, handle) in self.bind_groups.iter() {
            render_pass.set_bind_group(
                *index,
                &self.renderer.bind_groups_render_data[handle.0].bind_group,
                &[],
            );
        }

        let (vertex_buffer_handle, index_buffer_handle, instance_buffer) = renderable.get_buffers();

        render_pass.set_vertex_buffer(0, self.renderer.buffers[vertex_buffer_handle.0].slice(..));
        if let Some(instance_buffer_handle) = instance_buffer {
            render_pass
                .set_vertex_buffer(1, self.renderer.buffers[instance_buffer_handle.0].slice(..));
        }
        render_pass.set_index_buffer(
            self.renderer.buffers[index_buffer_handle.0].slice(..),
            wgpu::IndexFormat::Uint32,
        );

        if range.start + range.len() as u32 > renderable.num_instances() {
            range.end = renderable.num_instances();
        }

        render_pass.draw_indexed(0..renderable.num_indices(), 0, range);

        drop(render_pass);

        self.clear_color = None;
        self.depth = None;

        self
    }

    fn bind(
        mut self: Box<Self>,
        slot: u32,
        handle: crate::core::renderer::UntypedBindGroupHandle,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer> {
        self.bind_groups
            .entry(slot)
            .and_modify(|e| *e = handle)
            .or_insert(handle);
        self
    }

    fn submit(self: Box<Self>) {
        self.renderer
            .queue
            .submit(std::iter::once(self.encoder.finish()));
    }

    fn set_shader(
        mut self: Box<Self>,
        shader: crate::core::renderer::ShaderHandle,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer> {
        self.shader = shader;
        self
    }

    fn with_depth(
        mut self: Box<Self>,
        handle: TextureHandle,
        value: Option<f32>,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer> {
        self.depth_tex = Some(handle);
        self.depth = value;
        self
    }

    fn with_clear_color(
        mut self: Box<Self>,
        r: f32,
        g: f32,
        b: f32,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer> {
        self.clear_color = Some(Vec3::new(r, g, b));
        self
    }

    fn unbind(mut self: Box<Self>, slot: u32) -> Box<dyn RenderPass<'renderer> + 'renderer> {
        self.bind_groups.remove(&slot);
        self
    }

    fn with_target_texture_resolve(
        mut self: Box<Self>,
        target: TextureHandle,
        resolve: Option<TextureHandle>,
    ) -> Box<dyn RenderPass<'renderer> + 'renderer> {
        self.target = Some(target);
        self.resolve_target = resolve;
        self
    }
}
