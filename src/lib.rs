use bevy::{
    app::{App, Plugin},
    asset::Handle,
    prelude::{Component, Image},
};
use render::BillboardRenderPlugin;

mod render;

#[derive(Debug, Default)]
pub struct BillboardPlugin;

impl Plugin for BillboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BillboardRenderPlugin);
    }
}

#[derive(Component)]
pub struct Billboard {
    pub texture: Handle<Image>, /* TODO: Could be just added like this, but I want Vec in the future */
}
