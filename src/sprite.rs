/// This module is supposed to provide component and systems that allow you to easily spawn a -z facing sprite
/// automatically taking care of spawning the Mesh and Material
/// TODO: Sized by image size
/// TODO: double faced (configurable)
/// TODO: TextureAtlas support
///
// TODO: Fix imports
use bevy::prelude::*;

use crate::on_add_3d;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Sprite3d>();
}

/// Describes a sprite to be rendered to a 3D camera
#[derive(Component, Debug, Default, Clone, Reflect)]
#[require(Transform, Visibility)] // TODO: Add back the other ones? + TODO: Add Mesh as default quad?
#[reflect(Component, Default, Debug, Clone)]
#[component(on_add = on_add_3d)]
pub struct Sprite3d {
    /// The image used to render the sprite
    pub image: Handle<Image>,
    // TODO: Maybe this should be a generic component?
    /// The (optional) texture atlas used to render the sprite
    pub texture_atlas: Option<TextureAtlas>,
    // TODO: consider adding other things back
}
