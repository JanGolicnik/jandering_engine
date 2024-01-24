use std::any::Any;

use wgpu::BindGroupLayout;
use winit::{event::WindowEvent, event_loop::ControlFlow, window::Window};

use crate::renderer::Renderer;

pub mod resolution;
pub mod time;

pub trait Plugin: Any {
    fn event(
        &mut self,
        event: &WindowEvent,
        control_flow: &mut ControlFlow,
        renderer: &mut Renderer,
        window: &Window,
    );
    fn update(&mut self, control_flow: &mut ControlFlow, renderer: &mut Renderer, dt: f64);
    fn pre_render(&mut self, queue: &mut wgpu::Queue);
    fn initialize(&mut self, renderer: &mut Renderer);
    fn get_bind_group_layouts(&self) -> Option<Vec<&BindGroupLayout>>;
    fn get_bind_groups(&self) -> Option<Vec<&wgpu::BindGroup>>;
}
