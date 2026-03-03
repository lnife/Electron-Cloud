// uniform block passed from cpu to gpu
// contains the combined view and projection matrix
// this transforms world coordinates into clip space
struct Camera {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;


// data sent from vertex shader to fragment shader
// clip_position is required by the gpu pipeline
// color is passed through for final pixel shading
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}


@vertex
fn vs_main(
    // base sphere vertex position (unit sphere geometry)
    @location(0) model_pos: vec3<f32>,

    // per-instance position offset (each particle location)
    @location(1) instance_pos: vec3<f32>,

    // per-instance color computed from probability density
    @location(2) instance_color: vec4<f32>,
) -> VertexOutput {

    var out: VertexOutput;

    // small uniform scale so each sampled point becomes a tiny sphere
    let scale = 0.05;

    // final position = scaled sphere vertex + instance offset
    // then transformed by view_proj into clip space
    out.clip_position =
        camera.view_proj *
        vec4<f32>(model_pos * scale + instance_pos, 1.0);

    // pass color straight through to fragment stage
    out.color = instance_color;

    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // fragment shader just outputs interpolated color
    // no lighting model, no shading, purely density-based color
    return in.color;
}
