#import bevy_pbr::{
    mesh_view_bindings::view,
    mesh_view_bindings::globals,
    mesh_functions::{get_world_from_local, mesh_normal_local_to_world},
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    // Use an orthographic projection.
    var vertex_output: VertexOutput;
    vertex_output.clip_position = vec4(vertex.position.xyz, 1.0);
    vertex_output.uv = vertex.uv;

    return vertex_output;
}

struct FragmentInput {
     @location(0) uv: vec2<f32>,
};

@group(2) @binding(0) var texture: texture_2d_array<f32>;
@group(2) @binding(1) var texture_sampler: sampler;

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    return vec4(vec3<f32>(0.5, 0.8, 0.2), 1.0);
}