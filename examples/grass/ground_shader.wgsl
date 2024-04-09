struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct RenderData {
    ground_color: vec3<f32>,
    time: f32,
    grass_top_color: vec3<f32>,
    grass_height: f32,
    grass_height_variation: f32,
    wind_strength: f32,
    wind_scale: f32,
    wind_speed: f32,
    wind_direction: f32,
    wind_noise_scale: f32,
    wind_noise_strength: f32,
    sqrt_n_grass: u32,
    terrain_size: f32,
    render_square_size: f32,
    fov_x: f32,
};

@group(1) @binding(0)
var<uniform> render_data: RenderData;

@group(2) @binding(0)
var tex: texture_2d<f32>;
@group(2) @binding(1)
var tex_sampler: sampler;


struct InstanceInput{
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,

    @location(9)  inv_model_matrix_0: vec4<f32>,
    @location(10) inv_model_matrix_1: vec4<f32>,
    @location(11) inv_model_matrix_2: vec4<f32>,
    @location(12) inv_model_matrix_3: vec4<f32>,
}

struct VertexInput{
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput{
    let tex_size = textureDimensions(tex).xy;

    let height = textureSampleLevel(tex, tex_sampler, model.uv, 0.0).r;

    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let vertex_pos = model_matrix * vec4<f32>(model.position, 1.0);
    let pos = vec3<f32>(vertex_pos.x, height * 75.0, vertex_pos.z);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(pos, 1.0);
    out.normal = normalize(model.normal);
    out.uv = model.uv;
    out.world_pos = pos;
    
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32>{
    let dist = 1.0 - abs(camera.view_pos.xz - in.world_pos.xz) / (render_data.terrain_size * 0.5);
    let alpha = min(dist.x, dist.y);

    return vec4<f32>(render_data.ground_color * 1.1, alpha);
    // return vec4<f32>(1.0);
}


