//! Demonstrating per view billboard 3d sprites
//!

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_bor3d::{Sprite3d, Sprite3dPlugin};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Sprite3dPlugin)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cube_materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Spawn the camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, -100.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Reference cube
    commands.spawn((
        Name::new("Cube1"),
        Mesh3d(meshes.add(Cuboid::new(25.0, 25.0, 25.0))),
        MeshMaterial3d(cube_materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.0, 4.0),
    ));

    // Reference cube
    commands.spawn((
        Name::new("Cube2"),
        Mesh3d(meshes.add(Cuboid::new(25.0, 25.0, 25.0))),
        MeshMaterial3d(cube_materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.0, -4.0),
    ));

    commands.spawn(
Sprite3d {
            image: asset_server.load_with_settings(
                "sprites/bossa1.png",
                |s: &mut ImageLoaderSettings| {
                    s.sampler = ImageSampler::nearest(); // TODO: Without this there is a weird "glow/border"
                },
            ),
        }
    );
}