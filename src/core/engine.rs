use std::slice::Iter;

use crate::implementation::{renderer::wgpu::WGPURenderer, window::winit::WinitWindow};

use super::{
    event_handler::EventHandler,
    renderer::Renderer,
    window::{Key, Window, WindowConfig, WindowEvent, WindowEventHandler},
};

#[derive(Default)]
pub struct Events {
    events: Vec<WindowEvent>,
}

impl Events {
    pub fn matches<F>(&self, f: F) -> bool
    where
        F: Fn(&WindowEvent) -> bool,
    {
        self.events.iter().any(f)
    }

    pub fn is_pressed(&self, input_key: Key) -> bool {
        self.events.iter().any(|e| {
            if let WindowEvent::KeyInput {
                key,
                state: super::window::InputState::Pressed,
            } = e
            {
                *key == input_key
            } else {
                false
            }
        })
    }

    pub fn push(&mut self, event: WindowEvent) {
        self.events.push(event)
    }

    pub fn iter(&self) -> Iter<WindowEvent> {
        self.events.iter()
    }

    pub fn clear(&mut self) {
        self.events.clear()
    }
}

pub struct Engine<T: EventHandler + 'static + EngineNew> {
    event_handler: Option<Box<dyn EventHandler>>,
    pub events: Events,
    pub renderer: Option<Box<dyn Renderer>>,
    marker: std::marker::PhantomData<T>,
}

impl<T: EventHandler + 'static + EngineNew> Engine<T> {
    pub fn run(builder: EngineBuilder) {
        // let renderer = WGPURenderer::new(&window).await;
        let engine = Self {
            event_handler: None,
            events: Events::default(),
            renderer: None,
            marker: std::marker::PhantomData,
        };
        WinitWindow::run(builder.window_builder, Box::new(engine));
    }
}

pub struct EngineContext<'a> {
    pub events: &'a Events,
    pub window: &'a mut dyn Window,
    pub renderer: &'a mut Box<dyn Renderer>,
}

pub trait EngineNew {
    fn engine_new<T: EventHandler + 'static + EngineNew>(engine: &mut Engine<T>) -> Self;
}

impl<T: EventHandler + 'static + EngineNew> WindowEventHandler for Engine<T> {
    fn on_event(&mut self, event: WindowEvent, window: &mut dyn Window) {
        if self.renderer.is_none() || self.event_handler.is_none() {
            return;
        }

        let renderer = self.renderer.as_mut().unwrap();
        let event_handler = self.event_handler.as_mut().unwrap();
        match event {
            WindowEvent::Resized((width, height)) => {
                if window.size() != (width, height) {
                    window.resize(width, height);
                    renderer.resize(width, height);
                }
                self.events.push(event);
            }
            WindowEvent::RedrawRequested => {
                let mut context = EngineContext {
                    events: &self.events,
                    window,
                    renderer,
                };
                println!("REDRAW REQUESTED");
                event_handler.as_mut().on_update(&mut context);

                renderer.begin_frame();

                event_handler.as_mut().on_render(renderer);

                renderer.present();

                self.events.clear();
            }
            WindowEvent::CloseRequested => window.close(),
            WindowEvent::EventsCleared => {
                window.request_redraw();
            }
            _ => self.events.push(event),
        }
    }

    fn window_created(&mut self, window: &mut dyn Window) {
        let renderer = pollster::block_on(WGPURenderer::new(window));
        self.renderer = Some(Box::new(renderer));
        self.event_handler = Some(Box::new(T::engine_new(self)));
    }
}

#[derive(Default)]
pub struct EngineBuilder {
    window_builder: WindowConfig,
}

impl EngineBuilder {
    pub async fn run<T: EventHandler + 'static + EngineNew>(self) {
        Engine::<T>::run(self)
    }

    pub fn with_window(mut self, window_builder: WindowConfig) -> Self {
        self.window_builder = window_builder;
        self
    }
}
