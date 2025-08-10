//! Demonstrating per view billboard 3d sprites
//!

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_bor3d::{Billboard, Sprite3d, Sprite3dPlugin};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::camera_controller::{CameraController, CameraControllerPlugin};

#[path = "helpers/camera_controller.rs"]
mod camera_controller;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Sprite3dPlugin)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(CameraControllerPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn the camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 150.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraController::default(),
    ));

    let image =
        asset_server.load_with_settings("sprites/bossa1.png", |s: &mut ImageLoaderSettings| {
            s.sampler = ImageSampler::nearest(); // TODO: Without this there is a weird "glow/border"
        });

    commands.spawn((
        Sprite3d {
            image: image.clone(),
        },
        Transform::default(),
    ));

    commands.spawn((
        Sprite3d {
            image: image.clone(),
        },
        Billboard::Forward,
        Transform::from_translation(vec3(-60.0, 0.0, 0.0)),
    ));

    commands.spawn((
        Sprite3d {
            image: image.clone(),
        },
        Billboard::LookAt,
        Transform::from_translation(vec3(60.0, 0.0, 0.0)),
    ));
}
