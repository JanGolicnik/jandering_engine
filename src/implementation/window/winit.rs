use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{self, KeyEvent, MouseButton, MouseScrollDelta, StartCause, Touch, WindowEvent},
    keyboard::PhysicalKey,
    window::UserAttentionType,
};

use crate::core::window::{Window, WindowBuilder, WindowEventHandler};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

pub struct WinitWindow {
    event_loop: Option<winit::event_loop::EventLoop<()>>,
    window: winit::window::Window,
    should_close: bool,
    ignore_next_resize: bool,
    is_init: bool,
    size: (u32, u32),
}

impl WinitWindow {
    pub fn new(builder: WindowBuilder) -> Self {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                use winit::platform::web::WindowBuilderExtWebSys;
                let window_builder = winit::window::WindowBuilder::new().with_prevent_default(true);
            } else {
                let window_builder = winit::window::WindowBuilder::new();
            }
        }

        let (width, height) = match builder.resolution {
            crate::core::window::WindowResolution::Exact { width, height } => (width, height),
            crate::core::window::WindowResolution::Auto => {
                cfg_if::cfg_if! {
                    if #[cfg(target_arch = "wasm32")]{
                        web_sys::window()
                        .and_then(|win| win.screen().ok())
                        .and_then(|screen| Some((screen.avail_width().unwrap_or(1) as u32, screen.avail_height().unwrap_or(1) as u32))
                        ).unwrap_or((1,1))
                    }else{
                        let size = event_loop.primary_monitor().unwrap().size();
                        (size.width, size.height)
                    }
                }
            }
        };

        let window = window_builder
            .with_title(builder.title)
            .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
            .build(&event_loop)
            .unwrap();
        window.set_cursor_visible(builder.show_cursor);
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("jandering-engine-canvas-body")?;
                    let canvas = web_sys::Element::from(window.canvas().unwrap());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("coulnt append canvas to document body");
        }

        Self {
            event_loop: Some(event_loop),
            window,
            should_close: false,
            is_init: true,
            ignore_next_resize: false,
            size: (width, height),
        }
    }
}

impl Window for WinitWindow {
    fn resize(&mut self, width: u32, height: u32) {
        self.ignore_next_resize = self
            .window
            .request_inner_size(PhysicalSize::new(width, height))
            .is_none();
    }

    fn set_cursor_position(&self, x: u32, y: u32) {
        let _ = self.window.set_cursor_position(PhysicalPosition::new(x, y));
    }

    fn size(&self) -> (u32, u32) {
        self.size
    }

    fn width(&self) -> u32 {
        self.size.0
    }

    fn height(&self) -> u32 {
        self.size.1
    }

    fn request_redraw(&mut self) {
        self.window.request_redraw();
    }

    fn close(&mut self) {
        self.should_close = true;
    }

    fn get_raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.window.raw_window_handle()
    }

    fn get_raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        self.window.raw_display_handle()
    }

    fn run(&mut self, mut event_handler: Box<dyn WindowEventHandler>) {
        let event_loop = self.event_loop.take().unwrap();
        event_loop
            .run(|e, target| {
                if self.should_close {
                    target.exit();
                }
                if target.exiting() {
                    return;
                }

                if !matches!(target.control_flow(), winit::event_loop::ControlFlow::Poll) {
                    target.set_control_flow(winit::event_loop::ControlFlow::Poll);
                }
                match e {
                    winit::event::Event::WindowEvent { window_id, event } => {
                        if window_id != self.window.id() {
                            return;
                        }
                        let event = match event {
                            WindowEvent::CursorEntered { .. } => {
                                crate::core::window::WindowEvent::MouseEntered
                            }
                            WindowEvent::CursorLeft { .. } => {
                                crate::core::window::WindowEvent::MouseLeft
                            }
                            WindowEvent::Touch(Touch {
                                location, phase, ..
                            }) => match phase {
                                event::TouchPhase::Started => {
                                    crate::core::window::WindowEvent::MouseInput {
                                        button: crate::core::window::MouseButton::Left,
                                        state: crate::core::window::InputState::Pressed,
                                    }
                                }
                                event::TouchPhase::Moved => {
                                    crate::core::window::WindowEvent::MouseMotion((
                                        location.x as f32,
                                        location.y as f32,
                                    ))
                                }
                                event::TouchPhase::Cancelled | event::TouchPhase::Ended => {
                                    crate::core::window::WindowEvent::MouseInput {
                                        button: crate::core::window::MouseButton::Left,
                                        state: crate::core::window::InputState::Released,
                                    }
                                }
                            },
                            WindowEvent::Resized(size) => {
                                if self.is_init {
                                    self.ignore_next_resize = false;
                                    return;
                                }
                                crate::core::window::WindowEvent::Resized((size.width, size.height))
                            }
                            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                                crate::core::window::WindowEvent::Resized((
                                    ((self.width() as f64 * scale_factor) as u32),
                                    (self.height() as f64 * scale_factor) as u32,
                                ))
                            }
                            WindowEvent::CloseRequested => {
                                crate::core::window::WindowEvent::CloseRequested
                            }
                            WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        physical_key: PhysicalKey::Code(key_code),
                                        state,
                                        ..
                                    },
                                ..
                            } => crate::core::window::WindowEvent::KeyInput {
                                key: winit_key_to_window_key(key_code),
                                state: match state {
                                    winit::event::ElementState::Pressed => {
                                        crate::core::window::InputState::Pressed
                                    }
                                    winit::event::ElementState::Released => {
                                        crate::core::window::InputState::Released
                                    }
                                },
                            },
                            WindowEvent::CursorMoved { position, .. } => {
                                crate::core::window::WindowEvent::MouseMotion((
                                    position.x as f32,
                                    position.y as f32,
                                ))
                            }
                            WindowEvent::MouseWheel {
                                delta: MouseScrollDelta::LineDelta(x, y),
                                ..
                            } => crate::core::window::WindowEvent::Scroll((x, y)),
                            WindowEvent::MouseWheel {
                                delta: MouseScrollDelta::PixelDelta(pos),
                                ..
                            } => crate::core::window::WindowEvent::Scroll((
                                if pos.x.is_sign_positive() { 1.0 } else { -1.0 },
                                if pos.y.is_sign_positive() { 1.0 } else { -1.0 },
                            )),
                            WindowEvent::MouseInput { state, button, .. } => {
                                crate::core::window::WindowEvent::MouseInput {
                                    button: match button {
                                        MouseButton::Left => crate::core::window::MouseButton::Left,
                                        MouseButton::Right => {
                                            crate::core::window::MouseButton::Right
                                        }
                                        MouseButton::Middle => {
                                            crate::core::window::MouseButton::Middle
                                        }
                                        _ => crate::core::window::MouseButton::Unknown,
                                    },
                                    state: match state {
                                        winit::event::ElementState::Pressed => {
                                            crate::core::window::InputState::Pressed
                                        }
                                        winit::event::ElementState::Released => {
                                            crate::core::window::InputState::Released
                                        }
                                    },
                                }
                            }
                            WindowEvent::RedrawRequested => {
                                crate::core::window::WindowEvent::RedrawRequested
                            }
                            _ => {
                                return;
                            }
                        };
                        event_handler.on_event(event, self);
                    }
                    winit::event::Event::NewEvents(cause) => {
                        self.is_init = cause == StartCause::Init;
                    }
                    winit::event::Event::AboutToWait => {
                        event_handler
                            .on_event(crate::core::window::WindowEvent::EventsCleared, self);
                    }
                    _ => {}
                }
            })
            .unwrap();
    }

    fn set_cursor_visible(&mut self, val: bool) {
        self.window.set_cursor_visible(val);
    }

    fn set_title(&mut self, title: &'static str) {
        self.window.set_title(title);
    }

    fn set_transparency_enabled(&mut self, value: bool) {
        self.window.set_transparent(value);
    }

    fn set_visible(&mut self, value: bool) {
        self.window.set_visible(value);
    }

    fn set_resizable(&mut self, value: bool) {
        self.window.set_resizable(value);
    }

    fn set_minimized(&mut self, value: bool) {
        self.window.set_minimized(value);
    }

    fn set_fullscreen(&mut self) {
        self.window
            .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
    }

    fn set_borderless(&mut self) {
        self.window
            .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
    }

    fn set_windowed(&mut self) {
        self.window.set_fullscreen(None);
    }

    fn set_decorations(&mut self, value: bool) {
        self.window.set_decorations(value);
    }

    fn focus_window(&mut self) {
        self.window.focus_window();
    }

    fn request_user_attention(&mut self) {
        self.window
            .request_user_attention(Some(UserAttentionType::Critical));
    }
}

fn winit_key_to_window_key(key: winit::keyboard::KeyCode) -> crate::core::window::Key {
    match key {
        winit::keyboard::KeyCode::Digit0 => crate::core::window::Key::Key0,
        winit::keyboard::KeyCode::Digit1 => crate::core::window::Key::Key1,
        winit::keyboard::KeyCode::Digit2 => crate::core::window::Key::Key2,
        winit::keyboard::KeyCode::Digit3 => crate::core::window::Key::Key3,
        winit::keyboard::KeyCode::Digit4 => crate::core::window::Key::Key4,
        winit::keyboard::KeyCode::Digit5 => crate::core::window::Key::Key5,
        winit::keyboard::KeyCode::Digit6 => crate::core::window::Key::Key6,
        winit::keyboard::KeyCode::Digit7 => crate::core::window::Key::Key7,
        winit::keyboard::KeyCode::Digit8 => crate::core::window::Key::Key8,
        winit::keyboard::KeyCode::Digit9 => crate::core::window::Key::Key9,
        winit::keyboard::KeyCode::KeyA => crate::core::window::Key::A,
        winit::keyboard::KeyCode::KeyB => crate::core::window::Key::B,
        winit::keyboard::KeyCode::KeyC => crate::core::window::Key::C,
        winit::keyboard::KeyCode::KeyD => crate::core::window::Key::D,
        winit::keyboard::KeyCode::KeyE => crate::core::window::Key::E,
        winit::keyboard::KeyCode::KeyF => crate::core::window::Key::F,
        winit::keyboard::KeyCode::KeyG => crate::core::window::Key::G,
        winit::keyboard::KeyCode::KeyH => crate::core::window::Key::H,
        winit::keyboard::KeyCode::KeyI => crate::core::window::Key::I,
        winit::keyboard::KeyCode::KeyJ => crate::core::window::Key::J,
        winit::keyboard::KeyCode::KeyK => crate::core::window::Key::K,
        winit::keyboard::KeyCode::KeyL => crate::core::window::Key::L,
        winit::keyboard::KeyCode::KeyM => crate::core::window::Key::M,
        winit::keyboard::KeyCode::KeyN => crate::core::window::Key::N,
        winit::keyboard::KeyCode::KeyO => crate::core::window::Key::O,
        winit::keyboard::KeyCode::KeyP => crate::core::window::Key::P,
        winit::keyboard::KeyCode::KeyQ => crate::core::window::Key::Q,
        winit::keyboard::KeyCode::KeyR => crate::core::window::Key::R,
        winit::keyboard::KeyCode::KeyS => crate::core::window::Key::S,
        winit::keyboard::KeyCode::KeyT => crate::core::window::Key::T,
        winit::keyboard::KeyCode::KeyU => crate::core::window::Key::U,
        winit::keyboard::KeyCode::KeyV => crate::core::window::Key::V,
        winit::keyboard::KeyCode::KeyW => crate::core::window::Key::W,
        winit::keyboard::KeyCode::KeyX => crate::core::window::Key::X,
        winit::keyboard::KeyCode::KeyY => crate::core::window::Key::Y,
        winit::keyboard::KeyCode::KeyZ => crate::core::window::Key::Z,
        winit::keyboard::KeyCode::Minus => crate::core::window::Key::Minus,
        winit::keyboard::KeyCode::AltLeft | winit::keyboard::KeyCode::AltRight => {
            crate::core::window::Key::Alt
        }
        winit::keyboard::KeyCode::Backspace => crate::core::window::Key::Backspace,
        winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => {
            crate::core::window::Key::Control
        }
        winit::keyboard::KeyCode::Enter => crate::core::window::Key::Enter,
        winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => {
            crate::core::window::Key::Shift
        }
        winit::keyboard::KeyCode::Space => crate::core::window::Key::Space,
        winit::keyboard::KeyCode::Tab => crate::core::window::Key::Tab,
        winit::keyboard::KeyCode::Delete => crate::core::window::Key::Delete,
        winit::keyboard::KeyCode::ArrowDown => crate::core::window::Key::Down,
        winit::keyboard::KeyCode::ArrowLeft => crate::core::window::Key::Left,
        winit::keyboard::KeyCode::ArrowRight => crate::core::window::Key::Right,
        winit::keyboard::KeyCode::ArrowUp => crate::core::window::Key::Up,
        winit::keyboard::KeyCode::Numpad0 => crate::core::window::Key::Numpad0,
        winit::keyboard::KeyCode::Numpad1 => crate::core::window::Key::Numpad1,
        winit::keyboard::KeyCode::Numpad2 => crate::core::window::Key::Numpad2,
        winit::keyboard::KeyCode::Numpad3 => crate::core::window::Key::Numpad3,
        winit::keyboard::KeyCode::Numpad4 => crate::core::window::Key::Numpad4,
        winit::keyboard::KeyCode::Numpad5 => crate::core::window::Key::Numpad5,
        winit::keyboard::KeyCode::Numpad6 => crate::core::window::Key::Numpad6,
        winit::keyboard::KeyCode::Numpad7 => crate::core::window::Key::Numpad7,
        winit::keyboard::KeyCode::Numpad8 => crate::core::window::Key::Numpad8,
        winit::keyboard::KeyCode::Numpad9 => crate::core::window::Key::Numpad9,
        winit::keyboard::KeyCode::NumpadAdd => crate::core::window::Key::NumpadAdd,
        winit::keyboard::KeyCode::NumpadComma => crate::core::window::Key::NumpadComma,
        winit::keyboard::KeyCode::NumpadDecimal => crate::core::window::Key::NumpadDecimal,
        winit::keyboard::KeyCode::NumpadDivide => crate::core::window::Key::NumpadDivide,
        winit::keyboard::KeyCode::NumpadEnter => crate::core::window::Key::NumpadEnter,
        winit::keyboard::KeyCode::NumpadEqual => crate::core::window::Key::NumpadEquals,
        winit::keyboard::KeyCode::NumpadMultiply => crate::core::window::Key::NumpadMultiply,
        winit::keyboard::KeyCode::NumpadSubtract => crate::core::window::Key::NumpadSubtract,
        winit::keyboard::KeyCode::Escape => crate::core::window::Key::Escape,
        winit::keyboard::KeyCode::Copy => crate::core::window::Key::Copy,
        winit::keyboard::KeyCode::Cut => crate::core::window::Key::Cut,
        winit::keyboard::KeyCode::Paste => crate::core::window::Key::Paste,
        winit::keyboard::KeyCode::F1 => crate::core::window::Key::F1,
        winit::keyboard::KeyCode::F2 => crate::core::window::Key::F2,
        winit::keyboard::KeyCode::F3 => crate::core::window::Key::F3,
        winit::keyboard::KeyCode::F4 => crate::core::window::Key::F4,
        winit::keyboard::KeyCode::F5 => crate::core::window::Key::F5,
        winit::keyboard::KeyCode::F6 => crate::core::window::Key::F6,
        winit::keyboard::KeyCode::F7 => crate::core::window::Key::F7,
        winit::keyboard::KeyCode::F8 => crate::core::window::Key::F8,
        winit::keyboard::KeyCode::F9 => crate::core::window::Key::F9,
        winit::keyboard::KeyCode::F10 => crate::core::window::Key::F10,
        winit::keyboard::KeyCode::F11 => crate::core::window::Key::F11,
        winit::keyboard::KeyCode::F12 => crate::core::window::Key::F12,
        winit::keyboard::KeyCode::F13 => crate::core::window::Key::F13,
        winit::keyboard::KeyCode::F14 => crate::core::window::Key::F14,
        winit::keyboard::KeyCode::F15 => crate::core::window::Key::F15,
        winit::keyboard::KeyCode::F16 => crate::core::window::Key::F16,
        winit::keyboard::KeyCode::F17 => crate::core::window::Key::F17,
        winit::keyboard::KeyCode::F18 => crate::core::window::Key::F18,
        winit::keyboard::KeyCode::F19 => crate::core::window::Key::F19,
        winit::keyboard::KeyCode::F20 => crate::core::window::Key::F20,
        winit::keyboard::KeyCode::F21 => crate::core::window::Key::F21,
        winit::keyboard::KeyCode::F22 => crate::core::window::Key::F22,
        winit::keyboard::KeyCode::F23 => crate::core::window::Key::F23,
        winit::keyboard::KeyCode::F24 => crate::core::window::Key::F24,
        _ => crate::core::window::Key::Unknown,
    }
}
