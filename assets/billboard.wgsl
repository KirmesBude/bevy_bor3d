#import bevy_pbr::{
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip},
    forward_io::{Vertex, VertexOutput},
}

struct MyExtendedMaterial {
    lol: f32,
}

@group(2) @binding(100)
var<uniform> my_extended_material: MyExtendedMaterial;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0),
    );
    return out;
}
