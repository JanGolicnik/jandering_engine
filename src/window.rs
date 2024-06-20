use crate::implementation::window::winit::WinitWindow;

pub type Window = WinitWindow;

pub trait WindowTrait {
    fn resize(&mut self, width: u32, height: u32);

    fn set_position(&mut self, x: i32, y: i32);

    fn set_cursor_position(&self, x: u32, y: u32);

    fn set_cursor_visible(&mut self, val: bool);

    fn size(&self) -> (u32, u32);

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn request_redraw(&mut self);

    fn close(&mut self);

    fn set_title(&mut self, title: &'static str);

    fn set_transparency_enabled(&mut self, value: bool);

    fn set_visible(&mut self, value: bool);

    fn set_resizable(&mut self, value: bool);

    fn set_minimized(&mut self, value: bool);

    fn set_fullscreen(&mut self);

    fn set_borderless(&mut self);

    fn set_windowed(&mut self);

    fn set_as_desktop(&mut self);

    fn set_decorations(&mut self, value: bool);

    fn set_fps_prefrence(&mut self, preference: FpsPreference);

    fn get_fps_prefrence(&self) -> FpsPreference;

    fn focus_window(&mut self);

    fn request_user_attention(&mut self);

    fn get_raw_window_handle(&self) -> Option<raw_window_handle::RawWindowHandle>;

    fn get_raw_display_handle(&self) -> Option<raw_window_handle::RawDisplayHandle>;
}

unsafe impl raw_window_handle::HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.get_raw_window_handle().unwrap()
    }
}

unsafe impl raw_window_handle::HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        self.get_raw_display_handle().unwrap()
    }
}

// Adapted (taken 😈) from winit
#[derive(Copy, Debug, Clone, PartialEq)]
pub enum Key {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Delete,
    Left,
    Up,
    Right,
    Down,
    Backspace,
    Return,
    Space,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    Alt,
    Control,
    Shift,
    Minus,
    Plus,
    Tab,
    Copy,
    Paste,
    Cut,
    Enter,

    Unknown,
}

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum InputState {
    Released,
    Pressed,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Unknown,
}

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum WindowEvent {
    Resized((u32, u32)),
    ScaleFactorChanged(f32),

    MouseMotion((f32, f32)),

    Scroll((f32, f32)),

    KeyInput {
        key: Key,
        state: InputState,
    },

    MouseInput {
        button: MouseButton,
        state: InputState,
    },
    RedrawRequested,
    CloseRequested,

    MouseEntered,
    MouseLeft,
}

pub enum WindowResolution {
    Exact { width: u32, height: u32 },
    Auto,
}

#[derive(Clone, Copy)]
pub enum FpsPreference {
    Vsync,
    Exact(u32),
    Uncapped,
}

pub struct WindowConfig {
    pub title: &'static str,
    pub resolution: WindowResolution,
    pub show_cursor: bool,
    pub fps_preference: FpsPreference,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            resolution: WindowResolution::Auto,
            show_cursor: true,
            title: "Cool app",
            fps_preference: FpsPreference::Vsync,
        }
    }
}

impl WindowConfig {
    pub fn with_resolution(mut self, w: u32, h: u32) -> Self {
        self.resolution = WindowResolution::Exact {
            width: w,
            height: h,
        };
        self
    }
    pub fn with_auto_resolution(mut self) -> Self {
        self.resolution = WindowResolution::Auto;
        self
    }
    pub fn with_cursor(mut self, val: bool) -> Self {
        self.show_cursor = val;
        self
    }
    pub fn with_title(mut self, title: &'static str) -> Self {
        self.title = title;
        self
    }
    pub fn with_fps_preference(mut self, fps_preference: FpsPreference) -> Self {
        self.fps_preference = fps_preference;
        self
    }
}

#[cfg(target_arch = "wasm32")]
use crate::engine::EngineEvent;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;

pub trait WindowEventHandler<T> {
    fn on_event(&mut self, event: WindowEvent, window: &mut Window);

    fn on_custom_event(&mut self, event: T, window: &mut Window);

    #[cfg(not(target_arch = "wasm32"))]
    fn window_created(&mut self, window: &mut Window);

    #[cfg(target_arch = "wasm32")]
    fn window_created(
        &mut self,
        window: Arc<std::sync::Mutex<Window>>,
        event_loop_proxy: EventLoopProxy<EngineEvent>,
    );
}
