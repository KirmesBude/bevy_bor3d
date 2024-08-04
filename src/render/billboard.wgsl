#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

struct VertexInput {
    /* TODO: Necessary? */
    @builtin(vertex_index) index: u32,
    @location(0) position: vec4<f32>,
    @location(1) uv: vec4<f32>,
    @location(2) color: vec4<f32>,
}

/* TODO: This is entirely defined in the shader?*/
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) color: vec4<f32>,
};

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = view.view_proj * vec4<f32>(in_position, 1.0);
    out.uv = in.uv;
    out.color = in.color;

    return out;
}

/* TODO: This should be an array */
@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color * textureSample(texture, sampler, in.uv);

    return color;
}