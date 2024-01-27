use wgpu::CommandEncoder;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{WindowBuilder, WindowId},
};

use crate::{plugins::Plugin, renderer::Renderer};

use super::{default_plugins::default_plugins, Engine};

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")]{
        use web_time::Instant;
    }else{
        use std::time::Instant;
    }
}

impl Default for Engine {
    fn default() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("jandering-engine-canvas-body")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("coulnt append canvas to document body");
        }

        let mut renderer = pollster::block_on(Renderer::new(&window));

        let mut plugins = default_plugins();
        let bind_group_layouts: Vec<&wgpu::BindGroupLayout> = plugins
            .iter_mut()
            .map(|e| {
                e.initialize(&mut renderer);
                e.get_bind_group_layouts()
            })
            .filter(|e| e.is_some())
            .flat_map(|e| e.unwrap())
            .collect();

        let pipeline = renderer.create_pipeline(bind_group_layouts);
        window.set_cursor_visible(false);

        let shaders = vec![pipeline];

        Self {
            window,
            event_loop,
            renderer,
            plugins,
            shaders,
        }
    }
}

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;
impl Engine {
    pub fn new(mut plugins: Vec<Box<dyn Plugin>>) -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        let mut renderer = pollster::block_on(Renderer::new(&window));

        let bind_group_layouts: Vec<&wgpu::BindGroupLayout> = plugins
            .iter_mut()
            .map(|e| {
                e.initialize(&mut renderer);
                e.get_bind_group_layouts()
            })
            .filter(|e| e.is_some())
            .flat_map(|e| e.unwrap())
            .collect();

        let pipeline = renderer.create_pipeline(bind_group_layouts);
        let shaders = vec![pipeline];
        Self {
            window,
            event_loop,
            renderer,
            plugins,
            shaders,
        }
    }
    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }

    pub fn run<F>(self, mut update_function: F)
    where
        F: 'static
            + FnMut(
                &mut Renderer,
                &mut CommandEncoder,
                &mut [Box<dyn Plugin>],
                &mut wgpu::SurfaceTexture,
                &mut [wgpu::RenderPipeline],
                f64,
            ),
    {
        let Self {
            event_loop,
            window,
            mut renderer,
            mut plugins,
            mut shaders,
            ..
        } = self;

        #[allow(unused_assignments)]
        let mut time = Instant::now();
        let mut last_time = time;

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
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    _ => {}
                }

                plugins
                    .iter_mut()
                    .for_each(|e| e.event(event, control_flow, &mut renderer, &window));
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                time = Instant::now();
                let dt = (time - last_time).as_secs_f64();
                last_time = time;

                plugins
                    .iter_mut()
                    .for_each(|e| e.update(control_flow, &mut renderer, dt));

                let (mut encoder, mut surface) =
                    renderer.begin_frame().expect("could not begin frame");

                update_function(
                    &mut renderer,
                    &mut encoder,
                    &mut plugins,
                    &mut surface,
                    &mut shaders,
                    dt,
                );

                renderer.submit(encoder, surface);
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        });
    }

    pub fn add_shader(&mut self, shader: wgpu::RenderPipeline) -> usize {
        self.shaders.push(shader);
        self.shaders.len() - 1
    }

    pub fn get_bind_group_layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        self.plugins
            .iter()
            .map(|e| e.get_bind_group_layouts())
            .filter(|e| e.is_some())
            .flat_map(|e| e.unwrap())
            .collect()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_canvas(&self) -> web_sys::HtmlCanvasElement {
        self.window.canvas()
    }
}
