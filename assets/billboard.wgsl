#import bevy_pbr::{
    mesh_view_bindings::view,
    mesh_functions::get_world_from_local
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let clip_from_world = view.clip_from_world;
    let camera_right = normalize(vec3<f32>(clip_from_world[0].x, clip_from_world[1].x, clip_from_world[2].x));
    let camera_up = normalize(vec3<f32>(clip_from_world[0].y, clip_from_world[1].y, clip_from_world[2].y));

    let world_space = camera_right * vertex.position.x + camera_up * vertex.position.y;
    let position = view.clip_from_world * get_world_from_local(vertex.instance_index) * vec4<f32>(world_space, 1.);

    out.uv = vertex.uv;
    out.clip_position = position;

    return out;
}

struct FragmentInput {
     @location(0) uv: vec2<f32>
};

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var color = textureSample(texture, texture_sampler, in.uv);

    return color;
}