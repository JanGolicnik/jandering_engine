use jandering_engine::{
    object::{primitives::triangle_data, Renderable, Vertex},
    renderer::{BufferHandle, Janderer, Renderer},
};

pub struct StarObject {
    #[allow(dead_code)]
    pub vertices: Vec<Vertex>,
    //
    pub indices: Vec<u32>,
    //
    pub vertex_buffer: BufferHandle,
    //
    pub index_buffer: BufferHandle,
    //
    pub n_instances: u32,
}

impl StarObject {
    pub fn new(renderer: &mut Renderer, n_instances: u32) -> Self {
        let (vertices, indices) = triangle_data();

        let vertex_buffer = renderer.create_vertex_buffer(bytemuck::cast_slice(&vertices));
        let index_buffer = renderer.create_index_buffer(bytemuck::cast_slice(&indices));

        Self {
            vertices,
            indices,
            n_instances,
            vertex_buffer,
            index_buffer,
        }
    }
}

impl Renderable for StarObject {
    fn num_instances(&self) -> u32 {
        self.n_instances
    }

    fn num_indices(&self) -> u32 {
        self.indices.len() as u32
    }

    fn get_buffers(&self) -> (BufferHandle, BufferHandle, Option<BufferHandle>) {
        (self.vertex_buffer, self.index_buffer, None)
    }
}
