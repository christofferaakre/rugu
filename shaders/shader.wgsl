struct VertexOutput {
    @builtin(position) position: vec4<f32>
}

@vertex
fn vs_main() -> VertexOutput {
    var vertex_output: VertexOutput;
    return vertex_output;
}

@fragment
fn fs_main() {
}
