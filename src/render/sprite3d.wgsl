#import bevy_render::{
    maths::affine3_to_square,
    view::View,
}

#import bevy_bor3d::{
    sprite3d_view_bindings::view,
    sprite3d_bindings::sprite3d,
}

struct VertexInput {
    @builtin(vertex_index) index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let vertex_position = vec3<f32>(
        f32(in.index & 0x1u),
        f32((in.index & 0x2u) >> 1u),
        0.0
    );

    // Subtract by 0.5 to bring [0,1] into [-1,1] clip space
    var pre_local_position = vertex_position.xyz - vec3<f32>(0.5, 0.5, 0.0);
    // Scale with image size, z is 0 anyways
    pre_local_position = pre_local_position * sprite3d.size;
    // Extend by 1.0 for vec4
    let local_position = vec4<f32>(pre_local_position, 1.0);

    var world_from_local = affine3_to_square(sprite3d.world_from_local);
    var view_from_world = view.view_from_world;
#if BILLBOARD == 1 // Forward
    // Remove entity rotation
    world_from_local = mat4x4<f32>(
        vec4<f32>(length(world_from_local[0]), 0.0, 0.0, 0.0),
        vec4<f32>(0.0, length(world_from_local[1]), 0.0, 0.0),
        vec4<f32>(0.0, 0.0, length(world_from_local[2]), 0.0),
        world_from_local[3],
    );

    // Remove view rotation
    view_from_world = mat4x4<f32>(
        vec4<f32>(length(view_from_world[0]), 0.0, 0.0, 0.0),
        vec4<f32>(0.0, length(view_from_world[1]), 0.0, 0.0),
        vec4<f32>(0.0, 0.0, length(view_from_world[2]), 0.0),
        view_from_world[3],
    );
#else if BILLBOARD == 2 // LookAt
    // Compute rotation based on view and 
    let rotation = mat3x3<f32>(normalize(view_from_world[0].xyz), normalize(view_from_world[1].xyz), normalize(view_from_world[2].xyz));
    let inverse = transpose(rotation);

    world_from_local = mat4x4<f32>(
        vec4<f32>(length(world_from_local[0]) * inverse[0], 0.0),
        vec4<f32>(length(world_from_local[1]) * inverse[1], 0.0),
        vec4<f32>(length(world_from_local[2]) * inverse[2], 0.0),
        world_from_local[3],
    );
#endif

    let world_position = world_from_local * local_position;
    out.clip_position = view.clip_from_view * view_from_world * world_position;
    
    // UV correctly like this?
    out.uv = vec2<f32>(vertex_position.xy) * vec2<f32>(1.0, -1.0) + vec2<f32>(0.0, 1.0);
    
    return out;
}

@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(texture, texture_sampler, in.uv);
    
    return color;
}
