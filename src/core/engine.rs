use crate::implementation::window::winit::WinitWindow;

use super::{
    event_handler::EventHandler,
    renderer::Renderer,
    window::{Window, WindowBuilder, WindowEvent, WindowEventHandler},
};

pub struct Engine {
    pub events: Vec<WindowEvent>,
    pub window: Option<Box<dyn Window>>,
    pub renderer: Box<Renderer>,
    event_handler: Option<Box<dyn EventHandler>>,
}

impl Engine {
    pub fn new(builder: EngineBuilder) -> Self {
        let window: Box<dyn Window> = Box::new(WinitWindow::new(builder.window_builder));
        let renderer = pollster::block_on(Renderer::new(&window));

        Self {
            events: Vec::new(),
            window: Some(window),
            renderer: Box::new(renderer),
            event_handler: None,
        }
    }

    pub fn run<T: EventHandler + 'static>(mut self, event_handler: T) {
        let mut window = self.window.take().unwrap();
        self.event_handler = Some(Box::new(event_handler));
        window.run(Box::new(self));
    }
}

pub struct EngineContext<'a> {
    pub events: &'a Vec<WindowEvent>,
    pub window: &'a mut dyn Window,
    pub renderer: &'a mut Box<Renderer>,
}

impl WindowEventHandler for Engine {
    fn on_event(&mut self, event: WindowEvent, window: &mut dyn Window) {
        match event {
            WindowEvent::Resized((width, height)) => {
                if window.size() != (width, height) {
                    window.resize(width, height);
                    self.renderer.resize(width, height);
                }
                self.events.push(event);
            }
            WindowEvent::RedrawRequested => {
                let mut context = EngineContext {
                    events: &self.events,
                    window,
                    renderer: &mut self.renderer,
                };
                self.event_handler.as_mut().unwrap().on_update(&mut context);

                self.renderer.begin_frame();

                self.event_handler
                    .as_mut()
                    .unwrap()
                    .on_render(&mut self.renderer);

                self.renderer.present();

                self.events.clear();
            }
            WindowEvent::CloseRequested => window.close(),
            WindowEvent::EventsCleared => {
                window.request_redraw();
            }
            _ => self.events.push(event),
        }
    }
}

#[derive(Default)]
pub struct EngineBuilder {
    window_builder: WindowBuilder,
}

impl EngineBuilder {
    pub fn build(self) -> Engine {
        Engine::new(self)
    }
    pub fn with_window(mut self, window_builder: WindowBuilder) -> Self {
        self.window_builder = window_builder;
        self
    }
}
