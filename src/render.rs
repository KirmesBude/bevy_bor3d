/// Provides common render stuff for sprite3d and billboarding
///
// TODO: Fix imports
use bevy::{
    core_pipeline::core_3d::{CORE_3D_DEPTH_FORMAT, Transparent3d},
    ecs::{
        query::ROQueryItem,
        system::{SystemParamItem, lifetimeless::SRes},
    },
    pbr::{MaterialPipeline, MaterialPipelineKey, RenderMeshInstances},
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{
            MeshVertexBufferLayoutRef, VertexBufferLayout, VertexFormat, allocator::MeshAllocator,
        },
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            AsBindGroup, BufferUsages, ColorTargetState, ColorWrites, CompareFunction,
            DepthStencilState, FragmentState, IndexFormat, MultisampleState, PipelineCache,
            PrimitiveState, RawBufferVec, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, SpecializedRenderPipeline, SpecializedRenderPipelines,
            TextureFormat, VertexAttribute, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        view::{self, ExtractedView, RenderVisibleEntities, VisibilityClass},
    },
};
use bytemuck::{Pod, Zeroable};

use crate::billboard::Billboard;

// TODO: Use load_shader_library?
const BILLBOARD_SHADER_ASSET_PATH: &str = "billboard.wgsl";

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<BillboardMaterial>::default());

    app.register_type::<BillboardMaterial>();

    app.add_systems(Update, extract_forward);

    // Custom phase item
    app.add_plugins(ExtractComponentPlugin::<BillboardPhaseItem>::default());

    app.get_sub_app_mut(RenderApp)
        .unwrap()
        .init_resource::<BillboardPhasePipeline>()
        .init_resource::<SpecializedRenderPipelines<BillboardPhasePipeline>>()
        .add_render_command::<Transparent3d, DrawBillboardPhaseItemCommands>()
        .add_systems(
            Render,
            prepare_billboard_phase_item_buffers.in_set(RenderSet::Prepare),
        )
        .add_systems(Render, queue_billboard_phase_item.in_set(RenderSet::Queue));
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

#[derive(Clone, Component, ExtractComponent)]
#[require(VisibilityClass)]
#[component(on_add = view::add_visibility_class::<BillboardPhaseItem>)]
pub struct BillboardPhaseItem;

#[derive(Resource)]
struct BillboardPhasePipeline {
    shader: Handle<Shader>,
}

impl FromWorld for BillboardPhasePipeline {
    fn from_world(world: &mut World) -> Self {
        // Load and compile the shader in the background.
        let asset_server = world.resource::<AssetServer>();

        BillboardPhasePipeline {
            shader: asset_server.load("billboard.wgsl"),
        }
    }
}

struct DrawBillboardPhaseItem;

impl<P> RenderCommand<P> for DrawBillboardPhaseItem
where
    P: PhaseItem,
{
    type Param = SRes<BillboardPhaseItemBuffers>;

    type ViewQuery = ();

    type ItemQuery = ();

    fn render<'w>(
        _: &P,
        _: ROQueryItem<'w, Self::ViewQuery>,
        _: Option<ROQueryItem<'w, Self::ItemQuery>>,
        billboard_phase_item_buffers: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // Borrow check workaround.
        let billboard_phase_item_buffers = billboard_phase_item_buffers.into_inner();

        // Tell the GPU where the vertices are.
        pass.set_vertex_buffer(
            0,
            billboard_phase_item_buffers
                .vertices
                .buffer()
                .unwrap()
                .slice(..),
        );

        // Tell the GPU where the indices are.
        pass.set_index_buffer(
            billboard_phase_item_buffers
                .indices
                .buffer()
                .unwrap()
                .slice(..),
            0,
            IndexFormat::Uint32,
        );

        // TODO: Quad
        // Draw one quad (3 vertices).
        pass.draw_indexed(0..3, 0, 0..1);

        RenderCommandResult::Success
    }
}

#[derive(Resource)]
struct BillboardPhaseItemBuffers {
    /// The vertices for the single triangle.
    ///
    /// This is a [`RawBufferVec`] because that's the simplest and fastest type
    /// of GPU buffer, and [`Vertex`] objects are simple.
    vertices: RawBufferVec<Vertex>,

    /// The indices of the single triangle.
    ///
    /// As above, this is a [`RawBufferVec`] because `u32` values have trivial
    /// size and alignment.
    indices: RawBufferVec<u32>,
}

impl FromWorld for BillboardPhaseItemBuffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        // Create the vertex and index buffers.
        let mut vbo = RawBufferVec::new(BufferUsages::VERTEX);
        let mut ibo = RawBufferVec::new(BufferUsages::INDEX);

        for vertex in &VERTICES {
            vbo.push(*vertex);
        }
        for index in 0..3 {
            ibo.push(index);
        }

        // These two lines are required in order to trigger the upload to GPU.
        vbo.write_buffer(render_device, render_queue);
        ibo.write_buffer(render_device, render_queue);

        BillboardPhaseItemBuffers {
            vertices: vbo,
            indices: ibo,
        }
    }
}

/// The CPU-side structure that describes a single vertex of the triangle.
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    /// The 3D position of the triangle vertex.
    position: Vec3,
    /// Padding.
    pad0: u32,
    /// The color of the triangle vertex.
    color: Vec3,
    /// Padding.
    pad1: u32,
}

impl Vertex {
    /// Creates a new vertex structure.
    const fn new(position: Vec3, color: Vec3) -> Vertex {
        Vertex {
            position,
            color,
            pad0: 0,
            pad1: 0,
        }
    }
}

type DrawBillboardPhaseItemCommands = (SetItemPipeline, DrawBillboardPhaseItem);

// TODO: Quad
static VERTICES: [Vertex; 3] = [
    Vertex::new(vec3(-0.866, -0.5, 0.5), vec3(1.0, 0.0, 0.0)),
    Vertex::new(vec3(0.866, -0.5, 0.5), vec3(0.0, 1.0, 0.0)),
    Vertex::new(vec3(0.0, 1.0, 0.5), vec3(0.0, 0.0, 1.0)),
];

fn prepare_billboard_phase_item_buffers(mut commands: Commands) {
    commands.init_resource::<BillboardPhaseItemBuffers>();
}

fn queue_billboard_phase_item(
    pipeline_cache: Res<PipelineCache>,
    billboard_phase_pipeline: Res<BillboardPhasePipeline>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    mut specialized_render_pipelines: ResMut<SpecializedRenderPipelines<BillboardPhasePipeline>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    render_mesh_instances: Res<RenderMeshInstances>,
    mesh_allocator: Res<MeshAllocator>,
) {
    let draw_custom_phase_item = transparent_draw_functions
        .read()
        .id::<DrawBillboardPhaseItemCommands>();

    // Render phases are per-view, so we need to iterate over all views so that
    // the entity appears in them. (In this example, we have only one view, but
    // it's good practice to loop over all views anyway.)
    for (view, view_visible_entities, msaa) in views.iter() {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let rangefinder = view.rangefinder3d();

        // Find all the custom rendered entities that are visible from this
        // view.
        for (render_entity, visible_entity) in
            view_visible_entities.get::<BillboardPhaseItem>().iter()
        {
            // Ordinarily, the [`SpecializedRenderPipeline::Key`] would contain
            // some per-view settings, such as whether the view is HDR, but for
            // simplicity's sake we simply hard-code the view's characteristics,
            // with the exception of number of MSAA samples.
            let pipeline_id = specialized_render_pipelines.specialize(
                &pipeline_cache,
                &billboard_phase_pipeline,
                *msaa,
            );

            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(*visible_entity)
            else {
                continue;
            };

            // TODO: Add the custom render item
            let distance = rangefinder.distance_translation(&mesh_instance.translation);
            let (_vertex_slab, index_slab) =
                mesh_allocator.mesh_slabs(&mesh_instance.mesh_asset_id);
            transparent_phase.add(Transparent3d {
                distance,
                pipeline: pipeline_id,
                entity: (*render_entity, *visible_entity),
                draw_function: draw_custom_phase_item,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: index_slab.is_some(),
            });
        }
    }
}

impl SpecializedRenderPipeline for BillboardPhasePipeline {
    type Key = Msaa;

    fn specialize(&self, msaa: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("custom render pipeline".into()),
            layout: vec![],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    // This needs to match the layout of [`Vertex`].
                    attributes: vec![
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 16,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    // Ordinarily, you'd want to check whether the view has the
                    // HDR format and substitute the appropriate texture format
                    // here, but we omit that for simplicity.
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            // Note that if your view has no depth buffer this will need to be
            // changed.
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: default(),
                bias: default(),
            }),
            multisample: MultisampleState {
                count: msaa.samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}
