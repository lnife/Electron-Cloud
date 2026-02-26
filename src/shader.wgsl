// Uniforms accessible to the vertex shader
struct Camera {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

// Struct for vertex shader output / fragment shader input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    // Sphere model vertex position
    @location(0) model_pos: vec3<f32>,
    // Per-instance attributes
    @location(1) instance_pos: vec3<f32>,
    @location(2) instance_color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let scale = 0.05;
    out.clip_position = camera.view_proj * vec4<f32>(model_pos * scale + instance_pos, 1.0);
    out.color = instance_color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
