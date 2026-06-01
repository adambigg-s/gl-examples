struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) nor: vec3<f32>,
    @location(2) tex: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) nor: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct CameraMatrix {
    view_proj: mat4x4<f32>,
};

struct ModelMatrix {
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraMatrix;
@group(1) @binding(0)
var<uniform> model: ModelMatrix;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = camera.view_proj * model.model * vec4<f32>(in.pos, 1.0);
    out.nor = in.nor;
    out.uv = in.tex;
    return out;
}

@group(2) @binding(0)
var tex: texture_2d<f32>;
@group(2) @binding(1)
var sam: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, sam, in.uv);
}
