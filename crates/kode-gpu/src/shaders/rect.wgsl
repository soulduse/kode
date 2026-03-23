struct Uniforms {
    screen_size: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) rect_size: vec2<f32>,
    @location(3) radius: f32,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) border_radius: f32,
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
    // Local position in pixel space relative to rect center
    out.local_pos = (vec2(x, y) - 0.5) * size;
    out.rect_size = size;
    out.radius = border_radius;
    return out;
}

// Signed distance function for a rounded box
fn rounded_box_sdf(p: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p) - half_size + vec2(radius);
    return length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if in.radius <= 0.0 {
        return in.color;
    }

    let half_size = in.rect_size * 0.5;
    let r = min(in.radius, min(half_size.x, half_size.y));
    let dist = rounded_box_sdf(in.local_pos, half_size, r);

    // Anti-aliased edge (1px smoothing)
    let alpha = 1.0 - smoothstep(-1.0, 0.5, dist);

    if alpha < 0.01 {
        discard;
    }

    return vec4(in.color.rgb, in.color.a * alpha);
}
