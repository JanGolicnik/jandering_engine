use wgpu::util::DeviceExt;

use crate::plugins::Plugin;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ResolutionPluginUniform {
    resolution: [f32; 2],
    #[cfg(target_arch = "wasm32")]
    padding: [f32; 2],
}

struct ResolutionPluginRenderData {
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

pub struct ResolutionPlugin {
    uniform: ResolutionPluginUniform,
    render_data: ResolutionPluginRenderData,
}

impl Plugin for ResolutionPlugin {
    fn get_bind_group_layout(&self) -> Option<&wgpu::BindGroupLayout> {
        Some(&self.render_data.bind_group_layout)
    }

    fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        Some(&self.render_data.bind_group)
    }

    fn update(
        &mut self,
        _context: &mut crate::engine::EngineContext,
        renderer: &mut crate::renderer::Renderer,
    ) {
        self.uniform.resolution[0] = renderer.config.width as f32;
        self.uniform.resolution[1] = renderer.config.height as f32;

        renderer.queue.write_buffer(
            &self.render_data.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}

impl ResolutionPlugin {
    pub fn new(renderer: &crate::renderer::Renderer) -> Self {
        let uniform = ResolutionPluginUniform {
            resolution: [renderer.config.width as f32, renderer.config.height as f32],
            #[cfg(target_arch = "wasm32")]
            padding: [0.0; 2],
        };

        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Resolution Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            renderer
                .device
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
                    label: Some("resolution_bind_group_layout"),
                });

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("resolution_bind_group"),
            });

        Self {
            uniform,
            render_data: ResolutionPluginRenderData {
                buffer,
                bind_group_layout,
                bind_group,
            },
        }
    }
}
