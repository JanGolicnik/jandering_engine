pub trait Window {
    fn resize(&mut self, width: u32, height: u32);

    fn set_cursor_position(&self, x: u32, y: u32);

    fn set_cursor_visible(&mut self, val: bool);

    fn size(&self) -> (u32, u32);

    fn run(&mut self, event_handler: Box<dyn WindowEventHandler>);

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

    fn set_decorations(&mut self, value: bool);

    fn focus_window(&mut self);

    fn request_user_attention(&mut self);

    fn get_raw_window_handle(&self) -> raw_window_handle::RawWindowHandle;

    fn get_raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle;
}

unsafe impl raw_window_handle::HasRawWindowHandle for dyn Window {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.get_raw_window_handle()
    }
}

unsafe impl raw_window_handle::HasRawDisplayHandle for dyn Window {
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        self.get_raw_display_handle()
    }
}

// Adapted (taken 😈) from winit
#[derive(Copy, Debug, Clone)]
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

#[derive(Copy, Debug, Clone)]
pub enum InputState {
    Released,
    Pressed,
}

#[derive(Copy, Debug, Clone)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Unknown,
}

#[derive(Copy, Debug, Clone)]
pub enum WindowEvent {
    Resized((u32, u32)),

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
    EventsCleared,
}

pub struct WindowBuilder {
    pub title: &'static str,
    pub width: u32,
    pub height: u32,
    pub show_cursor: bool,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            width: 800,
            height: 800,
            show_cursor: true,
            title: "Cool app",
        }
    }
}

impl WindowBuilder {
    pub fn with_resolution(mut self, w: u32, h: u32) -> Self {
        self.width = w;
        self.height = h;
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
}

pub trait WindowEventHandler {
    fn on_event(&mut self, event: WindowEvent, window: &mut dyn Window);
}
