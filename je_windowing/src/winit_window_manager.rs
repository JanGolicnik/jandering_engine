use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
    },
};

use winit::{
    application::ApplicationHandler,
    event::{self, WindowEvent},
    event_loop::ActiveEventLoop,
};

use crate::{
    winit_window::{InnerWinitWindow, WinitWindow},
    Events, WindowConfig, WindowId, WindowManagerTrait,
};

#[cfg(windows)]
use winit::platform::windows::EventLoopBuilderExtWindows;

#[derive(Debug)]
pub struct WinitWindowManager {
    next_id: WindowId,

    should_end: bool,

    windows: HashMap<WindowId, Arc<Mutex<InnerWinitWindow>>>,
    queued_windows: HashMap<WindowId, Arc<Mutex<InnerWinitWindow>>>,
    ids_to_handles: HashMap<winit::window::WindowId, WindowId>,
}

impl WindowManagerTrait for WinitWindowManager {
    fn new() -> Self {
        Self {
            next_id: 0,

            should_end: false,

            windows: HashMap::new(),
            queued_windows: HashMap::new(),
            ids_to_handles: HashMap::new(),
        }
    }

    fn run(self, update_function: impl FnMut(&mut WinitWindowManager)) {
        let event_loop = winit::event_loop::EventLoop::<()>::with_user_event()
            .build()
            .unwrap();

        let mut event_handler = WinitEventHandler {
            window_manager: self,
            is_init: false,
            should_close: false,
            update_function,
        };

        event_loop.run_app(&mut event_handler).unwrap();
    }

    fn spawn_window(&mut self, config: WindowConfig) -> WinitWindow {
        let id = self.next_id;
        self.next_id += 1;

        let inner_window = Arc::new(Mutex::new(InnerWinitWindow::Uninitalized { config }));

        self.queued_windows.insert(id, inner_window.clone());

        WinitWindow { id, inner_window, polled_events: Default::default()}
    }

    fn end(&mut self) {
        self.should_end = true;
    }
}


struct WinitEventHandler<F: FnMut(&mut WinitWindowManager)> {
    is_init: bool, // we get a random resized event that fucks everything up, this ignores it
    should_close: bool,

    update_function: F,
    window_manager: WinitWindowManager,
}

impl<F: FnMut(&mut WinitWindowManager)> WinitEventHandler<F> {
    fn create_queued_windows(&mut self, event_loop: &ActiveEventLoop) {
        for (id, window_arc) in self
            .window_manager
            .queued_windows
            .drain()
        {
            let mut window_lock = window_arc.lock().unwrap();
            let InnerWinitWindow::Uninitalized { config } = & *window_lock else {
                continue;
            };

            let mut window_attributes = winit::window::Window::default_attributes();
            #[cfg(target_arch = "wasm32")]
            {
                use winit::platform::web::WindowAttributesExtWebSys;
                window_attributes = window_attributes.with_prevent_default(true);
            }

            #[cfg(target_arch = "wasm32")]
            let available_size = web_sys::window()
                .and_then(|win| win.screen().ok())
                .and_then(|screen| {
                    Some((
                        screen.avail_width().unwrap_or(1) as u32,
                        screen.avail_height().unwrap_or(1) as u32,
                    ))
                })
                .unwrap_or((1, 1));
            #[cfg(not(target_arch = "wasm32"))]
            let available_size = {
                let size = event_loop.primary_monitor().unwrap().size();
                (size.width, size.height)
            };

            let size = match config.resolution {
                crate::WindowResolution::Exact { width, height } => (width, height),
                crate::WindowResolution::Auto => available_size,
            };

            let position = (
                (available_size.0 - size.0) / 2,
                (available_size.1 - size.1) / 2,
            );

            window_attributes = window_attributes
                .with_title(config.title)
                .with_inner_size(winit::dpi::PhysicalSize::new(size.0, size.1))
                .with_transparent(config.transparent)
                .with_decorations(config.decorations)
                .with_position(winit::dpi::PhysicalPosition::new(position.0, position.1));

            let winit_window = event_loop.create_window(window_attributes).unwrap();
            winit_window.set_cursor_visible(config.show_cursor);
            
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::window()
                    .and_then(|win| win.document())
                    .and_then(|doc| {
                        use winit::platform::web::WindowExtWebSys;
                        let dst = doc.get_element_by_id("jandering-engine-canvas-body")?;
                        let canvas = web_sys::Element::from(winit_window.canvas().unwrap());
                        dst.append_child(&canvas).ok()?;
                        Some(())
                    })
                    .expect("coulnt append canvas to document body");
            }

            self.window_manager.ids_to_handles.insert(winit_window.id(), id);

            *window_lock = InnerWinitWindow::Initialized { window: winit_window, events: Events::with_initialized(), last_redraw_time: web_time::Instant::now(), should_close: false, fps_preference: config.fps_preference};

            self.window_manager.windows.insert(id, window_arc.clone());
        }

        if ! self.window_manager.queued_windows.is_empty(){
            panic!("somehow windows werent properly queued up!!")
        }
    }
}

impl<F: FnMut(&mut WinitWindowManager)> ApplicationHandler<()> for WinitEventHandler<F> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.create_queued_windows(event_loop);
    }

    // fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: EngineEvent) {
    //     let mut event_handler = self.event_handler.take().unwrap();
    //     event_handler.on_custom_event(event, self);
    //     self.event_handler = Some(event_handler);
    // }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: event::DeviceId,
        event: event::DeviceEvent,
    ) {
        if event_loop.exiting() {
            return;
        }

        for (_, window) in self.window_manager.windows.iter_mut() {
            if let event::DeviceEvent::MouseMotion { delta } = event {
                let mut window = window.lock().unwrap();
                let InnerWinitWindow::Initialized {  events, ..}  = &mut *window else {
                    return;
                };
                events.push(crate::WindowEvent::RawMouseMotion((
                    delta.0 as f32,
                    delta.1 as f32,
                )));
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if event_loop.exiting() {
            return;
        }

        if !matches!(
            event_loop.control_flow(),
            winit::event_loop::ControlFlow::Poll
        ) {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        }

        let Some((window, _)) = self
            .window_manager.ids_to_handles
            .get(&window_id)
            .and_then(|handle| self.window_manager.windows.get_mut(handle).map(|win| (win, handle)))
        else {
            return;
        };

        let Some(event) = winit_event_to_window_event(&event) else {
            return;
        };

        if self.is_init {
            if let crate::WindowEvent::Resized(_) = event {
                self.is_init = false;
                return;
            }
        }

        {        
            let mut window = window.lock().unwrap();
            window.handle_event(event);
        }

        if matches!(event, crate::WindowEvent::RedrawRequested ) {
            (self.update_function)(&mut self.window_manager);
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.should_close {
            event_loop.exit();
            return;
        }

        self.create_queued_windows(event_loop);

        self.window_manager.windows.retain(|_, window| {
            let window = window.lock().unwrap();
            match &*window {
                InnerWinitWindow::Initialized { should_close, .. } => !should_close,
                InnerWinitWindow::Uninitalized { .. } => false,
            }
        });

        if self.window_manager.windows.is_empty() {
            event_loop.exit();
        }
    }
}

fn winit_event_to_window_event(event: &WindowEvent) -> Option<crate::WindowEvent> {
    let e = match event {
        WindowEvent::CursorEntered { .. } => crate::WindowEvent::MouseEntered,
        WindowEvent::CursorLeft { .. } => crate::WindowEvent::MouseLeft,
        WindowEvent::Touch(winit::event::Touch {
            location, phase, ..
        }) => match phase {
            event::TouchPhase::Started => crate::WindowEvent::MouseInput {
                button: crate::MouseButton::Left,
                state: crate::InputState::Pressed,
            },
            event::TouchPhase::Moved => {
                crate::WindowEvent::MouseMotion((location.x as f32, location.y as f32))
            }
            event::TouchPhase::Cancelled | event::TouchPhase::Ended => {
                crate::WindowEvent::MouseInput {
                    button: crate::MouseButton::Left,
                    state: crate::InputState::Released,
                }
            }
        },
        WindowEvent::Resized(size) => crate::WindowEvent::Resized((size.width, size.height)),
        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            crate::WindowEvent::ScaleFactorChanged(*scale_factor as f32)
        }
        WindowEvent::CloseRequested => crate::WindowEvent::CloseRequested,
        WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                    state,
                    ..
                },
            ..
        } => crate::WindowEvent::KeyInput {
            key: winit_key_to_window_key(*key_code),
            state: match state {
                winit::event::ElementState::Pressed => crate::InputState::Pressed,
                winit::event::ElementState::Released => crate::InputState::Released,
            },
        },
        WindowEvent::CursorMoved { position, .. } => {
            crate::WindowEvent::MouseMotion((position.x as f32, position.y as f32))
        }
        WindowEvent::MouseWheel {
            delta: winit::event::MouseScrollDelta::LineDelta(x, y),
            ..
        } => crate::WindowEvent::Scroll((*x, *y)),
        WindowEvent::MouseWheel {
            delta: winit::event::MouseScrollDelta::PixelDelta(pos),
            ..
        } => crate::WindowEvent::Scroll((
            if pos.x.is_sign_positive() { 1.0 } else { -1.0 },
            if pos.y.is_sign_positive() { 1.0 } else { -1.0 },
        )),
        WindowEvent::MouseInput { state, button, .. } => crate::WindowEvent::MouseInput {
            button: match button {
                winit::event::MouseButton::Left => crate::MouseButton::Left,
                winit::event::MouseButton::Right => crate::MouseButton::Right,
                winit::event::MouseButton::Middle => crate::MouseButton::Middle,
                _ => crate::MouseButton::Unknown,
            },
            state: match state {
                winit::event::ElementState::Pressed => crate::InputState::Pressed,
                winit::event::ElementState::Released => crate::InputState::Released,
            },
        },
        WindowEvent::RedrawRequested => crate::WindowEvent::RedrawRequested,
        _ => {
            return None;
        }
    };

    Some(e)
}

fn winit_key_to_window_key(key: winit::keyboard::KeyCode) -> crate::Key {
    match key {
        winit::keyboard::KeyCode::Digit0 => crate::Key::Key0,
        winit::keyboard::KeyCode::Digit1 => crate::Key::Key1,
        winit::keyboard::KeyCode::Digit2 => crate::Key::Key2,
        winit::keyboard::KeyCode::Digit3 => crate::Key::Key3,
        winit::keyboard::KeyCode::Digit4 => crate::Key::Key4,
        winit::keyboard::KeyCode::Digit5 => crate::Key::Key5,
        winit::keyboard::KeyCode::Digit6 => crate::Key::Key6,
        winit::keyboard::KeyCode::Digit7 => crate::Key::Key7,
        winit::keyboard::KeyCode::Digit8 => crate::Key::Key8,
        winit::keyboard::KeyCode::Digit9 => crate::Key::Key9,
        winit::keyboard::KeyCode::KeyA => crate::Key::A,
        winit::keyboard::KeyCode::KeyB => crate::Key::B,
        winit::keyboard::KeyCode::KeyC => crate::Key::C,
        winit::keyboard::KeyCode::KeyD => crate::Key::D,
        winit::keyboard::KeyCode::KeyE => crate::Key::E,
        winit::keyboard::KeyCode::KeyF => crate::Key::F,
        winit::keyboard::KeyCode::KeyG => crate::Key::G,
        winit::keyboard::KeyCode::KeyH => crate::Key::H,
        winit::keyboard::KeyCode::KeyI => crate::Key::I,
        winit::keyboard::KeyCode::KeyJ => crate::Key::J,
        winit::keyboard::KeyCode::KeyK => crate::Key::K,
        winit::keyboard::KeyCode::KeyL => crate::Key::L,
        winit::keyboard::KeyCode::KeyM => crate::Key::M,
        winit::keyboard::KeyCode::KeyN => crate::Key::N,
        winit::keyboard::KeyCode::KeyO => crate::Key::O,
        winit::keyboard::KeyCode::KeyP => crate::Key::P,
        winit::keyboard::KeyCode::KeyQ => crate::Key::Q,
        winit::keyboard::KeyCode::KeyR => crate::Key::R,
        winit::keyboard::KeyCode::KeyS => crate::Key::S,
        winit::keyboard::KeyCode::KeyT => crate::Key::T,
        winit::keyboard::KeyCode::KeyU => crate::Key::U,
        winit::keyboard::KeyCode::KeyV => crate::Key::V,
        winit::keyboard::KeyCode::KeyW => crate::Key::W,
        winit::keyboard::KeyCode::KeyX => crate::Key::X,
        winit::keyboard::KeyCode::KeyY => crate::Key::Y,
        winit::keyboard::KeyCode::KeyZ => crate::Key::Z,
        winit::keyboard::KeyCode::Minus => crate::Key::Minus,
        winit::keyboard::KeyCode::AltLeft | winit::keyboard::KeyCode::AltRight => crate::Key::Alt,
        winit::keyboard::KeyCode::Backspace => crate::Key::Backspace,
        winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => {
            crate::Key::Control
        }
        winit::keyboard::KeyCode::Enter => crate::Key::Enter,
        winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => {
            crate::Key::Shift
        }
        winit::keyboard::KeyCode::Space => crate::Key::Space,
        winit::keyboard::KeyCode::Tab => crate::Key::Tab,
        winit::keyboard::KeyCode::Delete => crate::Key::Delete,
        winit::keyboard::KeyCode::ArrowDown => crate::Key::Down,
        winit::keyboard::KeyCode::ArrowLeft => crate::Key::Left,
        winit::keyboard::KeyCode::ArrowRight => crate::Key::Right,
        winit::keyboard::KeyCode::ArrowUp => crate::Key::Up,
        winit::keyboard::KeyCode::Numpad0 => crate::Key::Numpad0,
        winit::keyboard::KeyCode::Numpad1 => crate::Key::Numpad1,
        winit::keyboard::KeyCode::Numpad2 => crate::Key::Numpad2,
        winit::keyboard::KeyCode::Numpad3 => crate::Key::Numpad3,
        winit::keyboard::KeyCode::Numpad4 => crate::Key::Numpad4,
        winit::keyboard::KeyCode::Numpad5 => crate::Key::Numpad5,
        winit::keyboard::KeyCode::Numpad6 => crate::Key::Numpad6,
        winit::keyboard::KeyCode::Numpad7 => crate::Key::Numpad7,
        winit::keyboard::KeyCode::Numpad8 => crate::Key::Numpad8,
        winit::keyboard::KeyCode::Numpad9 => crate::Key::Numpad9,
        winit::keyboard::KeyCode::NumpadAdd => crate::Key::NumpadAdd,
        winit::keyboard::KeyCode::NumpadComma => crate::Key::NumpadComma,
        winit::keyboard::KeyCode::NumpadDecimal => crate::Key::NumpadDecimal,
        winit::keyboard::KeyCode::NumpadDivide => crate::Key::NumpadDivide,
        winit::keyboard::KeyCode::NumpadEnter => crate::Key::NumpadEnter,
        winit::keyboard::KeyCode::NumpadEqual => crate::Key::NumpadEquals,
        winit::keyboard::KeyCode::NumpadMultiply => crate::Key::NumpadMultiply,
        winit::keyboard::KeyCode::NumpadSubtract => crate::Key::NumpadSubtract,
        winit::keyboard::KeyCode::Escape => crate::Key::Escape,
        winit::keyboard::KeyCode::Copy => crate::Key::Copy,
        winit::keyboard::KeyCode::Cut => crate::Key::Cut,
        winit::keyboard::KeyCode::Paste => crate::Key::Paste,
        winit::keyboard::KeyCode::F1 => crate::Key::F1,
        winit::keyboard::KeyCode::F2 => crate::Key::F2,
        winit::keyboard::KeyCode::F3 => crate::Key::F3,
        winit::keyboard::KeyCode::F4 => crate::Key::F4,
        winit::keyboard::KeyCode::F5 => crate::Key::F5,
        winit::keyboard::KeyCode::F6 => crate::Key::F6,
        winit::keyboard::KeyCode::F7 => crate::Key::F7,
        winit::keyboard::KeyCode::F8 => crate::Key::F8,
        winit::keyboard::KeyCode::F9 => crate::Key::F9,
        winit::keyboard::KeyCode::F10 => crate::Key::F10,
        winit::keyboard::KeyCode::F11 => crate::Key::F11,
        winit::keyboard::KeyCode::F12 => crate::Key::F12,
        winit::keyboard::KeyCode::F13 => crate::Key::F13,
        winit::keyboard::KeyCode::F14 => crate::Key::F14,
        winit::keyboard::KeyCode::F15 => crate::Key::F15,
        winit::keyboard::KeyCode::F16 => crate::Key::F16,
        winit::keyboard::KeyCode::F17 => crate::Key::F17,
        winit::keyboard::KeyCode::F18 => crate::Key::F18,
        winit::keyboard::KeyCode::F19 => crate::Key::F19,
        winit::keyboard::KeyCode::F20 => crate::Key::F20,
        winit::keyboard::KeyCode::F21 => crate::Key::F21,
        winit::keyboard::KeyCode::F22 => crate::Key::F22,
        winit::keyboard::KeyCode::F23 => crate::Key::F23,
        winit::keyboard::KeyCode::F24 => crate::Key::F24,
        _ => crate::Key::Unknown,
    }
}
