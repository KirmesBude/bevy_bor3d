use bevy::prelude::ReflectComponent;
use bevy::prelude::ReflectResource;
use bevy::{
    app::{Plugin, Update},
    asset::{Asset, AssetId, AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        query::{Added, Changed},
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, Res, ResMut},
    },
    image::Image,
    math::{Vec2, Vec3, primitives::Plane3d},
    pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin, MeshMaterial3d},
    reflect::Reflect,
    render::{
        alpha::AlphaMode,
        mesh::{Mesh, Mesh3d, MeshVertexBufferLayoutRef, Meshable},
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

        app.init_resource::<ArrayTextureQueue>();

        app.register_type::<Sprite3d>();
        app.register_type::<BillboardMaterial>();
        app.register_type::<ArrayTextureQueue>();

        app.add_systems(Update, (queue_array_texture, process_array_texture).chain());
        app.add_systems(Update, add_mesh_and_material);
    }
}

#[derive(Asset, AsBindGroup, Debug, Clone, Reflect)]
#[bind_group_data(BillboardMaterialKey)]
pub struct BillboardMaterial {
    #[texture(0, dimension = "2d_array")]
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

/// Add this to an entity
#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Sprite3d {
    pub image: Handle<Image>,
    pub layers: u32,
}

// On Sprite3d insertion
#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Resource)]
struct ArrayTextureQueue {
    queue: Vec<(AssetId<Image>, u32)>,
}

// TODO: Consider an observer instead
// This reinserts stuff if it was already processsed -> fix
fn queue_array_texture(
    mut queue: ResMut<ArrayTextureQueue>,
    sprite3d: Query<&Sprite3d, Added<Sprite3d>>,
) {
    for sprite3d in &sprite3d {
        queue.queue.push((sprite3d.image.id(), sprite3d.layers));
    }
}

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

fn add_mesh_and_material(
    mut commands: Commands,
    sprite3d: Query<(Entity, &Sprite3d), Changed<Sprite3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BillboardMaterial>>,
) {
    for (entity, sprite3d) in &sprite3d {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::new(25.0, 25.0)).mesh())),
            MeshMaterial3d(materials.add(BillboardMaterial {
                image: sprite3d.image.clone(),
            })),
        ));
    }
}
