struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) col: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct CameraUniform {
    vp: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = camera.vp * vec4<f32>(model.pos, 1.0);
    out.color = model.col;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}


