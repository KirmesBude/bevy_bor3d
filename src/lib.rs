use bevy::{
    app::Plugin, asset::Asset, pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin, StandardMaterial}, reflect::Reflect, render::render_resource::{AsBindGroup, ShaderRef}
};

const BILLBOARD_SHADER_ASSET_PATH: &str = "billboard.wgsl";

pub struct BillboardPlugin;

impl Plugin for BillboardPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, BillboardExtension>,>::default());
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct BillboardExtension {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[uniform(100)]
    pub quantize_steps: u32,
}

impl MaterialExtension for BillboardExtension {
    fn fragment_shader() -> ShaderRef {
        BILLBOARD_SHADER_ASSET_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        BILLBOARD_SHADER_ASSET_PATH.into()
    }
}
