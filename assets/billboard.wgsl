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
    var out: VertexOutput;

    // Get rid of view rotation
    let clip_from_world = view.clip_from_world;
    let camera_right = normalize(vec3<f32>(clip_from_world[0].x, clip_from_world[1].x, clip_from_world[2].x));
    let camera_up = normalize(vec3<f32>(clip_from_world[0].y, clip_from_world[1].y, clip_from_world[2].y));

    let world_space = camera_right * vertex.position.x + camera_up * vertex.position.y;

    // Get rid of entity rotation
    var world_from_local = get_world_from_local(vertex.instance_index);
    world_from_local[0].x = 1.0;
    world_from_local[0].y = 0.0;
    world_from_local[0].z = 0.0;
    world_from_local[1].x = 0.0;
    world_from_local[1].y = 1.0;
    world_from_local[1].z = 0.0;
    world_from_local[2].x = 0.0;
    world_from_local[2].y = 0.0;
    world_from_local[2].z = 1.0;
    let position = view.clip_from_world * world_from_local * vec4<f32>(world_space, 1.);

    out.uv = vertex.uv;
    out.clip_position = position;

    return out;
}

struct FragmentInput {
     @location(0) uv: vec2<f32>,
};

@group(2) @binding(0) var texture: texture_2d_array<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> forward: vec3<f32>;

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    // TODO: I am pretty sure this is wrong
    let world_from_view = view.world_from_view;
    let cam_position = vec3<f32>(world_from_view[3].xyz);
    let angle = dot(normalize(cam_position), forward);
    
    let layer = acos(angle) / radians(360.0) * 7.0;
    
    var color = textureSample(texture, texture_sampler, in.uv, u32(layer));

    return color;
}