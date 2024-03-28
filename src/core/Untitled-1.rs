
pub fn begin_frame(&mut self) {
    let surface = match self.surface.get_current_texture() {
        Ok(surface) => surface,
        Err(e) => {
            panic!("{e}");
        }
    };

    let surface_view = surface
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let encoder = self
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    self.current_frame = Some((encoder, surface, surface_view));
}

pub fn render_with_range<T: Renderable>(
    &mut self,
    renderable: &T,
    shader: ShaderHandle,
    bind_groups: &[UntypedBindGroupHandle],
    mut range: Range<u32>,
) {
    if let Some((encoder, _, view)) = &mut self.current_frame {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    ..Default::default()
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.textures[self.depth_texture.0].view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        {
            let shader = self.shaders.get(shader.0).unwrap();
            render_pass.set_pipeline(&shader.pipeline);
        }

        for (index, handle) in bind_groups.iter().enumerate() {
            render_pass.set_bind_group(
                index as u32,
                &self.bind_groups_render_data[handle.0].bind_group,
                &[],
            );
        }

        if range.start + range.len() as u32 > renderable.num_instances() {
            range.end = renderable.num_instances();
        }

        // renderable.bind(WGPURenderPass { render_pass }, range);
    }
}

pub fn render<T: Renderable>(
    &mut self,
    renderables: &[&T],
    bind_groups: &[UntypedBindGroupHandle],
    shader: ShaderHandle,
) {
    for renderable in renderables {
        self.render_with_range(
            *renderable,
            shader,
            bind_groups,
            0..renderable.num_instances(),
        )
    }
}

// pub fn new_pass(&mut self) -> Box<dyn RenderPass + '_> {
//     let (encoder, _, view) = self.current_frame.as_mut().unwrap();

//     Box::new(WGPURenderPass { render_pass })
// }

pub fn submit(&mut self) {
    if let Some((encoder, surface, ..)) = self.current_frame.take() {
        self.queue.submit(std::iter::once(encoder.finish()));
        surface.present();
    }
}
