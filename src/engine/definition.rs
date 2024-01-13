use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowId},
};

use crate::{
    camera::{Camera, CameraRenderData},
    object::{Instance, Object},
    pipeline::Pipeline,
    renderer::Renderer,
};

use super::Engine;

impl Default for Engine {
    fn default() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        let mut renderer = pollster::block_on(Renderer::new(&window));
        let objects = Vec::new();
        let camera = Camera::new(&renderer.config);
        let camera_render_data = CameraRenderData::new(&renderer.device);
        renderer.add_default_pipeline(&camera_render_data);
        window.set_cursor_visible(false);
        Self {
            window,
            event_loop,
            renderer,
            objects,
            camera: (camera, camera_render_data),
        }
    }
}

impl Engine {
    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }

    pub fn run<F, A>(self, mut event_callback: F, mut update_callback: A)
    where
        F: 'static + FnMut(&WindowEvent<'_>, &mut ControlFlow, &mut Camera),
        A: 'static + FnMut(&mut ControlFlow, &mut Camera, &mut Renderer, &mut Vec<Object>, f64),
    {
        let Self {
            event_loop,
            window,
            mut renderer,
            mut objects,
            mut camera,
        } = self;

        let mut time = std::time::Instant::now();
        let mut last_time = time;

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(*physical_size);
                        camera.0.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        renderer.resize(**new_inner_size);
                    }
                    _ => {}
                }
                window
                    .set_cursor_position(PhysicalPosition::new(
                        renderer.config.width / 2,
                        renderer.config.height / 2,
                    ))
                    .expect("failed to set cursor position");
                camera.0.event(event, &mut renderer, &window);
                event_callback(event, control_flow, &mut camera.0)
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                time = std::time::Instant::now();
                let dt = (time - last_time).as_secs_f64();
                last_time = time;

                camera.0.update(dt);

                update_callback(control_flow, &mut camera.0, &mut renderer, &mut objects, dt);

                camera.1.update_uniform(&camera.0);
                objects.iter_mut().for_each(|e| e.update());
                if renderer.render(&objects, &camera.1).is_err() {
                    *control_flow = ControlFlow::Exit
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        });
    }

    pub fn add_object(&mut self, mut object: Object, instances: Vec<Instance>) {
        object.instances = instances;
        object.update();
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
        let instance_buffer =
            self.renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&object.instance_data),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        object.pipeline = Some(Pipeline {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            _shader: None,
        });
        self.objects.push(object);
    }

    pub fn get_camera(&mut self) -> &mut Camera {
        &mut self.camera.0
    }
}
