use crate::pipeline::Pipeline;

mod definition;
pub mod primitives;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

pub struct Instance {}

pub struct Object {
    pub vertices: Vec<Vertex>,
    //
    pub indices: Vec<u32>,
    //
    // pub instances: Vec<Instance>,
    //
    pub scale: cgmath::Vector3<f32>,
    //
    pub position: cgmath::Vector3<f32>,
    //
    pub rotation: cgmath::Vector3<f32>,
    //
    pub pipeline: Option<Pipeline>,
}
