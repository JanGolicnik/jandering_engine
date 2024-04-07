use std::collections::HashMap;

use crate::{
    core::{
        object::Renderable,
        renderer::{RenderPass, Renderer, ShaderHandle, UntypedBindGroupHandle},
    },
    types::Vec3,
};

pub struct WGPURenderPass<'renderer> {
    renderer: &'renderer mut Renderer,
    encoder: wgpu::CommandEncoder,
    shader: ShaderHandle,
    bind_groups: HashMap<u32, UntypedBindGroupHandle>,
    clear_color: Option<Vec3>,
    depth: Option<f32>,
}

impl<'renderer> WGPURenderPass<'renderer> {
    pub fn new(renderer: &'renderer mut Renderer) -> Self {
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
        let mut render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.renderer.surface_data.as_ref().unwrap().1,
                resolve_target: None,
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.renderer.textures[self.renderer.depth_texture.0].view,
                depth_ops: Some(wgpu::Operations {
                    load: if let Some(depth) = self.depth {
                        wgpu::LoadOp::Clear(depth)
                    } else {
                        wgpu::LoadOp::Load
                    },
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
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

    fn with_depth(mut self: Box<Self>, value: f32) -> Box<dyn RenderPass<'renderer> + 'renderer> {
        self.depth = Some(value);
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
}
