struct InstanceInput {
    // 4x4 model matrix split into 4 vec4s
    @location(1) model_mat0: vec4<f32>,
    @location(2) model_mat1: vec4<f32>,
    @location(3) model_mat2: vec4<f32>,
    @location(4) model_mat3: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> model: mat4x4<f32>;

@vertex
fn vs_main(vertex: VertexInput, instance_input: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>(vertex.position.xyz, 1.0);

    return out;
}

@fragment
fn fs_main(frag_in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 1.0, 1.0) ;
}
