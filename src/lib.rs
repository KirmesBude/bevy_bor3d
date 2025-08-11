use bevy::{
    app::{App, Plugin, Update},
    asset::{AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        reflect::ReflectComponent,
        system::{Query, Res, ResMut},
    },
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

        app.add_systems(Update, update_sprit3d_array_layers);
    }
}

#[derive(Component, Debug, Default, Clone, Reflect)]
#[require(Transform, Visibility, SyncToRenderWorld, VisibilityClass)]
#[reflect(Component, Default, Debug, Clone)]
#[component(on_add = view::add_visibility_class::<Sprite3d>)]
pub struct Sprite3d {
    pub image: Handle<Image>,
    pub layers: u32,
}

#[derive(Component, Debug, Default, Clone, Reflect)]
pub enum Billboard {
    #[default]
    Forward, // Uses negative view forward
    LookAt, // Always looks at camera
}

// TODO: This probably needs to happen at other times as well
fn update_sprit3d_array_layers(
    sprite3ds: Query<&Sprite3d>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    for sprite3d in &sprite3ds {
        // Get load state from asset server
        let image = sprite3d.image.id();
        let layers = if sprite3d.layers == 0 {
            1
        } else {
            sprite3d.layers
        };

        if asset_server.load_state(image).is_loaded() {
            // Loaded: reinterpret right away
            let image = images.get_mut(image).unwrap();
            if image.texture_descriptor.array_layer_count() == 1
                && image.texture_descriptor.array_layer_count() != layers
            {
                image.reinterpret_stacked_2d_as_array(layers);
            }
        }
    }
}
