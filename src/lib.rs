use bevy::{
    app::Plugin,
    asset::{Asset, Handle},
    image::Image,
    pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin},
    reflect::Reflect,
    render::{
        alpha::AlphaMode,
        mesh::{Mesh, MeshVertexBufferLayoutRef},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

const BILLBOARD_SHADER_ASSET_PATH: &str = "billboard.wgsl";

pub struct BillboardPlugin;

impl Plugin for BillboardPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(MaterialPlugin::<BillboardMaterial>::default());
    }
}

#[derive(Asset, AsBindGroup, Debug, Clone, Reflect)]
#[bind_group_data(BillboardMaterialKey)]
pub struct BillboardMaterial {
    #[texture(0)]
    #[sampler(1)]
    #[dependency]
    pub image: Handle<Image>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct BillboardMaterialKey;

impl From<&BillboardMaterial> for BillboardMaterialKey {
    fn from(_material: &BillboardMaterial) -> Self {
        Self
    }
}

impl Material for BillboardMaterial {
    fn vertex_shader() -> ShaderRef {
        BILLBOARD_SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        BILLBOARD_SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
        ])?;

        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
