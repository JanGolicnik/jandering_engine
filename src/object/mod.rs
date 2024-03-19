use std::ops::Range;

use crate::types::*;

pub mod constants;
mod definition;
pub mod primitives;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexRaw {
    pub position: Vec3,
    pub uv: Vec2,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub model: Mat4,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct D2Instance {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
}

pub struct ObjectRenderData {
    pub vertex_buffer: wgpu::Buffer,
    //
    pub index_buffer: wgpu::Buffer,
    //
    pub instance_buffer: wgpu::Buffer,
}

pub struct Object<T> {
    pub vertices: Vec<VertexRaw>,
    //
    pub indices: Vec<u32>,
    //
    pub instances: Vec<T>,
    //
    pub render_data: Option<ObjectRenderData>,

    previous_instances_len: usize,
}

pub trait Renderable {
    fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, range: Range<u32>);

    fn num_instances(&self) -> u32;
}
