use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{self, KeyEvent, MouseButton, MouseScrollDelta, Touch, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoopProxy},
    keyboard::PhysicalKey,
    window::UserAttentionType,
};

use crate::{
    engine::EngineEvent,
    window::{
        Window, WindowConfig, WindowEventHandler, WindowHandle, WindowManagerTrait, WindowTrait,
    },
};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

pub struct WinitWindowManager {
    windows: HashMap<crate::window::WindowHandle, WinitWindow>,
    ids_to_handles: HashMap<winit::window::WindowId, crate::window::WindowHandle>,

    window_create_queue: Vec<(WindowHandle, WindowConfig)>,

    should_close: bool,
    event_handler: Option<Box<dyn WindowEventHandler<EngineEvent>>>,
    active_event_loop: Option<&'static ActiveEventLoop>,
    event_loop_proxy: Option<EventLoopProxy<EngineEvent>>,

    next_handle: WindowHandle,

    is_init: bool, // we get a random resized event that fucks everything up, this ignores it
}

impl WindowManagerTrait for WinitWindowManager {
    fn get_window(&mut self, handle: crate::window::WindowHandle) -> Option<&mut Window> {
        self.windows.get_mut(&handle)
    }

    fn new() -> Self {
        Self {
            windows: HashMap::new(),
            ids_to_handles: HashMap::new(),

            window_create_queue: Vec::new(),

            should_close: false,
            event_handler: None,
            active_event_loop: None,
            event_loop_proxy: None,

            next_handle: WindowHandle(0),

            is_init: true,
        }
    }

    fn run<T: WindowEventHandler<EngineEvent> + 'static>(mut self, event_handler: T) {
        self.event_handler = Some(Box::new(event_handler));

        let event_loop = winit::event_loop::EventLoop::<EngineEvent>::with_user_event()
            .build()
            .unwrap();

        self.event_loop_proxy = Some(event_loop.create_proxy());

        event_loop.run_app(&mut self).unwrap();
    }

    fn create_window(&mut self, config: WindowConfig) -> WindowHandle {
        let handle = self.next_handle;
        self.window_create_queue.push((handle, config));
        self.next_handle.0 += 1;
        handle
    }
}

impl WinitWindowManager {
    fn create_queued_windows(&mut self, event_loop: &ActiveEventLoop) {
        let queue = self.window_create_queue.drain(..);
        for (handle, config) in queue {
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
                crate::window::WindowResolution::Exact { width, height } => (width, height),
                crate::window::WindowResolution::Auto => available_size,
            };

            let position = (
                (available_size.0 - size.0) / 2,
                (available_size.1 - size.1) / 2,
            );

            window_attributes = window_attributes
                .with_title(config.title)
                .with_inner_size(winit::dpi::PhysicalSize::new(size.0, size.1))
                .with_position(winit::dpi::PhysicalPosition::new(position.0, position.1));

            let window = event_loop.create_window(window_attributes).unwrap();
            window.set_cursor_visible(config.show_cursor);
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

            let window = WinitWindow {
                window,
                config,
                size,
            };

            self.ids_to_handles.insert(window.window.id(), handle);
            self.windows.insert(handle, window);
        }
    }
}

#[allow(dead_code)]
pub struct WinitWindow {
    window: winit::window::Window,
    config: WindowConfig,
    size: (u32, u32),
}

impl WindowTrait for WinitWindow {
    fn resize(&mut self, width: u32, height: u32) {
        println!("REISZED");
        let _ = self
            .window
            .request_inner_size(PhysicalSize::new(width as f32, height as f32));
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
        todo!();
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

    fn set_position(&mut self, x: i32, y: i32) {
        self.window.set_outer_position(PhysicalPosition::new(x, y));
    }

    fn set_as_desktop(&mut self) {
        match self.window.raw_window_handle() {
            raw_window_handle::RawWindowHandle::Win32(handle) => unsafe {
                let mut hwnd = windows::Win32::Foundation::HWND(0);

                let progman_hwnd = windows::Win32::UI::WindowsAndMessaging::FindWindowA(
                    windows::core::s!("ProgMan"),
                    None,
                );

                windows::Win32::UI::WindowsAndMessaging::SendMessageTimeoutA(
                    progman_hwnd,
                    0x052C,
                    None,
                    None,
                    windows::Win32::UI::WindowsAndMessaging::SMTO_NORMAL,
                    1000,
                    None,
                );

                windows::Win32::UI::WindowsAndMessaging::EnumWindows(
                    Some(enum_windows_proc),
                    windows::Win32::Foundation::LPARAM(
                        &mut hwnd as *mut windows::Win32::Foundation::HWND as isize,
                    ),
                )
                .unwrap();
                windows::Win32::UI::WindowsAndMessaging::SetParent(
                    windows::Win32::Foundation::HWND(handle.hwnd as isize),
                    hwnd,
                );

                self.window
                    .set_outer_position(PhysicalPosition::new(-10, -40));
                let mut size = (0, 0);
                for monitor in self.window.available_monitors() {
                    size.0 += monitor.size().width;
                    size.1 = size.1.max(monitor.size().height);
                }
                let _ = self
                    .window
                    .request_inner_size(PhysicalSize::new(size.0, size.1));
            },
            _ => todo!(),
        }
    }

    fn get_raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.window.raw_window_handle()
    }

    fn get_raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        self.window.raw_display_handle()
    }

    fn set_fps_prefrence(&mut self, preference: crate::window::FpsPreference) {
        self.config.fps_preference = preference;
    }

    fn get_fps_prefrence(&self) -> crate::window::FpsPreference {
        self.config.fps_preference
    }
}

impl ApplicationHandler<EngineEvent> for WinitWindowManager {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.create_queued_windows(event_loop);
        let mut event_handler = self.event_handler.take().unwrap();
        self.active_event_loop = Some(unsafe {
            std::mem::transmute::<&ActiveEventLoop, &'static ActiveEventLoop>(event_loop)
        });

        let handles = self.windows.keys().cloned().collect::<Vec<_>>();
        for handle in handles {
            event_handler.init(handle, self);
        }

        self.event_handler = Some(event_handler);
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: EngineEvent) {
        let mut event_handler = self.event_handler.take().unwrap();
        event_handler.on_custom_event(event, self);
        self.event_handler = Some(event_handler);
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

        let Some(handle) = self.ids_to_handles.get(&window_id) else {
            return;
        };

        let Some(event) = winit_event_to_window_event(&event) else {
            return;
        };

        if self.is_init {
            if let crate::window::WindowEvent::Resized(_) = event {
                self.is_init = false;
                return;
            }
        }

        let mut event_handler = self.event_handler.take().unwrap();

        self.active_event_loop = Some(unsafe {
            std::mem::transmute::<&ActiveEventLoop, &'static ActiveEventLoop>(event_loop)
        });

        event_handler.on_event(event, *handle, self);

        self.event_handler = Some(event_handler);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.should_close {
            event_loop.exit();
            return;
        }

        self.create_queued_windows(event_loop);
    }
}

use std::collections::HashMap;

unsafe extern "system" fn enum_windows_proc(
    hwnd: windows::Win32::Foundation::HWND,
    l_param: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    let out_hwnd = l_param.0 as *mut windows::Win32::Foundation::HWND;

    let p: windows::Win32::Foundation::HWND =
        windows::Win32::UI::WindowsAndMessaging::FindWindowExA(
            hwnd,
            None,
            windows::core::s!("SHELLDLL_DefView"),
            None,
        );

    if p.0 != 0 {
        *out_hwnd = windows::Win32::UI::WindowsAndMessaging::FindWindowExA(
            None,
            hwnd,
            windows::core::s!("WorkerW"),
            None,
        );
    }
    windows::Win32::Foundation::TRUE
}
fn winit_event_to_window_event(event: &WindowEvent) -> Option<crate::window::WindowEvent> {
    let e = match event {
        WindowEvent::CursorEntered { .. } => crate::window::WindowEvent::MouseEntered,
        WindowEvent::CursorLeft { .. } => crate::window::WindowEvent::MouseLeft,
        WindowEvent::Touch(Touch {
            location, phase, ..
        }) => match phase {
            event::TouchPhase::Started => crate::window::WindowEvent::MouseInput {
                button: crate::window::MouseButton::Left,
                state: crate::window::InputState::Pressed,
            },
            event::TouchPhase::Moved => {
                crate::window::WindowEvent::MouseMotion((location.x as f32, location.y as f32))
            }
            event::TouchPhase::Cancelled | event::TouchPhase::Ended => {
                crate::window::WindowEvent::MouseInput {
                    button: crate::window::MouseButton::Left,
                    state: crate::window::InputState::Released,
                }
            }
        },
        WindowEvent::Resized(size) => {
            crate::window::WindowEvent::Resized((size.width, size.height))
        }
        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            crate::window::WindowEvent::ScaleFactorChanged(*scale_factor as f32)
        }
        WindowEvent::CloseRequested => crate::window::WindowEvent::CloseRequested,
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(key_code),
                    state,
                    ..
                },
            ..
        } => crate::window::WindowEvent::KeyInput {
            key: winit_key_to_window_key(*key_code),
            state: match state {
                winit::event::ElementState::Pressed => crate::window::InputState::Pressed,
                winit::event::ElementState::Released => crate::window::InputState::Released,
            },
        },
        WindowEvent::CursorMoved { position, .. } => {
            crate::window::WindowEvent::MouseMotion((position.x as f32, position.y as f32))
        }
        WindowEvent::MouseWheel {
            delta: MouseScrollDelta::LineDelta(x, y),
            ..
        } => crate::window::WindowEvent::Scroll((*x, *y)),
        WindowEvent::MouseWheel {
            delta: MouseScrollDelta::PixelDelta(pos),
            ..
        } => crate::window::WindowEvent::Scroll((
            if pos.x.is_sign_positive() { 1.0 } else { -1.0 },
            if pos.y.is_sign_positive() { 1.0 } else { -1.0 },
        )),
        WindowEvent::MouseInput { state, button, .. } => crate::window::WindowEvent::MouseInput {
            button: match button {
                MouseButton::Left => crate::window::MouseButton::Left,
                MouseButton::Right => crate::window::MouseButton::Right,
                MouseButton::Middle => crate::window::MouseButton::Middle,
                _ => crate::window::MouseButton::Unknown,
            },
            state: match state {
                winit::event::ElementState::Pressed => crate::window::InputState::Pressed,
                winit::event::ElementState::Released => crate::window::InputState::Released,
            },
        },
        WindowEvent::RedrawRequested => crate::window::WindowEvent::RedrawRequested,
        _ => {
            return None;
        }
    };

    Some(e)
}

fn winit_key_to_window_key(key: winit::keyboard::KeyCode) -> crate::window::Key {
    match key {
        winit::keyboard::KeyCode::Digit0 => crate::window::Key::Key0,
        winit::keyboard::KeyCode::Digit1 => crate::window::Key::Key1,
        winit::keyboard::KeyCode::Digit2 => crate::window::Key::Key2,
        winit::keyboard::KeyCode::Digit3 => crate::window::Key::Key3,
        winit::keyboard::KeyCode::Digit4 => crate::window::Key::Key4,
        winit::keyboard::KeyCode::Digit5 => crate::window::Key::Key5,
        winit::keyboard::KeyCode::Digit6 => crate::window::Key::Key6,
        winit::keyboard::KeyCode::Digit7 => crate::window::Key::Key7,
        winit::keyboard::KeyCode::Digit8 => crate::window::Key::Key8,
        winit::keyboard::KeyCode::Digit9 => crate::window::Key::Key9,
        winit::keyboard::KeyCode::KeyA => crate::window::Key::A,
        winit::keyboard::KeyCode::KeyB => crate::window::Key::B,
        winit::keyboard::KeyCode::KeyC => crate::window::Key::C,
        winit::keyboard::KeyCode::KeyD => crate::window::Key::D,
        winit::keyboard::KeyCode::KeyE => crate::window::Key::E,
        winit::keyboard::KeyCode::KeyF => crate::window::Key::F,
        winit::keyboard::KeyCode::KeyG => crate::window::Key::G,
        winit::keyboard::KeyCode::KeyH => crate::window::Key::H,
        winit::keyboard::KeyCode::KeyI => crate::window::Key::I,
        winit::keyboard::KeyCode::KeyJ => crate::window::Key::J,
        winit::keyboard::KeyCode::KeyK => crate::window::Key::K,
        winit::keyboard::KeyCode::KeyL => crate::window::Key::L,
        winit::keyboard::KeyCode::KeyM => crate::window::Key::M,
        winit::keyboard::KeyCode::KeyN => crate::window::Key::N,
        winit::keyboard::KeyCode::KeyO => crate::window::Key::O,
        winit::keyboard::KeyCode::KeyP => crate::window::Key::P,
        winit::keyboard::KeyCode::KeyQ => crate::window::Key::Q,
        winit::keyboard::KeyCode::KeyR => crate::window::Key::R,
        winit::keyboard::KeyCode::KeyS => crate::window::Key::S,
        winit::keyboard::KeyCode::KeyT => crate::window::Key::T,
        winit::keyboard::KeyCode::KeyU => crate::window::Key::U,
        winit::keyboard::KeyCode::KeyV => crate::window::Key::V,
        winit::keyboard::KeyCode::KeyW => crate::window::Key::W,
        winit::keyboard::KeyCode::KeyX => crate::window::Key::X,
        winit::keyboard::KeyCode::KeyY => crate::window::Key::Y,
        winit::keyboard::KeyCode::KeyZ => crate::window::Key::Z,
        winit::keyboard::KeyCode::Minus => crate::window::Key::Minus,
        winit::keyboard::KeyCode::AltLeft | winit::keyboard::KeyCode::AltRight => {
            crate::window::Key::Alt
        }
        winit::keyboard::KeyCode::Backspace => crate::window::Key::Backspace,
        winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => {
            crate::window::Key::Control
        }
        winit::keyboard::KeyCode::Enter => crate::window::Key::Enter,
        winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => {
            crate::window::Key::Shift
        }
        winit::keyboard::KeyCode::Space => crate::window::Key::Space,
        winit::keyboard::KeyCode::Tab => crate::window::Key::Tab,
        winit::keyboard::KeyCode::Delete => crate::window::Key::Delete,
        winit::keyboard::KeyCode::ArrowDown => crate::window::Key::Down,
        winit::keyboard::KeyCode::ArrowLeft => crate::window::Key::Left,
        winit::keyboard::KeyCode::ArrowRight => crate::window::Key::Right,
        winit::keyboard::KeyCode::ArrowUp => crate::window::Key::Up,
        winit::keyboard::KeyCode::Numpad0 => crate::window::Key::Numpad0,
        winit::keyboard::KeyCode::Numpad1 => crate::window::Key::Numpad1,
        winit::keyboard::KeyCode::Numpad2 => crate::window::Key::Numpad2,
        winit::keyboard::KeyCode::Numpad3 => crate::window::Key::Numpad3,
        winit::keyboard::KeyCode::Numpad4 => crate::window::Key::Numpad4,
        winit::keyboard::KeyCode::Numpad5 => crate::window::Key::Numpad5,
        winit::keyboard::KeyCode::Numpad6 => crate::window::Key::Numpad6,
        winit::keyboard::KeyCode::Numpad7 => crate::window::Key::Numpad7,
        winit::keyboard::KeyCode::Numpad8 => crate::window::Key::Numpad8,
        winit::keyboard::KeyCode::Numpad9 => crate::window::Key::Numpad9,
        winit::keyboard::KeyCode::NumpadAdd => crate::window::Key::NumpadAdd,
        winit::keyboard::KeyCode::NumpadComma => crate::window::Key::NumpadComma,
        winit::keyboard::KeyCode::NumpadDecimal => crate::window::Key::NumpadDecimal,
        winit::keyboard::KeyCode::NumpadDivide => crate::window::Key::NumpadDivide,
        winit::keyboard::KeyCode::NumpadEnter => crate::window::Key::NumpadEnter,
        winit::keyboard::KeyCode::NumpadEqual => crate::window::Key::NumpadEquals,
        winit::keyboard::KeyCode::NumpadMultiply => crate::window::Key::NumpadMultiply,
        winit::keyboard::KeyCode::NumpadSubtract => crate::window::Key::NumpadSubtract,
        winit::keyboard::KeyCode::Escape => crate::window::Key::Escape,
        winit::keyboard::KeyCode::Copy => crate::window::Key::Copy,
        winit::keyboard::KeyCode::Cut => crate::window::Key::Cut,
        winit::keyboard::KeyCode::Paste => crate::window::Key::Paste,
        winit::keyboard::KeyCode::F1 => crate::window::Key::F1,
        winit::keyboard::KeyCode::F2 => crate::window::Key::F2,
        winit::keyboard::KeyCode::F3 => crate::window::Key::F3,
        winit::keyboard::KeyCode::F4 => crate::window::Key::F4,
        winit::keyboard::KeyCode::F5 => crate::window::Key::F5,
        winit::keyboard::KeyCode::F6 => crate::window::Key::F6,
        winit::keyboard::KeyCode::F7 => crate::window::Key::F7,
        winit::keyboard::KeyCode::F8 => crate::window::Key::F8,
        winit::keyboard::KeyCode::F9 => crate::window::Key::F9,
        winit::keyboard::KeyCode::F10 => crate::window::Key::F10,
        winit::keyboard::KeyCode::F11 => crate::window::Key::F11,
        winit::keyboard::KeyCode::F12 => crate::window::Key::F12,
        winit::keyboard::KeyCode::F13 => crate::window::Key::F13,
        winit::keyboard::KeyCode::F14 => crate::window::Key::F14,
        winit::keyboard::KeyCode::F15 => crate::window::Key::F15,
        winit::keyboard::KeyCode::F16 => crate::window::Key::F16,
        winit::keyboard::KeyCode::F17 => crate::window::Key::F17,
        winit::keyboard::KeyCode::F18 => crate::window::Key::F18,
        winit::keyboard::KeyCode::F19 => crate::window::Key::F19,
        winit::keyboard::KeyCode::F20 => crate::window::Key::F20,
        winit::keyboard::KeyCode::F21 => crate::window::Key::F21,
        winit::keyboard::KeyCode::F22 => crate::window::Key::F22,
        winit::keyboard::KeyCode::F23 => crate::window::Key::F23,
        winit::keyboard::KeyCode::F24 => crate::window::Key::F24,
        _ => crate::window::Key::Unknown,
    }
}
