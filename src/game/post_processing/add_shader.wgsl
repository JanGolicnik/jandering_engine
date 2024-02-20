struct Factor {
    factor: f32,
    padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> factor: Factor;

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
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.uv = model.uv;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32>{
    var uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
    let color = textureSample(tex, tex_sampler, uv).xyz;
    return vec4<f32>(color * factor.factor, 1.0);
}