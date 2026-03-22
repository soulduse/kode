struct Uniforms {
    screen_size: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    // Unit quad: 0,0 -> 1,0 -> 0,1 -> 1,1 (triangle strip)
    let x = select(0.0, 1.0, vi == 1u || vi == 3u);
    let y = select(0.0, 1.0, vi == 2u || vi == 3u);

    let pixel_pos = pos + vec2(x, y) * size;
    let ndc = vec2(
        pixel_pos.x / uniforms.screen_size.x * 2.0 - 1.0,
        1.0 - pixel_pos.y / uniforms.screen_size.y * 2.0,
    );

    var out: VertexOutput;
    out.position = vec4(ndc, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
