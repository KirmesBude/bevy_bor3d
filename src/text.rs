/// This module is supposed to provide component and systems that allow you to easily spawn text in 3d
// TODO: Fix imports
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Text3d>();
}

/// Describes a text to be rendered to a 3D camera
#[derive(Component, Debug, Default, Clone, Reflect)]
#[require(Transform, Visibility)] // TODO: Add back the other ones? + TODO: Add Mesh as default quad?
#[reflect(Component, Default, Debug, Clone)] // TODO: Add on_add hook to insert Mesh and Materials
pub struct Text3d {
    /// TODO: Text3d
    text: String,
}
