mod definition;

use jandering_engine::object::{ObjectRenderData, VertexRaw};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BillboardInstance {
    pub size: f32,
    //
    pub position: [f32; 3],
}

pub struct Billboard {
    pub vertices: Vec<VertexRaw>,
    //
    pub indices: Vec<u32>,
    //
    pub instances: Vec<BillboardInstance>,
    //
    pub render_data: ObjectRenderData,
    pub shader: usize,
}
