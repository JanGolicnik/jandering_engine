use winit_window::WinitWindow;
use winit_window_manager::WinitWindowManager;

pub mod winit_window;
pub mod winit_window_manager;

pub type Window = WinitWindow;
pub type WindowManager = WinitWindowManager;

pub type WindowId = u32;
pub trait WindowManagerTrait {
    fn new() -> Self;

    fn run(self, function: impl FnMut(&mut WindowManager));

    fn spawn_window(&mut self, config: WindowConfig) -> Window;

    fn end(&mut self);
}

pub trait WindowTrait {
    fn id(&self) -> u32;

    fn resize(&mut self, width: u32, height: u32);

    fn position(&self) -> (u32, u32);

    fn is_initialized(&self) -> bool;

    fn set_position(&mut self, x: u32, y: u32);

    fn set_absolute_position(&mut self, x: u32, y: u32);

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

    fn events(&self) -> &Events;

    fn get_window_handle(&self) -> raw_window_handle::WindowHandle;

    fn get_display_handle(&self) -> raw_window_handle::DisplayHandle;

    fn should_close(&self) -> bool;

    fn poll_events(&mut self);
}

impl raw_window_handle::HasWindowHandle for Window {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        Ok(self.get_window_handle())
    }
}

impl raw_window_handle::HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        Ok(self.get_display_handle())
    }
}

// Adapted (taken 😈) from winit
#[derive(Copy, Debug, Clone, PartialEq, Hash, Eq)]
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
    RawMouseMotion((f32, f32)),

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

    WindowInitialized
}

#[derive(Debug)]
pub enum WindowResolution {
    Exact { width: u32, height: u32 },
    Auto,
}

#[derive(Clone, Copy, Debug)]
pub enum FpsPreference {
    Vsync,
    Exact(u32),
    Uncapped,
}

#[derive(Debug)]
pub struct WindowConfig {
    pub title: &'static str,
    pub resolution: WindowResolution,
    pub fps_preference: FpsPreference,
    pub position: Option<(u32, u32)>,
    pub show_cursor: bool,
    pub transparent: bool,
    pub decorations: bool,
    pub resizable: bool,
    pub fullscreen: bool,
    pub borderless: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Cool app",
            resolution: WindowResolution::Auto,
            fps_preference: FpsPreference::Vsync,
            position: None,
            show_cursor: true,
            transparent: false,
            decorations: true,
            resizable: true,
            fullscreen: false,
            borderless: false
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

    pub fn with_transparency(mut self, value: bool) -> Self {
        self.transparent = value;
        self
    }
    pub fn with_decorations(mut self, value: bool) -> Self {
        self.decorations = value;
        self
    }
    pub fn with_fullscreen(mut self, value: bool) -> Self {
        self.fullscreen = value;
        self
    }
    pub fn with_borderless(mut self, value: bool) -> Self {
        self.borderless = value;
        self
    }
    pub fn with_position(mut self, x: u32, y: u32) -> Self {
        self.position= Some((x,y));
        self            

    }
    pub fn resizable(mut self, value: bool) -> Self {
        self.resizable = value;
        self
    }
}

#[derive(Default, Debug)]
pub struct Events {
    events: Vec<WindowEvent>,
}

impl Events {
    pub fn with_initialized() -> Self{
        Self { events: vec![WindowEvent::WindowInitialized] }
    }

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
                state: crate::InputState::Pressed,
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
                state: crate::InputState::Pressed,
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

    pub fn iter(&self) -> std::slice::Iter<WindowEvent> {
        self.events.iter()
    }

    pub fn clear(&mut self) {
        self.events.clear()
    }
}
