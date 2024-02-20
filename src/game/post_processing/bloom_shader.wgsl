struct Resolution {
    res: vec2<f32>,
    padding: vec2<f32>
};

@group(0) @binding(0)
var<uniform> resolution: Resolution;

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
fn fs_blit(in: VertexOutput) -> @location(0) vec4<f32>{
    var uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);

    let dims = textureDimensions(tex) / 2u;
    let dimsf32 = vec2<f32>(f32(dims.x), f32(dims.y));

    var color = vec3<f32>(0.0);
    for(var x = -1.0; x <= 1.0; x += 1.0){
        for(var y = -1.0; y <= 1.0; y += 1.0){
            let offset_uv = ((uv * dimsf32) + vec2<f32>(x,y)) / dimsf32;
            color += textureSample(tex, tex_sampler, offset_uv).xyz;
        }
    }

    color /= 9.0;

    return vec4<f32>(color, 1.0);
}


@fragment
fn fs_thresholdblit(in: VertexOutput) -> @location(0) vec4<f32>{
    var uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);

    let dims = textureDimensions(tex) / 2u;
    let dimsf32 = vec2<f32>(f32(dims.x), f32(dims.y));

    var color = vec3<f32>(0.0);
    for(var x = -1.0; x <= 1.0; x += 1.0){
        for(var y = -1.0; y <= 1.0; y += 1.0){
            let offset_uv = ((uv * dimsf32) + vec2<f32>(x,y)) / dimsf32;
            color += textureSample(tex, tex_sampler, offset_uv).xyz;
        }
    }

    color /= 9.0;

    if(length(sqrt(color)) > 1.0){
        return vec4<f32>(color, 1.0);
    }
    return vec4<f32>(0.0);
}

@fragment
fn fs_tonemap(in: VertexOutput) -> @location(0) vec4<f32>{
    var uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
    var color = textureSample(tex, tex_sampler, uv).xyz;
    color = max(vec3<f32>(0.0), color - 0.004);
    color = (color * (6.2 * color + 0.5)) / (color * (6.2 * color + 1.7) + 0.06);
    color = pow(color, vec3<f32>(2.2));
    return vec4<f32>(color, 1.0);
}