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

    let world_from_local = affine3_to_square(sprite3d.world_from_local);
    let world_position = world_from_local * local_position;
    let clip_from_world = view.clip_from_world;
    out.clip_position = clip_from_world * world_position;
    
    // Subtract position from (1,1) to flip ?????
    // TODO: Now it is flipped on the x axis
    out.uv = vec2<f32>(1.0, 1.0) - vec2<f32>(vertex_position.xy);
    
    return out;
}

@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(texture, texture_sampler, in.uv);
    
    return color;
}
