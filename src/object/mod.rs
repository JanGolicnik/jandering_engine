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
    pub model: [[f32; 4]; 4],
}

pub struct Instance {
    pub scale: Option<cgmath::Vector3<f32>>,
    //
    pub position: Option<cgmath::Point3<f32>>,
    //
    pub rotation: Option<cgmath::Quaternion<f32>>,
    //
    pub changed: bool,
}

pub struct ObjectRenderData {
    pub vertex_buffer: wgpu::Buffer,
    //
    pub index_buffer: wgpu::Buffer,
    //
    pub instance_buffer: wgpu::Buffer,
}

pub struct Object {
    pub vertices: Vec<VertexRaw>,
    //
    pub indices: Vec<u32>,
    //
    pub instances: Vec<Instance>,
    pub instance_data: Vec<InstanceRaw>,
    //
    pub render_data: Option<ObjectRenderData>,
    pub shader: u32,
}

pub trait Renderable {
    fn bind<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        queue: &mut wgpu::Queue,
        shaders: &'a [wgpu::RenderPipeline],
    );
}
