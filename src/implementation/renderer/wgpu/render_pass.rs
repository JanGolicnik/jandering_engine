use crate::{
    object::Renderable,
    render_pass::{RenderPassData, RenderPassTrait},
};

use super::WGPURenderer;

pub struct WGPURenderPass<'renderer> {
    renderer: &'renderer mut WGPURenderer,
    surface_texture_view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,

    data: RenderPassData,
}

impl<'renderer> WGPURenderPass<'renderer> {
    pub fn new(
        renderer: &'renderer mut WGPURenderer,
        surface_texture_view: wgpu::TextureView,
    ) -> Self {
        let encoder = renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        Self {
            renderer,
            surface_texture_view,
            encoder,

            data: RenderPassData::default(),
        }
    }
}

impl<'renderer> RenderPassTrait for WGPURenderPass<'renderer> {
    fn submit(self) {
        self.renderer
            .queue
            .submit(std::iter::once(self.encoder.finish()));
    }
    fn render_range(mut self, renderable: &dyn Renderable, mut range: std::ops::Range<u32>) -> Self
    where
        Self: Sized,
    {
        let RenderPassData {
            shader,
            bind_groups,
            clear_color,
            depth,
            depth_tex,
            target,
            resolve_target,
            alpha,
        } = &self.data;

        if range.start + range.len() as u32 > renderable.num_instances() {
            range.end = renderable.num_instances();
        }

        let (view, resolve_target) = if let Some(tex) = target {
            let view = &self.renderer.textures[tex.0].view;
            let resolve_target = Some(
                resolve_target
                    .map(|tex| &self.renderer.textures[tex.0].view)
                    .unwrap_or(&self.surface_texture_view),
            );
            (view, resolve_target)
        } else {
            (&self.surface_texture_view, None)
        };

        let depth_stencil_attachment =
            depth_tex.map(|tex| wgpu::RenderPassDepthStencilAttachment {
                view: &self.renderer.textures[tex.0].view,
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

        let mut render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        let shader = self.renderer.shaders.get(shader.0).unwrap();
        render_pass.set_pipeline(&shader.pipeline);

        for (index, handle) in bind_groups.iter() {
            render_pass.set_bind_group(
                *index,
                &self.renderer.bind_groups_render_data[handle.0].bind_group,
                &[],
            );
        }

        let (vertex_buffer_handle, index_buffer_handle, instance_buffer) = renderable.get_buffers();

        render_pass.set_vertex_buffer(
            0,
            self.renderer.buffers[vertex_buffer_handle.index].slice(..),
        );
        if let Some(instance_buffer_handle) = instance_buffer {
            render_pass.set_vertex_buffer(
                1,
                self.renderer.buffers[instance_buffer_handle.index].slice(..),
            );
        }
        render_pass.set_index_buffer(
            self.renderer.buffers[index_buffer_handle.index].slice(..),
            wgpu::IndexFormat::Uint32,
        );

        if range.start + range.len() as u32 > renderable.num_instances() {
            range.end = renderable.num_instances();
        }

        render_pass.draw_indexed(0..renderable.num_indices(), 0, range);

        drop(render_pass);

        self.data.clear_color = None;
        self.data.depth = None;

        self
    }

    fn get_data(&mut self) -> &mut RenderPassData {
        &mut self.data
    }

    fn render_empty(mut self) -> Self
    where
        Self: Sized,
    {
        let RenderPassData {
            clear_color,
            alpha,
            target,
            resolve_target,
            ..
        } = &self.data;

        let (view, resolve_target) = if let Some(tex) = target {
            let view = &self.renderer.textures[tex.0].view;
            let resolve_target = Some(
                resolve_target
                    .map(|tex| &self.renderer.textures[tex.0].view)
                    .unwrap_or(&self.surface_texture_view),
            );
            (view, resolve_target)
        } else {
            (&self.surface_texture_view, None)
        };

        self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        self
    }
}
