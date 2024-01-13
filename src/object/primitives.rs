use super::{Object, VertexRaw};

pub fn triangle() -> Object {
    Object {
        vertices: vec![
            VertexRaw {
                position: [0.0, 1.0, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            VertexRaw {
                position: [-1.0, -1.0, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            VertexRaw {
                position: [1.0, -1.0, 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ],
        instances: Vec::new(),
        instance_data: Vec::new(),
        indices: vec![0, 1, 2],
        pipeline: None,
    }
}
