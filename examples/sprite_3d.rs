use bevy::prelude::*;
use bevy_bor3d::BillboardPlugin;
use bevy_flycam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(BillboardPlugin)
        .run();
}
