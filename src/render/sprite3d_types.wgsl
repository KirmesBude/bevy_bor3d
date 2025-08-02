#define_import_path bevy_bor3d::sprite3d_types

struct Sprite3d {
    world_from_local: mat3x4<f32>,
    size: vec3<f32>,
};