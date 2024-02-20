struct Camera {
    position: vec2<f32>,
    resolution: vec2<f32>
};

struct Hue {
    h: f32,
    padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> hue: Hue;

@group(2) @binding(0)
var tex: texture_2d<f32>;
@group(2) @binding(1)
var tex_sampler: sampler;


struct InstanceInput{
    @location(5) position: vec2<f32>,
    @location(6) scale: vec2<f32>,
    @location(7) rotation: f32,
}

struct VertexInput{
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput{
    var vertex_pos = model.position.xy;
    let sin_a = sin(instance.rotation);
    let cos_a = cos(instance.rotation);
    vertex_pos = vec2<f32>(vertex_pos.x * cos_a - vertex_pos.y * sin_a, vertex_pos.x * sin_a + vertex_pos.y * cos_a);
    vertex_pos *= instance.scale * 0.5;

    var position = (camera.position + round(instance.position + vertex_pos)) / (camera.resolution * 0.5);

    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.uv = model.uv;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32>{
    let uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
    let c: vec4<f32> = textureSample(tex, tex_sampler, uv);
    let h = hue_to_rgb(hue.h);

    let color =c * vec4<f32>(h, 1.0); 

    return color;
}

fn hue_to_rgb(hue: f32) -> vec3<f32> {
    let m: vec3<f32> = modulo_vec(vec3<f32>(0.0, 4.0, 2.0) + vec3<f32>(hue * 6.0), 6.0);
    let a: vec3<f32> = abs(m - 3.0);
    let c: vec3<f32> = clamp(a - 1.0, vec3<f32>(0.0), vec3<f32>(1.0));
    return c + 0.5;
}

fn modulo_vec(a: vec3<f32>, b: f32) -> vec3<f32> {
	var m: vec3<f32> = a % b;
    return vec3<f32>(modulo_f(m.r, b), modulo_f(m.g, b), modulo_f(m.b, b));
}

fn modulo_f(a: f32, b: f32) -> f32 {
	var m = a % b;
	if (m < 0.0) {
		if (b < 0.0) {
			m -= b;
		} else {
			m += b;
		}
	}
	return m;
}