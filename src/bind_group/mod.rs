use std::any::Any;

use wgpu::BindGroupLayout;

use crate::engine::EngineContext;

pub mod camera;
pub mod resolution;
pub mod texture;
pub mod time;

pub struct BindGroupWriteData<'a> {
    pub queue: &'a wgpu::Queue,
    pub config: &'a wgpu::SurfaceConfiguration,
    pub context: &'a EngineContext<'a>,
}

pub trait BindGroup: Any + BindGroupToAny {
    fn write(&mut self, data: &BindGroupWriteData);
    fn get_bind_group_layout(&self) -> Option<&BindGroupLayout>;
    fn get_bind_group(&self) -> Option<&wgpu::BindGroup>;
}

pub trait BindGroupToAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> BindGroupToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct BindGroupRenderData {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
}
