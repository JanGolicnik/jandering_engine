struct Time {
    dt: f32,
    time: f32,
};

@group(0) @binding(0)
var<uniform> time: Time;

struct Resolution {
    resolution: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> resolution: Resolution;

struct InstanceInput{
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

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

    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.uv = model.uv;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32>{

    var uv = in.uv * 2.0 - 1.0;
    uv.x *= resolution.resolution.x / resolution.resolution.y;

    let origin = vec3<f32>(0.0, 0.0, 0.0);
    let direction = normalize(vec3<f32>(uv, 1.0));

    var t = 0.0;

    var i = 0;
    for(; i < 80; i++){
        var position: vec3<f32> = origin + direction * t;
        let percent = f32(i) / 80.0;
        let cos_sin_time = cos(sin(time.time * 0.32));
        let rotated_xy = position.xy * rot2D(t * cos_sin_time * percent) * cos_sin_time * cos_sin_time;
        position.x = rotated_xy.x;
        position.y = rotated_xy.y;

        let distance = map(position);

        t += distance;

        if(distance < 0.001 || t > 100.0){ break; }
    }
 
    let color = pallete(t * 0.02 + (f32(i) / 80.0) * 0.2 );

    return vec4<f32>(color, 1.0);
}

fn dist_octahedron(position: vec3<f32>, s: f32) -> f32 {
    let pos = abs(position);
    return (pos.x + pos.y + pos.z - s) * 0.57735027;
}

fn dist_sphere(position: vec3<f32>, radius: f32) -> f32 {
    return length(position) - radius;
}

fn dist_box(position: vec3<f32>, sides: vec3<f32>) -> f32{
    let q = abs(position) - sides;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y,q.z)), 0.0);
}

fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = max( k - abs(a - b), 0.0) / k;
    return min( a, b) - h * h * h * k * (1.0 / 6.0);
}

fn pallete(t: f32) -> vec3<f32> {
    let a = vec3<f32>(0.5);
    let b = vec3<f32>(0.5);
    let c = vec3<f32>(1.0);
    let d = vec3<f32>(0.263, 0.416, 0.557);

    return a + b * cos( 6.28318 * ( c * t + d ) );
}

fn modulo_euclidean(a: f32, b: f32) -> f32 {
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

fn rot2D(angle: f32) -> mat2x2<f32>{
    let s = sin(angle);
    let c = cos(angle);
    return mat2x2(c, -s, s, c);
}

fn map(position: vec3<f32>) -> f32 {
    var p = position;
    p.z += time.time * 0.9;

    p.x = fract(p.x) - 0.5;
    p.y = fract(p.y) - 0.5;
    p.z = modulo_euclidean(p.z, 0.25) - 0.125;

    let box = dist_octahedron(p, 0.13);

    return box;
}