use wgpu::util::DeviceExt;

use crate::{engine::EngineContext, plugins::Plugin, renderer::Renderer};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct TimePluginUniform {
    dt: f32,
    time: f32,
    #[cfg(target_arch = "wasm32")]
    padding: [f32; 2],
}

struct TimePluginRenderData {
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

pub struct TimePlugin {
    uniform: TimePluginUniform,
    render_data: TimePluginRenderData,
}

impl Plugin for TimePlugin {
    fn get_bind_group_layout(&self) -> Option<&wgpu::BindGroupLayout> {
        Some(&self.render_data.bind_group_layout)
    }

    fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        Some(&self.render_data.bind_group)
    }

    fn update(&mut self, context: &mut EngineContext, renderer: &mut Renderer) {
        self.uniform.time += context.dt as f32;
        self.uniform.dt = context.dt as f32;

        renderer.queue.write_buffer(
            &self.render_data.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }
}

impl TimePlugin {
    pub fn new(renderer: &Renderer) -> Self {
        let uniform = TimePluginUniform {
            dt: 0.0,
            time: 0.0,
            #[cfg(target_arch = "wasm32")]
            padding: [0.0; 2],
        };

        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Time Buffer"),
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
                    label: Some("time_bind_group_layout"),
                });

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("time_bind_group"),
            });

        Self {
            uniform,
            render_data: TimePluginRenderData {
                buffer,
                bind_group_layout,
                bind_group,
            },
        }
    }
}
