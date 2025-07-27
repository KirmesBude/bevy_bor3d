use bevy::{
    asset::{load_internal_asset, weak_handle},
    core_pipeline::core_3d::{CORE_3D_DEPTH_FORMAT, Transparent3d},
    ecs::{
        query::ROQueryItem,
        system::{SystemParamItem, lifetimeless::SRes},
    },
    pbr::RenderMeshInstances,
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{VertexBufferLayout, VertexFormat, allocator::MeshAllocator},
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            BindGroupLayout, BindGroupLayoutEntries, BlendState, BufferUsages, ColorTargetState,
            ColorWrites, CompareFunction, DepthStencilState, FragmentState, IndexFormat,
            MultisampleState, PipelineCache, PrimitiveState, RawBufferVec,
            RenderPipelineDescriptor, SamplerBindingType, ShaderStages, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, TextureSampleType, VertexState,
            VertexStepMode,
            binding_types::{sampler, texture_2d, uniform_buffer},
        },
        renderer::{RenderDevice, RenderQueue},
        view::{self, ExtractedView, RenderVisibleEntities, ViewUniform, VisibilityClass},
    },
};
use bytemuck::{Pod, Zeroable};

pub const BILLBOARD_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("6b8a508f-fb80-479e-bcf3-c3f565c3ed5b");

pub struct BillboardPlugin;

impl Plugin for BillboardPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        load_internal_asset!(
            app,
            BILLBOARD_SHADER_HANDLE,
            "new_billboard.wgsl",
            Shader::from_wgsl
        );

        // Custom phase item
        app.add_plugins(ExtractComponentPlugin::<BillboardPhaseItem>::default());

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(
                    Render,
                    prepare_billboard_phase_item_buffers.in_set(RenderSet::Prepare),
                )
                .add_systems(Render, queue_billboard_phase_item.in_set(RenderSet::Queue));
        }
    }

    fn finish(&self, app: &mut bevy::app::App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<BillboardPhasePipeline>()
                .init_resource::<SpecializedRenderPipelines<BillboardPhasePipeline>>()
                .add_render_command::<Transparent3d, DrawBillboardPhaseItemCommands>();
        }
    }
}

#[derive(Clone, Component, ExtractComponent)]
#[require(VisibilityClass)]
#[component(on_add = view::add_visibility_class::<BillboardPhaseItem>)]
pub struct BillboardPhaseItem;

#[derive(Resource)]
struct BillboardPhasePipeline {
    pub view_layout: BindGroupLayout,
    pub image_layout: BindGroupLayout,
    shader: Handle<Shader>,
}

impl FromWorld for BillboardPhasePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let view_layout = render_device.create_bind_group_layout(
            "billboard_view_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                uniform_buffer::<ViewUniform>(true),
            ),
        );

        let image_layout = render_device.create_bind_group_layout(
            "billboard_image_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );
        // TODO: Forward uniform?

        BillboardPhasePipeline {
            view_layout,
            image_layout,
            shader: BILLBOARD_SHADER_HANDLE,
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

        // Draw one quad (3 vertices).
        pass.draw_indexed(0..4, 0, 0..1);

        RenderCommandResult::Success
    }
}

#[derive(Resource)]
struct BillboardPhaseItemBuffers {
    /// The vertices for the single triangle.
    ///
    /// This is a [`RawBufferVec`] because that's the simplest and fastest type
    /// of GPU buffer, and [`Vertex`] objects are simple.
    vertices: RawBufferVec<BillboardVertex>,

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

        for vertex in &QUAD_VERTEX_POSITIONS {
            vbo.push(BillboardVertex::new(vertex.extend(0.0)));
        }
        for index in QUAD_INDICES {
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
struct BillboardVertex {
    /// The 3D position of the triangle vertex.
    position: [f32; 3],
    // TODO: uv
    uv: [f32; 2],
}

impl BillboardVertex {
    /// Creates a new vertex structure.
    const fn new(position: Vec3) -> BillboardVertex {
        BillboardVertex {
            position: [position.x, position.y, position.z],
            uv: [0.0, 0.0],
        }
    }
}

type DrawBillboardPhaseItemCommands = (SetItemPipeline, DrawBillboardPhaseItem);

const QUAD_VERTEX_POSITIONS: [Vec2; 4] = [
    Vec2::new(-0.5, -0.5),
    Vec2::new(0.5, -0.5),
    Vec2::new(0.5, 0.5),
    Vec2::new(-0.5, 0.5),
];

const QUAD_INDICES: [u32; 6] = [0, 2, 3, 0, 1, 2];

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
        let vertex_layout = VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Vertex,
            vec![
                // position
                VertexFormat::Float32x3,
                // uv
                VertexFormat::Float32x2,
            ],
        );

        RenderPipelineDescriptor {
            label: Some("billboard_render_pipeline".into()),
            layout: vec![self.view_layout.clone(), self.image_layout.clone()],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    // TODO
                    // Ordinarily, you'd want to check whether the view has the
                    // HDR format and substitute the appropriate texture format
                    // here, but we omit that for simplicity.
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
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
