use crate::{plugins::Plugin, renderer::Renderer};
use wgpu::util::DeviceExt;

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
    render_data: Option<ResolutionPluginRenderData>,
}

impl Plugin for ResolutionPlugin {
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
        renderer: &mut Renderer,
        _dt: f64,
    ) {
        self.uniform.resolution[0] = renderer.config.width as f32;
        self.uniform.resolution[1] = renderer.config.height as f32;
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
                label: Some("Resolution Buffer"),
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

        self.render_data = Some(ResolutionPluginRenderData {
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

impl Default for ResolutionPlugin {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")]{
                Self {
                    uniform: ResolutionPluginUniform {
                        resolution: [0.0, 0.0],
                        padding: [0.0; 2]
                    },
                    render_data: None,
                }
            }else{
                Self {
                uniform: ResolutionPluginUniform {
                    resolution: [0.0, 0.0],
                    },
                    render_data: None,
                }
            }
        }
    }
}
