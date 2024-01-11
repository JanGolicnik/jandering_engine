use super::{Object, Vertex};

pub fn triangle() -> Object {
    Object {
        vertices: vec![
            Vertex {
                position: [0.0, 1.0, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-1.0, -1.0, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ],
        indices: vec![0, 1, 2],
        pipeline: None,
    }
}
