struct Camera {
    up: vec4<f32>,
    right: vec4<f32>,
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct InstanceInput{
    @location(5) position: vec3<f32>,
    @location(6) size: f32,
}

struct VertexInput{
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coords: vec2<f32>
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput
) -> VertexOutput{

    // let world_pos = vec4(instance.position, 1.0) + camera.right * vertex.position.x * instance.size + camera.up * vertex.position.y * instance.size;
    let world_pos = vec4(instance.position, 1.0) + camera.right * vertex.position.x * instance.size + camera.up * vertex.position.y * instance.size;
    // let world_pos =  vec4(instance.position, 1.0) + camera.right * vertex.position.x + camera.up * vertex.position.y;
    
    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_pos;
    out.coords = vertex.position.xy;
    return out;
}

@fragment
fn fs_main(in: VertexOutput, @builtin(front_facing) facing: bool) -> @location(0) vec4<f32>{
    let p = vec2<f32>(in.coords.x, in.coords.y + 0.23);
    var len = 1.0 - length(p);
    len = pow(len + 0.1, 17.0);
    var col = vec3<f32>(1.0);
    if facing {
        col = vec3<f32>(0.0, 1.0, 0.0);
    }
    return vec4<f32>(col, clamp(len, 0.0, 1.0));
}