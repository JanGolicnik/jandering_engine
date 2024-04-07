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
    @location(0) pos: vec2<f32>,
    @location(1) terrain_pos_t: vec2<f32>,
    @location(2) height: f32,
    @location(3) index: u32,
};

@vertex
fn vs_main(
    model: VertexInput,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput{
    var v_pos = model.position;

    let camera_pos_t = camera.view_pos.xz / render_data.render_square_size;

    var grid_pos_u = vec2<u32>(instance_index % render_data.sqrt_n_grass, instance_index / render_data.sqrt_n_grass);
    var grid_pos = vec2<i32>(grid_pos_u);
    grid_pos += vec2<i32>(camera_pos_t * f32(render_data.sqrt_n_grass));
    let grid_index = grid_pos.x + grid_pos.y * i32(render_data.sqrt_n_grass);
    var pos = vec2<f32>(grid_pos) / f32(render_data.sqrt_n_grass);

    let height_t = model.position.y / 2.0;
    {
        let angle = (random_float(grid_index + 20) - 0.5) * 2.0 * 3.14159;
        let cos_a = cos(angle); 
        let sin_a = sin(angle);

        v_pos.y *= (1.0 - render_data.grass_height_variation * 0.5) + random_float(grid_index + 30) * render_data.grass_height_variation;
        v_pos.y *= render_data.grass_height;
        v_pos = vec3<f32>(cos_a * v_pos.x - sin_a * v_pos.z, v_pos.y, sin_a * v_pos.x + cos_a * v_pos.z);
    }

    pos = (pos - 0.5) * render_data.render_square_size;
    
    pos.x += random_float(grid_index) - 0.5;
    pos.y += random_float(grid_index + 10) - 0.5;

    let terrain_pos_t = pos / render_data.terrain_size + 0.5;
    let y = textureSampleLevel(tex, tex_sampler, terrain_pos_t, 0.0).r;
    
    let wind = calculate_wind(terrain_pos_t) - render_data.wind_strength * 0.1;
    v_pos += wind * height_t; 

    let final_pos = v_pos + vec3<f32>(pos.x, y * 75.0, pos.y);
    // let final_pos = v_pos + vec3<f32>(f32(grid_pos.x), 0.0, f32(grid_pos.y));

    let ss_pos = camera.view_proj * vec4<f32>(final_pos, 1.0);

    var out: VertexOutput;

    out.clip_position = ss_pos;
    out.height = height_t;
    out.pos = final_pos.xz;
    out.terrain_pos_t = terrain_pos_t;
    out.index = instance_index;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32>{
    let bottom_color = render_data.ground_color;
    let top_color = render_data.grass_top_color;
    
    let color = bottom_color * (1.0 - in.height) + top_color * in.height;

    let noise = 1.0 - textureSample(noise_tex, noise_sampler, in.terrain_pos_t).r * 0.5;

    let dist = 1.0 - abs(camera.view_pos.xz - in.pos) / (render_data.render_square_size * 0.5);
    let alpha = min(dist.x, dist.y);

    return vec4<f32>(color.r, color.g, color.b * noise, alpha * 0.9);

    // return vec4<f32>(color.r, color.g, color.b * noise, 0.5);
  
    // let instance_index = in.index;
    // let coords = vec2<f32>(f32(instance_index % render_data.sqrt_n_grass), f32(instance_index / render_data.sqrt_n_grass)) / f32(render_data.sqrt_n_grass);
    // return vec4<f32>(alpha * 0.9);
    // let wind = calculate_wind(normalized_pos);
    // return vec4<f32>(vec3<f32>(wind), 1.0);

    // let grid_pos = vec2<u32>(instance_index % render_data.sqrt_n_grass, instance_index / render_data.sqrt_n_grass); 
    // let i = grid_pos.x + grid_pos.y * render_data.sqrt_n_grass;
    // let t = f32(instance_index) / f32(render_data.sqrt_n_grass * render_data.sqrt_n_grass);
    // let c = random_float(i);
    // return vec4<f32>(vec3<f32>(c), 1.0);
}

fn pcg_hash(input: u32) -> u32
{
    let state = input * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn random_float(input: i32) -> f32{
    return f32(pcg_hash(u32(input))) / f32(0xFFFFFFFFu);
}

fn calculate_wind(coords: vec2<f32>) -> f32{
    let time = render_data.time * render_data.wind_speed;

    let noise = textureSampleLevel(noise_tex, noise_sampler, coords * render_data.wind_noise_scale + time * 0.01, 0.0).r;

    var pos = (coords.x + coords.y) + noise * render_data.wind_noise_strength;

    return sin(pos * render_data.wind_scale + time) * render_data.wind_strength;
}