use crate::{
    bind_group::{BindGroup, BindGroupLayout, BindGroupLayoutEntry},
    renderer::{BufferHandle, Janderer, Renderer},
    types::{UVec2, Vec2},
    window::{InputState, Key, MouseButton, Window, WindowEvent},
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct D2CameraData {
    position: Vec2,
    resolution: Vec2,
}

pub struct D2CameraController {
    pub right_pressed: bool,
    pub left_pressed: bool,
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub is_mouse_pressed: bool,
    pub is_shift_pressed: bool,
    pub zoom: f32,
    pub velocity: Vec2,
    pub pan_offset: Vec2,
}

pub struct D2CameraBindGroup {
    pub controller: Option<D2CameraController>,

    pub data: D2CameraData,

    pub position: Vec2,
    resolution: Vec2,

    last_mouse_position: Option<(f32, f32)>,
    pressing: bool,
    mouse_is_inside: bool,
    pub right_click_move: bool,

    buffer_handle: BufferHandle,
}

impl BindGroup for D2CameraBindGroup {
    fn get_data(&self) -> Box<[u8]> {
        bytemuck::cast_slice(&[self.data]).into()
    }

    fn get_layout(&self) -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data(self.buffer_handle)],
        }
    }
}

impl D2CameraBindGroup {
    pub fn resize(&mut self, resolution: UVec2) {
        self.resolution = Vec2::new(resolution.x as f32, resolution.y as f32);
    }

    pub fn update_data(&mut self) {
        self.data.position = -self.position;
        let zoom = if let Some(controller) = self.controller.as_ref() {
            controller.zoom
        } else {
            1.0
        };
        self.data.resolution = self.resolution * zoom;
    }

    pub fn new(renderer: &mut Renderer, resolution: UVec2, with_controller: bool) -> Self {
        let data = D2CameraData {
            position: Vec2::ZERO,
            resolution: Vec2::new(resolution.x as f32, resolution.y as f32),
        };

        let controller = if with_controller {
            Some(D2CameraController::default())
        } else {
            None
        };

        let buffer_handle = renderer.create_uniform_buffer(bytemuck::cast_slice(&[data]));

        Self {
            position: data.position,
            resolution: data.resolution,
            controller,
            data,

            last_mouse_position: None,
            pressing: false,
            mouse_is_inside: false,
            right_click_move: false,

            buffer_handle,
        }
    }

    #[allow(unused_variables)]
    fn handle_events(&mut self, events: &[WindowEvent]) {
        for event in events.iter() {
            if let Some(controller) = self.controller.as_mut() {
                controller.event(event);
            };

            match event {
                WindowEvent::MouseMotion(pos) => {
                    if let Some(controller) = self.controller.as_mut() {
                        let (dx, dy) = if self.mouse_is_inside && self.pressing {
                            let last_mouse_position = self.last_mouse_position.unwrap_or(*pos);
                            let dx = pos.0 - last_mouse_position.0;
                            let dy = pos.1 - last_mouse_position.1;
                            self.last_mouse_position = Some(*pos);
                            (dx, -dy)
                        } else {
                            self.last_mouse_position = None;
                            (0.0, 0.0)
                        };

                        controller.cursor_moved(dx, dy);
                    };
                }
                WindowEvent::MouseEntered => self.mouse_is_inside = true,
                WindowEvent::MouseLeft => self.mouse_is_inside = false,
                WindowEvent::MouseInput { state, button, .. } => {
                    if (self.right_click_move && *button == MouseButton::Right)
                        || (!self.right_click_move && *button == MouseButton::Left)
                    {
                        self.pressing = matches!(state, InputState::Pressed);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn update(&mut self, events: &[WindowEvent], _: &Window, resolution: UVec2, dt: f32) {
        self.resize(resolution);

        self.handle_events(events);

        if let Some(controller) = self.controller.as_mut() {
            controller.update(&mut self.position, dt);
        }

        self.update_data();
    }

    fn resolution(&self) -> Vec2 {
        let zoom = if let Some(controller) = &self.controller {
            controller.zoom
        } else {
            1.0
        };
        self.resolution * zoom
    }

    pub fn mouse_to_world(&self, mut position: Vec2) -> Vec2 {
        position.y = self.resolution.y - position.y;
        let resolution = self.resolution();
        position.x = (position.x / self.resolution.x) * resolution.x;
        position.y = (position.y / self.resolution.y) * resolution.y;
        position += self.position - resolution / 2.0;
        position
    }

    pub fn get_layout() -> BindGroupLayout {
        BindGroupLayout {
            entries: vec![BindGroupLayoutEntry::Data(BufferHandle::uniform(0))],
        }
    }
}

impl Default for D2CameraController {
    fn default() -> Self {
        Self {
            right_pressed: false,
            left_pressed: false,
            up_pressed: false,
            down_pressed: false,
            is_shift_pressed: false,
            zoom: 1.0,
            is_mouse_pressed: false,
            velocity: Vec2 { x: 0.0, y: 0.0 },
            pan_offset: Vec2 { x: 0.0, y: 0.0 },
        }
    }
}

impl D2CameraController {
    pub fn cursor_moved(&mut self, dx: f32, dy: f32) {
        self.pan_offset.x += dx * self.zoom * 2.0;
        self.pan_offset.y += dy * self.zoom * 2.0;
    }

    pub fn event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Scroll((_, y)) => self.zoom = (self.zoom - *y / 10.0).max(0.2),
            WindowEvent::KeyInput { state, key } => {
                let is_pressed = InputState::Pressed == *state;
                match key {
                    Key::A => self.left_pressed = is_pressed,
                    Key::D => self.right_pressed = is_pressed,
                    Key::S => self.up_pressed = is_pressed,
                    Key::W => self.down_pressed = is_pressed,
                    Key::Shift => self.is_shift_pressed = is_pressed,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self, object_position: &mut Vec2, dt: f32) {
        let Self {
            right_pressed,
            left_pressed,
            up_pressed,
            down_pressed,
            is_shift_pressed,
            ..
        } = *self;

        let speed = if is_shift_pressed { 2.0 } else { 1.0 };
        if left_pressed {
            self.velocity.x = speed;
        }
        if right_pressed {
            self.velocity.x = -speed;
        }
        if up_pressed {
            self.velocity.y = speed;
        }
        if down_pressed {
            self.velocity.y = -speed;
        }

        *object_position += self.velocity * dt * 2000.0;
        self.velocity += -self.velocity * (dt * 6.0);

        object_position.x += self.pan_offset.x;
        object_position.y += self.pan_offset.y;

        self.pan_offset.x = 0.0;
        self.pan_offset.y = 0.0;
    }
}
