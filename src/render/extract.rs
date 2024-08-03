use bevy::{
    asset::AssetId,
    prelude::{Commands, Component, Entity, GlobalTransform, Image, Query},
    render::Extract,
};

use crate::Billboard;

/* TODO: Do I have to despawn the entities with this approach? Alternative is resource */
#[derive(Component)]
pub struct ExtractedBillboard {
    transform: GlobalTransform,
    texture: AssetId<Image>,
    /* TODO: visiblity, color, rect(atlas), custom_size, frustum */
}

/* TODO: Do I need this removed stuff? */
pub fn extract_billboard(
    mut commands: Commands,
    billboard_q: Extract<Query<(Entity, &Billboard, &GlobalTransform)>>,
) {
    let extracted_billboards: Vec<_> = billboard_q
        .iter()
        .map(|(entity, billboard, global_transform)| {
            (
                entity,
                ExtractedBillboard {
                    transform: *global_transform,
                    texture: billboard.texture.id(),
                },
            )
        })
        .collect();

    commands.insert_or_spawn_batch(extracted_billboards);
}
