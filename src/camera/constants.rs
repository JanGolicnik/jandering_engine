#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub const CAMERA_SPEED: f32 = 16.5;
pub const CAMERA_SENSITIVITY: f32 = 0.65;

pub const CAMERA_UP: cgmath::Vector3<f32> = cgmath::Vector3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};
