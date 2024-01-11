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
    // pub scale: glm::Vec3,
    // //
    // pub position: glm::Vec3,
    // //
    // pub rotation: glm::Vec3,
    //
    pub pipeline: Option<Pipeline>,
}
