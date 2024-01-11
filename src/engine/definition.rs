use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowId},
};

use crate::{
    object::{self, Object},
    pipeline::Pipeline,
    renderer::Renderer,
};

use super::Engine;

impl Default for Engine {
    fn default() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        let renderer = pollster::block_on(Renderer::new(&window));
        let objects = Vec::new();
        Self {
            window,
            event_loop,
            renderer,
            objects,
        }
    }
}

impl Engine {
    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }

    pub fn run<F>(self, mut function: F)
    where
        F: 'static + FnMut(&WindowEvent<'_>, &mut ControlFlow),
    {
        let Self {
            event_loop,
            window,
            mut renderer,
            objects,
        } = self;

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        renderer.resize(**new_inner_size);
                    }
                    _ => {}
                }
                function(event, control_flow)
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                if renderer.render(&objects).is_err() {
                    *control_flow = ControlFlow::Exit
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        });
    }

    pub fn add_object(&mut self, mut object: Object) {
        let vertex_buffer =
            self.renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&object.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let index_buffer =
            self.renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&object.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        object.pipeline = Some(Pipeline {
            vertex_buffer,
            index_buffer,
            _shader: None,
        });
        self.objects.push(object);
    }
}
