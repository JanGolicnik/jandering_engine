use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{WindowBuilder, WindowId},
};

use crate::{renderer::Renderer, types::Vec3};

use super::{Engine, EngineContext, EngineDescriptor};

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")]{
        use web_time::Instant;
    }else{
        use std::time::Instant;
    }
}

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;
impl Engine {
    pub fn new(desc: EngineDescriptor) -> Self {
        let event_loop = EventLoop::new();
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                use winit::platform::web::WindowBuilderExtWebSys;
                let window_builder = WindowBuilder::new().with_prevent_default(true);
            } else {
                let window_builder = WindowBuilder::new();
            }
        }
        let window = window_builder.build(&event_loop).unwrap();
        window.set_inner_size(PhysicalSize::new(desc.resolution.0, desc.resolution.1));
        window.set_cursor_visible(desc.show_cursor);
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
        let renderer = pollster::block_on(Renderer::new(&window));

        Self {
            window,
            event_loop,
            renderer,
            clear_color: desc.clear_color,
        }
    }
    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }

    pub fn run<F>(self, mut update_function: F)
    where
        F: 'static + FnMut(&mut EngineContext, &mut Renderer),
    {
        let Self {
            event_loop,
            window,
            mut renderer,
            clear_color,
            ..
        } = self;

        #[allow(unused_assignments)]
        let mut time = Instant::now();
        let mut last_time = time;

        let mut events = Vec::new();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        renderer.resize(physical_size);
                    }
                    WindowEvent::ScaleFactorChanged {
                        ref new_inner_size, ..
                    } => {
                        renderer.resize(**new_inner_size);
                    }
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    _ => {}
                }
                if let Some(event) = event.to_static() {
                    events.push(event);
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                time = Instant::now();
                let dt = (time - last_time).as_secs_f64();
                last_time = time;

                let (mut encoder, view, surface) = renderer
                    .begin_frame(clear_color)
                    .expect("could not begin frame");

                let mut context: EngineContext<'_> = EngineContext {
                    encoder: &mut encoder,
                    control_flow,
                    surface_view: view,
                    dt,
                    events: &events,
                    window: &window,
                    resolution: (renderer.config.width, renderer.config.height),
                };

                update_function(&mut context, &mut renderer);

                events.clear();

                renderer.submit(encoder, surface);
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        });
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_canvas(&self) -> web_sys::HtmlCanvasElement {
        self.window.canvas()
    }
}

impl Default for EngineDescriptor {
    fn default() -> Self {
        Self {
            resolution: (800, 800),
            clear_color: Vec3::new(0.015, 0.007, 0.045),
            show_cursor: true,
        }
    }
}
