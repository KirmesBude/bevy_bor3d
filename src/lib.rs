use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::{
    app::Plugin,
    asset::Assets,
    math::{Vec2, Vec3, primitives::Plane3d},
    pbr::MeshMaterial3d,
    render::mesh::{Mesh, Mesh3d, Meshable},
};

pub use crate::billboard::Billboard;
use crate::render::BillboardMaterial;
pub use crate::sprite::Sprite3d;
pub use crate::new::BillboardPhaseItem;

mod billboard;
mod new;
mod render;
mod sprite;
mod text;

// TODO: rename
pub struct BillboardPlugin;

impl Plugin for BillboardPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(sprite::plugin);
        app.add_plugins(billboard::plugin);
        app.add_plugins(text::plugin);
        app.add_plugins(render::plugin);
        app.add_plugins(new::NewBillboardPlugin);
    }
}

// TODO: This needs to be reworked for Text3d
fn on_add_3d(mut world: DeferredWorld, context: HookContext) {
    let mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(Plane3d::new(Vec3::Z, Vec2::new(25.0, 25.0)).mesh());

    let image = world.get::<Sprite3d>(context.entity).unwrap().image.clone();
    let material = world.resource_mut::<Assets<BillboardMaterial>>().add(image);

    world
        .commands()
        .entity(context.entity)
        .insert((Mesh3d(mesh), MeshMaterial3d(material)));
}
