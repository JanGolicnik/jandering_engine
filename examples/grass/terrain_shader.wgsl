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

struct GrassVertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pos: vec3<f32>,
    @location(1) terrain_pos_t: vec2<f32>,
    @location(2) height: f32,
    @location(3) index: u32,
    @location(4) normal: vec3<f32>,
};

@vertex
fn vs_grass(
    model: VertexInput,
    @builtin(instance_index) instance_index: u32
) -> GrassVertexOutput{


    var v_pos = model.position;
    var v_normal = model.normal;

    let camera_pos_t = camera.view_pos.xz / render_data.render_square_size;

    var grid_pos_u = vec2<u32>(instance_index % render_data.sqrt_n_grass, instance_index / render_data.sqrt_n_grass);
    var grid_pos = vec2<i32>(grid_pos_u);
    grid_pos += vec2<i32>(camera_pos_t * f32(render_data.sqrt_n_grass));
    let grid_index = grid_pos.x + grid_pos.y * i32(render_data.sqrt_n_grass);
    var pos = vec2<f32>(grid_pos) / f32(render_data.sqrt_n_grass);

    var rng_state = u32(grid_index);

    let height_t = model.position.y / 2.0;
    {
        let angle = (random_float(&rng_state) - 0.5) * 2.0 * 3.14159;
        let cos_a = cos(angle); 
        let sin_a = sin(angle);

        v_pos.y *= (1.0 - render_data.grass_height_variation * 0.5) + random_float(&rng_state) * render_data.grass_height_variation;
        v_pos.y *= render_data.grass_height;
        v_pos = vec3<f32>(cos_a * v_pos.x - sin_a * v_pos.z, v_pos.y, sin_a * v_pos.x + cos_a * v_pos.z);

        v_normal = vec3<f32>(cos_a * v_normal.x - sin_a * v_normal.z, v_normal.y, sin_a * v_normal.x + cos_a * v_normal.z);
    }

    pos = (pos - 0.5) * render_data.render_square_size;
    
    pos.x += random_float(&rng_state) - 0.5;
    pos.y += random_float(&rng_state) - 0.5;

    let terrain_pos_t = pos / render_data.terrain_size + 0.5;
    let y = textureSampleLevel(tex, tex_sampler, terrain_pos_t, 0.0).r;
    
    let wind = calculate_wind(terrain_pos_t) - render_data.wind_strength * 0.1;
    v_pos += wind * height_t; 

    let final_pos = v_pos + vec3<f32>(pos.x, y * 75.0, pos.y);
    // let final_pos = v_pos + vec3<f32>(f32(grid_pos.x), 0.0, f32(grid_pos.y));

    let ss_pos = camera.view_proj * vec4<f32>(final_pos, 1.0);

    var out: GrassVertexOutput;

    out.clip_position = ss_pos;
    out.height = height_t;
    out.pos = final_pos.xyz;
    out.terrain_pos_t = terrain_pos_t;
    out.index = instance_index;
    out.normal = v_normal;
    return out;
}


@fragment
fn fs_grass(in: GrassVertexOutput, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32>{

    let bottom_color = render_data.ground_color;
    let top_color = render_data.grass_top_color;
    let noise = 1.0 - textureSample(noise_tex, noise_sampler, in.terrain_pos_t).r * 0.5;
    
    var color = bottom_color * (1.0 - in.height) + top_color * in.height;
    color.b *= noise;

    var normal = in.normal;
    if front_facing{
        normal = -normal;
    }

    let shaded = get_phong_color(in.pos, normal, color, 1.0);
    
    let dist = 1.0 - abs(camera.view_pos.xz - in.pos.xz) / (render_data.render_square_size * 0.5);
    var alpha = min(dist.x, dist.y);
    alpha = 1.0 - pow(1.0 - alpha, 5.0);

    return vec4<f32>(shaded, alpha);

    // return vec4<f32>(vec3<f32>(d), 1.0);

    ///////// ALPHA //////// 
    // return vec4<f32>(color.r, color.g, color.b * noise, 0.5);
    // return vec4<f32>(alpha * 0.9);

    ///////// WIND //////// 
    // let instance_index = in.index;
    // let coords = vec2<f32>(f32(instance_index % render_data.sqrt_n_grass), f32(instance_index / render_data.sqrt_n_grass)) / f32(render_data.sqrt_n_grass);
    // let wind = calculate_wind(in.terrain_pos_t);
    // return vec4<f32>(vec3<f32>(wind), 1.0);


    //////// RANDOM //////// 
    // let grid_pos = vec2<u32>(instance_index % render_data.sqrt_n_grass, instance_index / render_data.sqrt_n_grass); 
    // let i = grid_pos.x + grid_pos.y * render_data.sqrt_n_grass;
    // let t = f32(instance_index) / f32(render_data.sqrt_n_grass * render_data.sqrt_n_grass);
    // let c = random_float(i);
    // return vec4<f32>(vec3<f32>(c), 1.0);
}

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

struct GroundVertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

@vertex
fn vs_ground(
    model: VertexInput,
    instance: InstanceInput,
    @builtin(instance_index) instance_index: u32
) -> GroundVertexOutput {
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

    var out: GroundVertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(pos, 1.0);
    out.normal = normalize(model.normal);
    out.uv = model.uv;
    out.world_pos = pos;
    
    return out;
}


@fragment
fn fs_ground(in: GroundVertexOutput) -> @location(0) vec4<f32>{

    let color = render_data.ground_color;

    // let eps = 1.0 / 700.0; // heightmap size
    // let fx0 = textureSample(tex, tex_sampler, vec2<f32>(in.uv.x + eps, in.uv.y)).r;
    // let fx1 = textureSample(tex, tex_sampler, vec2<f32>(in.uv.x - eps, in.uv.y)).r;
    // let fy0 = textureSample(tex, tex_sampler, vec2<f32>(in.uv.x, in.uv.y + eps)).r;
    // let fy1 = textureSample(tex, tex_sampler, vec2<f32>(in.uv.x, in.uv.y - eps)).r;

    // let normal = normalize(vec3<f32>(fx0 - fx1, fy0 - fy1, 0.0));

    let shaded = get_phong_color(in.world_pos, vec3<f32>(0.0, 1.0, 0.0), color, 0.0);

    let dist = 1.0 - abs(camera.view_pos.xz - in.world_pos.xz) / (render_data.terrain_size * 0.5);
    var alpha = min(dist.x, dist.y);
    alpha = 1.0 - pow(1.0 - alpha, 5.0);

    return vec4<f32>(shaded, alpha);
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


fn calculate_wind(coords: vec2<f32>) -> f32{
    let time = render_data.time * render_data.wind_speed;

    let noise = textureSampleLevel(noise_tex, noise_sampler, coords * render_data.wind_noise_scale + time * 0.01, 0.0).r;

    var pos = (coords.x + coords.y) + noise * render_data.wind_noise_strength;

    return sin(pos * render_data.wind_scale + time) * render_data.wind_strength;
}

fn get_phong_color(world_pos: vec3<f32>, normal: vec3<f32>, color: vec3<f32>, shading_strength: f32) -> vec3<f32>{
    // Set up variables
    let view_dir = normalize(camera.view_pos.xyz - world_pos);
    let light_dir = -view_dir;
    // let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let light_color = 0.7;
    let reflect_dir = reflect(-light_dir, normal);
    let halfway_dir = normalize(light_dir + view_dir);
    
    // Get ambient
    let ambient = 0.1;

    // Get specular
    let specular_t = pow(max(dot(normal, halfway_dir), 0.0), 28.0);
    let specular = vec3<f32>(specular_t * 0.5);

    // Get diffuse
    var diff = max(dot(light_dir, normal), 0.0);

    let bottom_color = render_data.ground_color;
    let top_color = render_data.grass_top_color;
    var diffuse = color;
    diffuse *= 1.0 - (1.0 - diff) * shading_strength;
    diffuse *= light_color;

    return ambient + diffuse + specular * shading_strength;
}
