//! Demonstrating per view billboard 3d sprites
//!

use std::f32::consts::PI;

use bevy::{
    pbr::CascadeShadowConfigBuilder, prelude::*, render::camera::Viewport, window::WindowResized,
};
use bevy_bor3d::MyMaterial;
use ops::{cos, sin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<MyMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                set_camera_viewports,
                spin,
                orbit,
                shuffle,
                create_array_texture,
            ),
        )
        .run();
}

#[derive(Resource)]
struct LoadingTexture {
    is_loaded: bool,
    handle: Handle<Image>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Billboard 3d sprite
    // TODO
    // Start loading the texture.
    commands.insert_resource(LoadingTexture {
        is_loaded: false,
        handle: asset_server.load("sprites/array.png"),
    });

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::new(25.0, 25.0)).mesh())),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            base_color_texture: Some(asset_server.load("sprites/bossa1.png")),
            ..Default::default()
        })),
        Transform::from_translation(Vec3::new(65.0, 0.0, 0.0)),
        Spinning::default(),
        Shuffling::default(),
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
            Orbiting,
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

#[derive(Debug, Default, Component)]
struct Orbiting;

fn orbit(mut transforms: Query<&mut Transform, With<Orbiting>>, time: Res<Time>) {
    for mut transform in &mut transforms {
        let angle = time.delta_secs() * 2.0 * PI / 100.0;
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

fn create_array_texture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_texture: ResMut<LoadingTexture>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MyMaterial>>,
) {
    if loading_texture.is_loaded
        || !asset_server
            .load_state(loading_texture.handle.id())
            .is_loaded()
    {
        return;
    }
    loading_texture.is_loaded = true;
    let image = images.get_mut(&loading_texture.handle).unwrap();

    // Create a new array texture asset from the loaded texture.
    let array_layers = 8;
    image.reinterpret_stacked_2d_as_array(array_layers);

    // Spawn some cubes using the array texture
    let material_handle = materials.add(MyMaterial {
        array_texture: loading_texture.handle.clone(),
        layer: 0,
    });

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::new(25.0, 25.0)).mesh())),
        MeshMaterial3d(material_handle.clone()),
        Transform::from_translation(Vec3::new(-65.0, 0.0, 0.0)),
        Spinning::default(),
        Shuffling::default(),
    ));
}
