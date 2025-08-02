// `custom_phase_item.wgsl`
//
// This shader goes with the `custom_phase_item` example. It demonstrates how to
// enqueue custom rendering logic in a `RenderPhase`.
#import bevy_render::view::View
#import bevy_render::maths::affine3_to_square

struct Billboard {
    world_from_local: mat3x4<f32>,
    size: vec3<f32>,
};

@group(0) @binding(0) var<uniform> view: View;
@group(2) @binding(0) var<uniform> billboard: Billboard;

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
    var pre_local_position = vertex_position.xyz - vec3<f32>(0.5, 0.5, 0.0);
    // Scale with image size, z is 0 anyways
    pre_local_position = pre_local_position * billboard.size;
    // Extend by 1.0 for vec4
    let local_position = vec4<f32>(pre_local_position, 1.0);

    let world_from_local = affine3_to_square(billboard.world_from_local);
    let world_position = world_from_local * local_position;
    let clip_from_world = view.clip_from_world;
    vertex_output.clip_position = clip_from_world * world_position;

    // Subtract position from (1,1) to flip ?????
    // TODO: Now it is flipped on the x axis
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
