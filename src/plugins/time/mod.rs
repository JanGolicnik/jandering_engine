use crate::{plugins::Plugin, renderer::Renderer};
use wgpu::util::DeviceExt;

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
    render_data: Option<TimePluginRenderData>,
}

impl Plugin for TimePlugin {
    fn event(
        &mut self,
        _event: &winit::event::WindowEvent,
        _control_flow: &mut winit::event_loop::ControlFlow,
        _renderer: &mut Renderer,
        _window: &winit::window::Window,
    ) {
    }

    fn update(
        &mut self,
        _control_flow: &mut winit::event_loop::ControlFlow,
        _renderer: &mut Renderer,
        dt: f64,
    ) {
        self.uniform.time += dt as f32;
        self.uniform.dt = dt as f32;
    }

    fn pre_render(&mut self, queue: &mut wgpu::Queue) {
        let render_data = self.render_data.as_ref().unwrap();

        queue.write_buffer(
            &render_data.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }

    fn initialize(&mut self, renderer: &mut Renderer) {
        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Time Buffer"),
                contents: bytemuck::cast_slice(&[self.uniform]),
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

        self.render_data = Some(TimePluginRenderData {
            buffer,
            bind_group_layout,
            bind_group,
        });
    }

    fn get_bind_group_layouts(&self) -> Option<Vec<&wgpu::BindGroupLayout>> {
        let render_data = self.render_data.as_ref().unwrap();
        Some(vec![&render_data.bind_group_layout])
    }

    fn get_bind_groups(&self) -> Option<Vec<&wgpu::BindGroup>> {
        let render_data = self.render_data.as_ref().unwrap();
        Some(vec![&render_data.bind_group])
    }
}

impl Default for TimePlugin {
    fn default() -> Self {
        Self {
            uniform: TimePluginUniform {
                dt: 0.0,
                time: 0.0,
                #[cfg(target_arch = "wasm32")]
                padding: [0.0; 2],
            },
            render_data: None,
        }
    }
}
