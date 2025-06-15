/// Provides common render stuff for sprite3d and billboarding
///
// TODO: Fix imports
use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

use crate::billboard::Billboard;

// TODO: Use load_shader_library?
const BILLBOARD_SHADER_ASSET_PATH: &str = "billboard.wgsl";

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<BillboardMaterial>::default());

    app.register_type::<BillboardMaterial>();

    app.add_systems(Update, extract_forward);
}

// TODO: Should this material hold everything?
// TODO: Can texture with 2d_array dimension also serve 1 layer images; What if those are not reinterpreted?
#[derive(Asset, AsBindGroup, Debug, Clone, Reflect)]
#[bind_group_data(BillboardMaterialKey)]
pub struct BillboardMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    #[dependency]
    image: Handle<Image>,
    #[uniform(2)]
    forward: Vec3,
}

impl From<Handle<Image>> for BillboardMaterial {
    fn from(image: Handle<Image>) -> Self {
        Self {
            image,
            forward: Vec3::Z,
        }
    }
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

// We need the entity forward vector in the fragment shader to decide which layer to sample
fn extract_forward(
    sprite3d: Query<
        (&GlobalTransform, &MeshMaterial3d<BillboardMaterial>),
        (With<Billboard>, Changed<GlobalTransform>),
    >,
    mut materials: ResMut<Assets<BillboardMaterial>>,
) {
    for (transform, material) in &sprite3d {
        let material = materials.get_mut(material).unwrap();
        material.forward = transform.forward().as_vec3();
    }
}
