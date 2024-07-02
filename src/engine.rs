use std::slice::Iter;

use crate::{
    renderer::Janderer,
    window::{MouseButton, WindowManager, WindowManagerTrait, WindowTrait},
};

use super::{
    event_handler::EventHandler,
    renderer::Renderer,
    window::{Key, WindowEvent, WindowEventHandler},
};

pub struct Engine<T: EventHandler> {
    event_handler: Option<T>,
    pub events: Events,
    pub renderer: Renderer,

    window_manager: Option<WindowManager>,

    last_frame_time: std::time::Instant,
}

impl<T: EventHandler + 'static> Engine<T> {
    pub async fn new() -> Self {
        let renderer = Renderer::new().await;
        let window_manager = WindowManager::new();

        Self {
            event_handler: None,
            events: Events::default(),
            renderer,
            window_manager: Some(window_manager),
            last_frame_time: std::time::Instant::now(),
        }
    }

    pub fn window_manager(&mut self) -> &mut WindowManager {
        self.window_manager.as_mut().unwrap()
    }

    pub async fn run(mut self, event_handler: T) {
        self.event_handler = Some(event_handler);
        self.window_manager.take().unwrap().run(self);
    }
}

pub struct EngineContext<'a> {
    pub events: &'a Events,
    pub window_handle: crate::window::WindowHandle,
    pub window_manager: &'a mut WindowManager,
    pub renderer: &'a mut Renderer,
}
use web_time::Duration;
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;

impl<T: EventHandler> WindowEventHandler<EngineEvent> for Engine<T> {
    fn on_event(
        &mut self,
        event: WindowEvent,
        window_handle: crate::window::WindowHandle,
        window_manager: &mut WindowManager,
    ) {
        match event {
            WindowEvent::Resized((width, height)) => {
                let window = window_manager.get_window(window_handle).unwrap();

                if window.size() != (width, height) {
                    window.resize(width, height);
                    self.renderer.resize(window_handle, width, height);
                }
                self.events.push(event);
            }
            WindowEvent::RedrawRequested => {
                if let crate::window::FpsPreference::Exact(fps) = window_manager
                    .get_window(window_handle)
                    .unwrap()
                    .get_fps_prefrence()
                {
                    #[cfg(target_arch = "wasm32")]
                    panic!();

                    let now = std::time::Instant::now();
                    let dt = now - self.last_frame_time;
                    let min_dt = Duration::from_millis(1000 / fps as u64);
                    if dt < min_dt {
                        std::thread::sleep(min_dt - dt);
                    }
                    self.last_frame_time = std::time::Instant::now();
                }

                {
                    let mut context = EngineContext {
                        events: &self.events,
                        window_handle,
                        window_manager,
                        renderer: &mut self.renderer,
                    };
                    self.event_handler.as_mut().unwrap().on_update(&mut context);
                }

                self.event_handler
                    .as_mut()
                    .unwrap()
                    .on_render(&mut self.renderer);

                self.renderer.present();

                self.events.clear();

                window_manager
                    .get_window(window_handle)
                    .unwrap()
                    .request_redraw();
            }
            WindowEvent::CloseRequested => {
                window_manager.get_window(window_handle).unwrap().close()
            }
            _ => self.events.push(event),
        }
    }

    fn on_custom_event(&mut self, _: EngineEvent, _: &mut WindowManager) {
        todo!()
    }

    fn init(
        &mut self,
        window_handle: crate::window::WindowHandle,
        window_manager: &mut WindowManager,
    ) {
        let mut context = EngineContext {
            events: &self.events,
            window_handle,
            window_manager,
            renderer: &mut self.renderer,
        };
        self.event_handler.as_mut().unwrap().init(&mut context);
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
    pub fn is_mouse_pressed(&self, input_button: MouseButton) -> bool {
        self.events.iter().any(|e| {
            if let WindowEvent::MouseInput {
                button,
                state: super::window::InputState::Pressed,
            } = e
            {
                *button == input_button
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

pub enum EngineEvent {}

pub trait EventHandlerBuilder<E: EventHandler> {
    fn build(self, renderer: &mut Renderer) -> E;
}
