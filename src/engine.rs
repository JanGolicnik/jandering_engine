use std::{
    slice::Iter,
    sync::{Arc, Mutex},
};

use crate::{renderer::Janderer, window::WindowTrait};

use super::{
    event_handler::EventHandler,
    renderer::Renderer,
    window::{Key, Window, WindowConfig, WindowEvent, WindowEventHandler},
};

pub struct Engine<E: EventHandler, T: EventHandlerBuilder<E>> {
    event_handler: EngineApplication<E, T>,
    pub events: Events,
    renderer: Option<Renderer>,
    marker: std::marker::PhantomData<T>,
}

impl<E: EventHandler + 'static, T: EventHandlerBuilder<E> + 'static> Engine<E, T> {
    pub fn run(builder: EngineBuilder, event_handle_builder: T) {
        let engine = Self {
            event_handler: EngineApplication::Builder(event_handle_builder),
            events: Events::default(),
            renderer: None,
            marker: std::marker::PhantomData,
        };
        Window::run(builder.window_builder, engine);
    }

    pub fn renderer(&self) -> &Renderer {
        self.renderer.as_ref().unwrap()
    }
    pub fn renderer_mut(&mut self) -> &mut Renderer {
        self.renderer.as_mut().unwrap()
    }
}

pub struct EngineContext<'a> {
    pub events: &'a Events,
    pub window: &'a mut Window,
    pub renderer: &'a mut Renderer,
}
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;

impl<E: EventHandler, T: EventHandlerBuilder<E>> WindowEventHandler<EngineEvent> for Engine<E, T> {
    fn on_event(&mut self, event: WindowEvent, window: &mut Window) {
        if self.renderer.is_none() {
            return;
        }

        let event_handler = match &mut self.event_handler {
            EngineApplication::Builder(_) => return,
            EngineApplication::App(event_handler) => event_handler,
        };

        let renderer = self.renderer.as_mut().unwrap();
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
                event_handler.on_update(&mut context);

                renderer.begin_frame();

                event_handler.on_render(renderer);

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

    #[cfg(not(target_arch = "wasm32"))]
    fn window_created<'a>(&'a mut self, window: &'a mut Window) {
        let mut renderer = pollster::block_on(Renderer::new(window));
        take_mut::take(
            &mut self.event_handler,
            |event_handler| match event_handler {
                EngineApplication::Builder(builder) => {
                    EngineApplication::App(builder.build(&mut renderer))
                }
                v => v,
            },
        );
        self.renderer = Some(renderer);
    }

    #[cfg(target_arch = "wasm32")]
    fn window_created<'a>(
        &'a mut self,
        window_ref: Arc<std::sync::Mutex<Window>>,
        event_loop_proxy: EventLoopProxy<EngineEvent>,
    ) {
        wasm_bindgen_futures::spawn_local(async move {
            let mutex = window_ref.as_ref();
            let mut window = mutex.lock().unwrap();
            let renderer = Arc::new(Mutex::new(Some(Renderer::new(&mut window).await)));
            let _ = event_loop_proxy.send_event(EngineEvent::RendererCreated(renderer));
        });
    }

    fn on_custom_event(&mut self, event: EngineEvent, _window: &mut Window) {
        match event {
            EngineEvent::RendererCreated(renderer) => {
                let mut guard = renderer.lock().unwrap();
                let mut renderer = guard.take().unwrap();
                take_mut::take(
                    &mut self.event_handler,
                    |event_handler| match event_handler {
                        EngineApplication::Builder(builder) => {
                            EngineApplication::App(builder.build(&mut renderer))
                        }
                        v => v,
                    },
                );
                self.renderer = Some(renderer);
            }
        }
    }
}

#[derive(Default)]
pub struct EngineBuilder {
    window_builder: WindowConfig,
}

impl EngineBuilder {
    pub fn run<E: EventHandler + 'static, T: EventHandlerBuilder<E> + 'static>(self, builder: T) {
        Engine::run(self, builder)
    }

    pub fn with_window(mut self, window_builder: WindowConfig) -> Self {
        self.window_builder = window_builder;
        self
    }
}

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

pub enum EngineEvent {
    RendererCreated(Arc<Mutex<Option<Renderer>>>),
}

enum EngineApplication<E: EventHandler, T: EventHandlerBuilder<E>> {
    Builder(T),
    App(E),
}

pub trait EventHandlerBuilder<E: EventHandler> {
    fn build(self, renderer: &mut Renderer) -> E;
}
