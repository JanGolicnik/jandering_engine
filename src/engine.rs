use crate::{
    renderer::Janderer,
    window::{WindowManager, WindowManagerTrait, WindowTrait},
};

use super::{
    event_handler::EventHandler,
    renderer::Renderer,
    window::{WindowEvent, WindowEventHandler},
};

pub struct Engine<T: EventHandler> {
    event_handler: Option<T>,
    pub renderer: Renderer,

    window_manager: Option<WindowManager>,
}

impl<T: EventHandler + 'static> Engine<T> {
    pub async fn new() -> Self {
        let renderer = Renderer::new().await;
        let window_manager = WindowManager::new();

        Self {
            event_handler: None,
            renderer,
            window_manager: Some(window_manager),
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
    pub window_handle: crate::window::WindowHandle,
    pub window_manager: &'a mut WindowManager,
    pub renderer: &'a mut Renderer,
}
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
                self.renderer.resize(window_handle, width, height);
            }
            WindowEvent::RedrawRequested => {
                {
                    let mut context = EngineContext {
                        window_handle,
                        window_manager,
                        renderer: &mut self.renderer,
                    };
                    self.event_handler.as_mut().unwrap().on_update(&mut context);
                }

                self.event_handler.as_mut().unwrap().on_render(
                    &mut self.renderer,
                    window_handle,
                    window_manager,
                );

                self.renderer.present();

                let window = window_manager.get_window(window_handle).unwrap();

                window.request_redraw();
                window.events.clear();
            }
            _ => {}
        }
    }

    fn on_custom_event(&mut self, _: EngineEvent, _: &mut WindowManager) {
        todo!()
    }

    fn init(&mut self, window_manager: &mut WindowManager) {
        self.event_handler
            .as_mut()
            .unwrap()
            .init(&mut self.renderer, window_manager);
    }
}
pub enum EngineEvent {}

pub trait EventHandlerBuilder<E: EventHandler> {
    fn build(self, renderer: &mut Renderer) -> E;
}
