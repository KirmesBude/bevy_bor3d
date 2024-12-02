//! Demonstrating per view billboard 3d sprites
//!

use std::f32::consts::PI;

use bevy::{
    pbr::{CascadeShadowConfigBuilder, ExtendedMaterial},
    prelude::*,
    render::camera::Viewport,
    window::WindowResized,
};
use bevy_bor3d::MyExtension;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, MyExtension>,
        >::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (set_camera_viewports, spin))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut extended_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Billboard 3d sprite
    // TODO
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(75.0, 75.0, 75.0).mesh())),
        MeshMaterial3d(extended_materials.add(ExtendedMaterial {
            base: Color::srgb(0.3, 0.5, 0.3).into(),
            extension: MyExtension { lol: 0.0 },
        })),
        Transform::from_translation(Vec3::new(-65.0, 0.0, 0.0)),
        Spinning::default(),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(75.0, 75.0, 75.0).mesh())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.3))),
        Transform::from_translation(Vec3::new(65.0, 0.0, 0.0)),
        Spinning::default(),
    ));

    // Light
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 2, // webgl supports only 1
            first_cascade_far_bound: 200.0,
            maximum_distance: 280.0,
            ..default()
        }
        .build(),
    ));

    // Cameras and their dedicated UI
    for (index, camera_pos) in [
        Vec3::new(0.0, 200.0, -150.0),
        Vec3::new(150.0, 150., 50.0),
        Vec3::new(100.0, 150., -150.0),
        Vec3::new(-100.0, 80., 150.0),
    ]
    .iter()
    .enumerate()
    {
        commands.spawn((
            Camera3d::default(),
            Transform::from_translation(*camera_pos).looking_at(Vec3::ZERO, Vec3::Y),
            Camera {
                // Renders cameras with different priorities to prevent ambiguities
                order: index as isize,
                ..default()
            },
            CameraPosition {
                pos: UVec2::new((index % 2) as u32, (index / 2) as u32),
            },
        ));
    }
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
