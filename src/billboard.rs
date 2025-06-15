/// This module is supposed to provide component and systems that allow you to easily add camera facing aka a billboard
/// functionality to Sprite3d (or Text3d?)
// TODO: Fix imports
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::sprite::Sprite3d;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Billboard>();
    app.register_type::<ArrayTextureQueue>();

    app.init_resource::<ArrayTextureQueue>();

    app.add_systems(Update, process_array_texture);
}

/// Adds billboard capabilities to Sprite3d
/// Will always face camera
#[derive(Component, Debug, Default, Clone, Reflect)]
#[require(Transform, Visibility)] // TODO: What do I actually need here?
#[reflect(Component, Default, Debug, Clone)]
#[component(on_add = on_billboard_added)]
pub struct Billboard {
    /// (optional) Axis around which the billboard shall rotate
    pub axis: Option<Vec3>,
    /// (optional) texture_array layer size
    pub layers: Option<u32>,
}

// On SpritBillboarde3d insertion
#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
struct ArrayTextureQueue {
    queue: Vec<(AssetId<Image>, u32)>,
}

// TODO: This probably needs to happen at other times as well
fn on_billboard_added(mut world: DeferredWorld, context: HookContext) {
    // We only need to do anything if layers is set
    let billboard = world.get::<Billboard>(context.entity).unwrap();
    let Some(layers) = billboard.layers else {
        return;
    };

    // Retrieve Sprite3d if available
    let Some(sprite3d) = world.get::<Sprite3d>(context.entity) else {
        return;
    };

    // Get load state from asset server
    let image = sprite3d.image.id();
    let asset_server = world.get_resource::<AssetServer>().unwrap();
    if !asset_server.load_state(image).is_loaded() {
        // Not loaded: Save in Queue to be done later
        let mut queue = world.resource_mut::<ArrayTextureQueue>();
        queue.queue.push((image, layers));
    } else {
        // Loaded: reinterpret right away
        let mut images = world.resource_mut::<Assets<Image>>();
        let image = images.get_mut(image).unwrap();
        image.reinterpret_stacked_2d_as_array(layers);
    }
}

// TODO: This can be nicer
// For every image we need to reinterpret it as a array texture
fn process_array_texture(
    mut queue: ResMut<ArrayTextureQueue>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let mut remaining_queue = vec![];

    queue.queue.iter().for_each(|(image, layers)| {
        if asset_server.load_state(*image).is_loaded() {
            let image = images.get_mut(*image).unwrap();
            image.reinterpret_stacked_2d_as_array(*layers);
        } else {
            remaining_queue.push((*image, *layers));
        }
    });

    queue.queue = remaining_queue;
}
