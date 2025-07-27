use bevy::{
    core_pipeline::core_3d::{CORE_3D_DEPTH_FORMAT, Transparent3d},
    ecs::{
        query::ROQueryItem,
        system::{SystemParamItem, lifetimeless::SRes},
    },
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        primitives::Aabb,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            BufferUsages, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState,
            FragmentState, IndexFormat, MultisampleState, PipelineCache, PrimitiveState,
            RawBufferVec, RenderPipelineDescriptor, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, VertexAttribute, VertexBufferLayout,
            VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        view::{self, ExtractedView, RenderVisibleEntities, VisibilityClass},
    },
};
use bytemuck::{Pod, Zeroable};

/// A marker component that represents an entity that is to be rendered using
/// our custom phase item.
///
/// Note the [`ExtractComponent`] trait implementation: this is necessary to
/// tell Bevy that this object should be pulled into the render world. Also note
/// the `on_add` hook, which is needed to tell Bevy's `check_visibility` system
/// that entities with this component need to be examined for visibility.
#[derive(Clone, Component, ExtractComponent)]
#[require(VisibilityClass)]
#[component(on_add = view::add_visibility_class::<CustomRenderedEntity>)]
struct CustomRenderedEntity;

/// Holds a reference to our shader.
///
/// This is loaded at app creation time.
#[derive(Resource)]
struct CustomPhasePipeline {
    shader: Handle<Shader>,
}

/// A [`RenderCommand`] that binds the vertex and index buffers and issues the
/// draw command for our custom phase item.
struct DrawCustomPhaseItem;

impl<P> RenderCommand<P> for DrawCustomPhaseItem
where
    P: PhaseItem,
{
    type Param = SRes<CustomPhaseItemBuffers>;

    type ViewQuery = ();

    type ItemQuery = ();

    fn render<'w>(
        _: &P,
        _: ROQueryItem<'w, Self::ViewQuery>,
        _: Option<ROQueryItem<'w, Self::ItemQuery>>,
        custom_phase_item_buffers: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // Borrow check workaround.
        let custom_phase_item_buffers = custom_phase_item_buffers.into_inner();

        // Tell the GPU where the vertices are.
        pass.set_vertex_buffer(
            0,
            custom_phase_item_buffers
                .vertices
                .buffer()
                .unwrap()
                .slice(..),
        );

        // Tell the GPU where the indices are.
        pass.set_index_buffer(
            custom_phase_item_buffers
                .indices
                .buffer()
                .unwrap()
                .slice(..),
            0,
            IndexFormat::Uint32,
        );

        // Draw one quad (4 vertices?).
        pass.draw_indexed(0..6, 0, 0..1);

        RenderCommandResult::Success
    }
}

/// The GPU vertex and index buffers for our custom phase item.
///
/// As the custom phase item is a single triangle, these are uploaded once and
/// then left alone.
#[derive(Resource)]
struct CustomPhaseItemBuffers {
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

/// The custom draw commands that Bevy executes for each entity we enqueue into
/// the render phase.
type DrawCustomPhaseItemCommands = (SetItemPipeline, DrawCustomPhaseItem);

const QUAD_VERTEX_POSITIONS: [Vec2; 4] = [
    vec2(-0.5, -0.5),
    vec2(0.5, -0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5),
];

const QUAD_INDICES: [u32; 6] = [0, 2, 3, 0, 1, 2];

const QUAD_VERTEX_COLOR: [Vec3; 4] = [
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0),
    vec3(0.5, 0.5, 0.5),
];

/// The entry point.
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(ExtractComponentPlugin::<CustomRenderedEntity>::default())
        .add_systems(Startup, setup);

    // We make sure to add these to the render app, not the main app.
    app.get_sub_app_mut(RenderApp)
        .unwrap()
        .init_resource::<CustomPhasePipeline>()
        .init_resource::<SpecializedRenderPipelines<CustomPhasePipeline>>()
        .add_render_command::<Transparent3d, DrawCustomPhaseItemCommands>()
        .add_systems(
            Render,
            prepare_custom_phase_item_buffers.in_set(RenderSet::Prepare),
        )
        .add_systems(Render, queue_custom_phase_item.in_set(RenderSet::Queue));

    app.run();
}

/// Spawns the objects in the scene.
fn setup(mut commands: Commands) {
    // Spawn a single entity that has custom rendering. It'll be extracted into
    // the render world via [`ExtractComponent`].
    commands.spawn((
        Visibility::default(),
        Transform::default(),
        // This `Aabb` is necessary for the visibility checks to work.
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::splat(0.5),
        },
        CustomRenderedEntity,
    ));

    // Spawn the camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Creates the [`CustomPhaseItemBuffers`] resource.
///
/// This must be done in a startup system because it needs the [`RenderDevice`]
/// and [`RenderQueue`] to exist, and they don't until [`App::run`] is called.
fn prepare_custom_phase_item_buffers(mut commands: Commands) {
    commands.init_resource::<CustomPhaseItemBuffers>();
}

/// A render-world system that enqueues the entity with custom rendering into
/// the opaque render phases of each view.
fn queue_custom_phase_item(
    pipeline_cache: Res<PipelineCache>,
    custom_phase_pipeline: Res<CustomPhasePipeline>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    mut specialized_render_pipelines: ResMut<SpecializedRenderPipelines<CustomPhasePipeline>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
) {
    let draw_custom_phase_item = transparent_draw_functions
        .read()
        .id::<DrawCustomPhaseItemCommands>();

    // Render phases are per-view, so we need to iterate over all views so that
    // the entity appears in them. (In this example, we have only one view, but
    // it's good practice to loop over all views anyway.)
    for (view, view_visible_entities, msaa) in views.iter() {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        // Find all the custom rendered entities that are visible from this
        // view.
        for (render_entity, visible_entity) in
            view_visible_entities.get::<CustomRenderedEntity>().iter()
        {
            // Ordinarily, the [`SpecializedRenderPipeline::Key`] would contain
            // some per-view settings, such as whether the view is HDR, but for
            // simplicity's sake we simply hard-code the view's characteristics,
            // with the exception of number of MSAA samples.
            let pipeline_id = specialized_render_pipelines.specialize(
                &pipeline_cache,
                &custom_phase_pipeline,
                *msaa,
            );

            // TODO: Would need to handle rangefinder somehow
            let distance = 0.0;
            // TODO: How to determine this?
            let indexed = false;
            transparent_phase.add(Transparent3d {
                distance,
                pipeline: pipeline_id,
                entity: (*render_entity, *visible_entity),
                draw_function: draw_custom_phase_item,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed,
            });
        }
    }
}

impl SpecializedRenderPipeline for CustomPhasePipeline {
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

impl FromWorld for CustomPhaseItemBuffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        // Create the vertex and index buffers.
        let mut vbo = RawBufferVec::new(BufferUsages::VERTEX);
        let mut ibo = RawBufferVec::new(BufferUsages::INDEX);

        for (position, color) in QUAD_VERTEX_POSITIONS.iter().zip(QUAD_VERTEX_COLOR) {
            vbo.push(Vertex::new(position.extend(0.0), color));
        }
        for index in QUAD_INDICES {
            ibo.push(index);
        }

        // These two lines are required in order to trigger the upload to GPU.
        vbo.write_buffer(render_device, render_queue);
        ibo.write_buffer(render_device, render_queue);

        CustomPhaseItemBuffers {
            vertices: vbo,
            indices: ibo,
        }
    }
}

impl FromWorld for CustomPhasePipeline {
    fn from_world(world: &mut World) -> Self {
        // Load and compile the shader in the background.
        let asset_server = world.resource::<AssetServer>();

        CustomPhasePipeline {
            shader: asset_server.load("shaders/custom_phase_item.wgsl"),
        }
    }
}
