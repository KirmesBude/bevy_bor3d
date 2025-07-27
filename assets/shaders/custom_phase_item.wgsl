// `custom_phase_item.wgsl`
//
// This shader goes with the `custom_phase_item` example. It demonstrates how to
// enqueue custom rendering logic in a `RenderPhase`.

// The GPU-side vertex structure.
struct Vertex {
    // The world-space position of the vertex.
    @location(0) position: vec3<f32>,
    // The color of the vertex.
    @location(1) color: vec3<f32>,
};

// Information passed from the vertex shader to the fragment shader.
struct VertexOutput {
    // The clip-space position of the vertex.
    @builtin(position) clip_position: vec4<f32>,
    // The color of the vertex.
    @location(0) color: vec3<f32>,
};

// The vertex shader entry point.
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    // Use an orthographic projection.
    var vertex_output: VertexOutput;
    vertex_output.clip_position = vec4(vertex.position.xyz, 1.0);
    vertex_output.color = vertex.color;
    return vertex_output;
}

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

// The fragment shader entry point.
@fragment
fn fragment(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(texture, texture_sampler, vec2<f32>(0.0,0.0));
    return color;
}
