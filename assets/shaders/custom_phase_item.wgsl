// `custom_phase_item.wgsl`
//
// This shader goes with the `custom_phase_item` example. It demonstrates how to
// enqueue custom rendering logic in a `RenderPhase`.
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

// The GPU-side vertex structure.
struct Vertex {
    @builtin(vertex_index) index: u32,
};

// Information passed from the vertex shader to the fragment shader.
struct VertexOutput {
    // The clip-space position of the vertex.
    @builtin(position) clip_position: vec4<f32>,
    // The color of the vertex.
    @location(0) uv: vec2<f32>,
};

// The vertex shader entry point.
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var vertex_output: VertexOutput;

    let vertex_position = vec3<f32>(
        f32(vertex.index & 0x1u),
        f32((vertex.index & 0x2u) >> 1u),
        0.0
    );

    // Subtract by 0.5 to bring [0,1] into [-1,1] clip space
    vertex_output.clip_position = vec4(vertex_position.xyz - vec3<f32>(0.5, 0.5, 0.0), 1.0);
    // Subtract position from (1,1) to flip ?????
    vertex_output.uv = vec2<f32>(1.0, 1.0) - vec2<f32>(vertex_position.xy);
    return vertex_output;
}

@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

// The fragment shader entry point.
@fragment
fn fragment(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(texture, texture_sampler, vertex_output.uv);
    return color;
}
