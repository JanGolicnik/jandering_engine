struct Camera {
    position: vec2<f32>,
    resolution: vec2<f32>
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var tex: texture_2d<f32>;
@group(1) @binding(1)
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
    vertex_pos *= instance.scale * 0.5;
    
    let sin_a = sin(instance.rotation);
    let cos_a = cos(instance.rotation);
    vertex_pos = vec2<f32>(vertex_pos.x * cos_a - vertex_pos.y * sin_a, vertex_pos.x * sin_a + vertex_pos.y * cos_a);

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
    return c;
}