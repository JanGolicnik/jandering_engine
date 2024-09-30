use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::{Arc, Mutex};
use web_time::Duration;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    window::UserAttentionType,
};

use crate::{Events, FpsPreference, WindowConfig, WindowId, WindowResolution, WindowTrait};

pub struct WinitWindow {
    pub(crate) id: WindowId,

    pub(crate) polled_events: Events, // gets filled when polling

    pub(crate) inner_window: Arc<Mutex<InnerWinitWindow>>,
}

impl WindowTrait for WinitWindow {
    fn resize(&mut self, width: u32, height: u32) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => {
                let _ = window.request_inner_size(PhysicalSize::new(width as f32, height as f32));
            }
            InnerWinitWindow::Uninitalized { ref mut config } => {
                config.resolution = WindowResolution::Exact { width, height }
            }
        }
    }

    fn set_cursor_position(&self, x: u32, y: u32) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => {
                window
                    .set_cursor_position(PhysicalPosition::new(x, y))
                    .unwrap();
            }
            InnerWinitWindow::Uninitalized { .. } => {
                panic!("Attempting to set cursor position on uninitalized window")
            }
        }
    }

    fn size(&self) -> (u32, u32) {
        let window = self.inner_window.lock().unwrap();
        match &*window {
            InnerWinitWindow::Initialized { window, .. } => {
                let size = window.inner_size();
                (size.width, size.height)
            }
            InnerWinitWindow::Uninitalized { config } => match config.resolution {
                WindowResolution::Exact { width, height } => (width, height),
                WindowResolution::Auto => {
                    panic!("Attempting to get window size on uninitalized window with Auto sizing")
                }
            },
        }
    }

    fn width(&self) -> u32 {
        self.size().0
    }

    fn height(&self) -> u32 {
        self.size().1
    }

    fn request_redraw(&mut self) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => window.request_redraw(),
            InnerWinitWindow::Uninitalized { .. } => {
                panic!("Attempting to request redraw on uninitalized window")
            }
        }
    }

    fn close(&mut self) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { should_close, .. } => *should_close = false,
            InnerWinitWindow::Uninitalized { .. } => {
                panic!("Attempting to close uninitialized window")
            }
        }
    }

    fn set_cursor_visible(&mut self, val: bool) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => window.set_cursor_visible(val),
            InnerWinitWindow::Uninitalized { ref mut config } => config.show_cursor = true,
        }
    }

    fn set_title(&mut self, title: &'static str) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => window.set_title(title),
            InnerWinitWindow::Uninitalized { ref mut config } => config.title = title,
        }
    }

    fn set_transparency_enabled(&mut self, value: bool) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => window.set_transparent(value),
            InnerWinitWindow::Uninitalized { ref mut config } => config.transparent = true,
        }
    }

    fn set_visible(&mut self, value: bool) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => window.set_visible(value),
            InnerWinitWindow::Uninitalized { .. } => {
                panic!("Attempting to set visibility of uninitalized window")
            }
        }
    }

    fn set_resizable(&mut self, value: bool) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => window.set_resizable(value),
            InnerWinitWindow::Uninitalized { ref mut config } => config.resizable = value,
        }
    }

    fn set_minimized(&mut self, value: bool) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => window.set_minimized(value),
            InnerWinitWindow::Uninitalized { .. } => {
                panic!("Attempting to minimize uninitialized window")
            }
        }
    }

    fn set_fullscreen(&mut self) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => {
                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
            }
            InnerWinitWindow::Uninitalized { ref mut config } => config.fullscreen = true,
        }
    }

    fn set_borderless(&mut self) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => {
                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
            }
            InnerWinitWindow::Uninitalized { ref mut config } => config.borderless = true,
        }
    }

    fn set_windowed(&mut self) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => window.set_fullscreen(None),
            InnerWinitWindow::Uninitalized { .. } => {
                panic!("Attempting to set uninitialized window to windowed")
            }
        }
    }

    fn set_decorations(&mut self, value: bool) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => window.set_decorations(value),
            InnerWinitWindow::Uninitalized { ref mut config } => config.decorations = value,
        }
    }

    fn focus_window(&mut self) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => window.focus_window(),
            InnerWinitWindow::Uninitalized { .. } => {
                panic!("Attempting to focus uninitialzed window")
            }
        }
    }

    fn request_user_attention(&mut self) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => {
                window.request_user_attention(Some(UserAttentionType::Informational))
            }
            InnerWinitWindow::Uninitalized { .. } => {
                panic!("Attempting to request user attention on uninitialzed window")
            }
        }
    }

    fn position(&self) -> (u32, u32) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => window.outer_position().unwrap().into(),
            InnerWinitWindow::Uninitalized { ref mut config } => match config.position {
                Some(position) => position,
                None => panic!(
                    "Attempting to get position of uninitialized window without a set position"
                ),
            },
        }
    }

    fn set_position(&mut self, x: u32, y: u32) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => {
                window.set_outer_position(PhysicalPosition::new(x, y))
            }
            InnerWinitWindow::Uninitalized { ref mut config } => config.position = Some((x, y)),
        }
    }

    fn set_absolute_position(&mut self, x: u32, y: u32) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => {
                let mut monitors = window.available_monitors();
                let mut top_left = monitors.next().unwrap().position();
                for monitor in monitors {
                    top_left.x = top_left.x.min(monitor.position().x);
                    top_left.y = top_left.y.min(monitor.position().y);
                }

                let new_x = top_left.x as u32 + x;
                let new_y = top_left.y as u32 + y;

                window.set_outer_position(PhysicalPosition::new(new_x, new_y));
            }
            InnerWinitWindow::Uninitalized { ref mut config } => config.position = Some((x, y)),
        }
    }

    fn set_as_desktop(&mut self) {
        let mut window = self.inner_window.lock().unwrap();

        #[allow(unused_variables)]
        let InnerWinitWindow::Initialized { window, .. } = &mut *window
        else {
            panic!("Attempting to set an unitialized window as desktop")
        };

        #[cfg(windows)]
        match window.window_handle().unwrap().as_raw() {
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
                    windows::Win32::Foundation::HWND(isize::from(handle.hwnd)),
                    hwnd,
                );

                window.set_outer_position(PhysicalPosition::new(-10, -40));
                let mut monitors = window.available_monitors();
                let (mut top, mut left, mut bottom, mut right) = {
                    let monitor = monitors.next().unwrap();
                    let pos = monitor.position();
                    let size = monitor.size();
                    (
                        pos.y,
                        pos.x,
                        pos.y + size.height as i32,
                        pos.x + size.width as i32,
                    )
                };
                for monitor in window.available_monitors() {
                    let pos = monitor.position();
                    let size = monitor.size();
                    top = top.min(pos.y);
                    left = left.min(pos.x);
                    bottom = bottom.max(pos.y + size.height as i32);
                    right = right.max(pos.x + size.width as i32);
                }
                let size = (right - left, bottom - top);
                let _ = window.request_inner_size(PhysicalSize::new(size.0, size.1));
            },
            _ => todo!(),
        }
    }

    fn get_window_handle(&self) -> raw_window_handle::WindowHandle {
        use raw_window_handle::WindowHandle;

        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => unsafe {
                std::mem::transmute::<WindowHandle<'_>, WindowHandle<'static>>(
                    window.window_handle().unwrap(),
                )
            },
            InnerWinitWindow::Uninitalized { .. } => {
                panic!("Attempting to get a raw widnow handle on uninitialized window")
            }
        }
    }

    fn get_display_handle(&self) -> raw_window_handle::DisplayHandle {
        use raw_window_handle::DisplayHandle;

        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { window, .. } => unsafe {
                std::mem::transmute::<DisplayHandle<'_>, DisplayHandle<'static>>(
                    window.display_handle().unwrap(),
                )
            },
            InnerWinitWindow::Uninitalized { .. } => {
                panic!("Attempting to get a raw widnow handle on uninitialized window")
            }
        }
    }

    fn set_fps_prefrence(&mut self, preference: crate::FpsPreference) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { fps_preference, .. } => *fps_preference = preference,
            InnerWinitWindow::Uninitalized { ref mut config } => config.fps_preference = preference,
        }
    }

    fn get_fps_prefrence(&self) -> crate::FpsPreference {
        let window = self.inner_window.lock().unwrap();
        match &*window {
            InnerWinitWindow::Initialized { fps_preference, .. } => *fps_preference,
            InnerWinitWindow::Uninitalized { ref config } => config.fps_preference,
        }
    }

    fn events(&self) -> &Events {
        &self.polled_events
    }

    fn id(&self) -> WindowId {
        self.id
    }

    fn should_close(&self) -> bool {
        let window = self.inner_window.lock().unwrap();
        match &*window {
            InnerWinitWindow::Initialized { should_close, .. } => *should_close,
            InnerWinitWindow::Uninitalized { .. } => false,
        }
    }

    fn poll_events(&mut self) {
        let mut window = self.inner_window.lock().unwrap();
        match &mut *window {
            InnerWinitWindow::Initialized { events, .. } => {
                self.polled_events = Events {
                    events: events.events.drain(..).collect(),
                }
            }
            InnerWinitWindow::Uninitalized { .. } => {
                panic!("Attempting to poll events on unitialized window")
            }
        }
    }

    fn is_initialized(&self) -> bool {
        let window = self.inner_window.lock().unwrap();
        matches!(*window, InnerWinitWindow::Initialized { .. })
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum InnerWinitWindow {
    Initialized {
        window: winit::window::Window,

        events: Events,
        #[cfg(target_arch = "wasm32")]
        last_redraw_time: web_time::Instant,
        #[cfg(not(target_arch = "wasm32"))]
        last_redraw_time: std::time::Instant,
        should_close: bool,
        fps_preference: FpsPreference,
    },
    Uninitalized {
        config: WindowConfig,
    },
}

impl InnerWinitWindow {
    pub(crate) fn handle_event(&mut self, event: crate::WindowEvent) {
        let InnerWinitWindow::Initialized {
            events,
            last_redraw_time,
            should_close,
            fps_preference,
            ..
        } = self
        else {
            return;
        };

        match event {
            crate::WindowEvent::RedrawRequested => {
                if let crate::FpsPreference::Exact(fps) = fps_preference {
                    #[cfg(target_arch = "wasm32")]
                    panic!();

                    let now = web_time::Instant::now();
                    let dt = now - *last_redraw_time;
                    let min_dt = Duration::from_millis(1000 / *fps as u64);
                    if dt < min_dt {
                        std::thread::sleep(min_dt - dt);
                    }
                    *last_redraw_time = web_time::Instant::now();
                }
            }
            crate::WindowEvent::CloseRequested => *should_close = true,
            _ => events.push(event),
        }
    }
}

#[cfg(windows)]
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
