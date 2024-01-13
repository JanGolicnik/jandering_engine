use crate::pipeline::Pipeline;

pub mod constants;
mod definition;
pub mod primitives;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexRaw {
    position: [f32; 3],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

pub struct Instance {
    pub scale: Option<cgmath::Vector3<f32>>,
    //
    pub position: Option<cgmath::Vector3<f32>>,
    //
    pub rotation: Option<cgmath::Quaternion<f32>>,
    //
    pub changed: bool,
}

pub struct Object {
    pub vertices: Vec<VertexRaw>,
    //
    pub indices: Vec<u32>,
    //
    pub instances: Vec<Instance>,
    pub instance_data: Vec<InstanceRaw>,
    //
    pub pipeline: Option<Pipeline>,
}
