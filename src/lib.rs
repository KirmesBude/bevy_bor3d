use bevy::{
    app::{App, Plugin},
    asset::Handle,
    ecs::{component::Component, reflect::ReflectComponent},
    image::Image,
    reflect::{Reflect, std_traits::ReflectDefault},
    render::{
        sync_world::SyncToRenderWorld,
        view::{self, Visibility, VisibilityClass},
    },
    transform::components::Transform,
};

use crate::render::Sprite3dRenderPlugin;

mod render;

#[derive(Debug, Default)]
pub struct Sprite3dPlugin;

impl Plugin for Sprite3dPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Sprite3d>();

        app.add_plugins(Sprite3dRenderPlugin);
    }
}

#[derive(Component, Debug, Default, Clone, Reflect)]
#[require(Transform, Visibility, SyncToRenderWorld, VisibilityClass)]
#[reflect(Component, Default, Debug, Clone)]
#[component(on_add = view::add_visibility_class::<Sprite3d>)]
pub struct Sprite3d {
    pub image: Handle<Image>,
}
