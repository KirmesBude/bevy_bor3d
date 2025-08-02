//! Demonstrating per view billboard 3d sprites
//!

use std::f32::consts::PI;

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
    render::{camera::Viewport, primitives::Aabb},
    window::WindowResized,
};
use bevy_bor3d::{Sprite3d, Sprite3dPlugin};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use ops::{cos, sin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Sprite3dPlugin)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup)
        .add_systems(Update, (set_camera_viewports, spin, orbit, shuffle))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cube_materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Visibility::default(),
        Transform::from_translation(vec3(0.5, 0.0, 0.0)),
        // This `Aabb` is necessary for the visibility checks to work.
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::splat(0.5),
        },
        Sprite3d {
            image: asset_server.load_with_settings(
                "sprites/bossa1.png",
                |s: &mut ImageLoaderSettings| {
                    s.sampler = ImageSampler::nearest();
                },
            ),
        },
    ));

    // Spawn the camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Component)]
struct CameraPosition {
    pos: UVec2,
}

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<(&CameraPosition, &mut Camera)>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        let size = window.physical_size() / 2;

        for (camera_position, mut camera) in &mut query {
            camera.viewport = Some(Viewport {
                physical_position: camera_position.pos * size,
                physical_size: size,
                ..default()
            });
        }
    }
}

#[derive(Debug, Default, Component)]
struct Spinning;

fn spin(mut transforms: Query<&mut Transform, With<Spinning>>, time: Res<Time>) {
    for mut transform in &mut transforms {
        transform.rotate_y(time.delta_secs() * 2.0 * PI / 20.0);
    }
}

#[derive(Debug, Default, Component)]
struct Orbiting;

fn orbit(mut transforms: Query<&mut Transform, With<Orbiting>>, time: Res<Time>) {
    for mut transform in &mut transforms {
        let angle = time.delta_secs() * 2.0 * PI / 100.0 * 10.0;
        let x = transform.translation.x * cos(angle) + transform.translation.y * sin(angle);
        let y = -transform.translation.x * sin(angle) + transform.translation.y * cos(angle);

        transform.translation.x = x;
        transform.translation.y = y;
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

#[derive(Debug, Component)]
struct Shuffling {
    offset: f32,
    right: bool,
}

impl Default for Shuffling {
    fn default() -> Self {
        Self {
            offset: 32.0,
            right: true,
        }
    }
}

fn shuffle(mut query: Query<(&mut Transform, &mut Shuffling)>, time: Res<Time>) {
    for (mut transform, mut shuffling) in &mut query {
        let mut offset = time.delta_secs() * 4.0;
        let right = shuffling.right;

        if offset < shuffling.offset {
            shuffling.offset -= offset;
        } else {
            offset = shuffling.offset;
            *shuffling = Shuffling {
                right: !right,
                ..default()
            }
        }

        if !right {
            offset = -offset;
        }

        transform.translation.x += offset;
    }
}
