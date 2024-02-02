use std::any::Any;

use wgpu::BindGroupLayout;

use crate::{engine::EngineContext, renderer::Renderer};

pub mod resolution;
pub mod time;

pub trait Plugin: Any {
    fn update(&mut self, context: &mut EngineContext, renderer: &mut Renderer);
    fn get_bind_group_layout(&self) -> Option<&BindGroupLayout>;
    fn get_bind_group(&self) -> Option<&wgpu::BindGroup>;
}
