use bevy::prelude::*;
use bevy_bor3d::{Billboard, BillboardPlugin};
use bevy_flycam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(BillboardPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    /* Camera is already spawned by PlayerPlugin */

    commands.spawn((
        Billboard {
            texture: asset_server.load("generic-rpg-vendor.png"),
        },
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
            ..default()
        },
    ));
}
