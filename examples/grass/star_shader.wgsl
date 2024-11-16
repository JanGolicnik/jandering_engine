struct Camera {
    up: vec4<f32>,
    right: vec4<f32>,
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
    sky_color: vec3<f32>,
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

@group(3) @binding(0)
var noise_tex: texture_2d<f32>;
@group(3) @binding(1)
var noise_sampler: sampler;

struct VertexInput{
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coords: vec2<f32>
};

@vertex
fn vs_main(
    vertex: VertexInput,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput{
    var state = instance_index;
    let random_vec = normalize(vec3<f32>(random_float(&state), random_float(&state), random_float(&state)) * 2.0 - 1.0) * 1000.0;
    let position = vec4<f32>(random_vec + camera.view_pos.xyz, 1.0);
    // let position = vec4<f32>(random_vec, 1.0);
    let random_size = random_float(&state);
    let size = 5.0 * (1.0 + random_size * random_size);

    let world_pos = position + camera.right * vertex.position.x * size + camera.up * vertex.position.y * size;
    
    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_pos;
    out.coords = vertex.position.xy;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32>{
    let p = vec2<f32>(in.coords.x, in.coords.y + 0.23);
    var len = 1.0 - length(p);
    len = pow(len + 0.1, 17.0);
    var col = vec3<f32>(1.0);
    return vec4<f32>(col, clamp(len, 0.0, 1.0));
}

fn pcg_hash(state: ptr<function, u32>) -> u32
{
    *state = *state * 747796405u + 2891336453u;
    let word = ((*state >> ((*state >> 28u) + 4u)) ^ *state) * 277803737u;
    *state = (word >> 22u) ^ word; 
    return *state;
}

fn random_float(state: ptr<function, u32>) -> f32{
    return f32(pcg_hash(state)) / f32(0xFFFFFFFFu);
}
