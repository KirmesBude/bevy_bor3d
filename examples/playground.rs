use bevy::{
    core_pipeline::core_3d::{CORE_3D_DEPTH_FORMAT, Transparent3d},
    ecs::{
        query::ROQueryItem,
        system::{
            SystemParamItem,
            lifetimeless::{Read, SRes},
        },
    },
    image::{ImageLoaderSettings, ImageSampler},
    math::Affine3,
    platform::collections::{HashMap, hash_map::Entry},
    prelude::*,
    render::{
        Extract, Render, RenderApp, RenderSet,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        primitives::Aabb,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, BlendState,
            BufferUsages, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState,
            FragmentState, IndexFormat, MultisampleState, PipelineCache, PrimitiveState,
            RawBufferVec, RenderPipelineDescriptor, SamplerBindingType, ShaderStages, ShaderType,
            SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat,
            TextureSampleType, UniformBuffer, VertexState,
            binding_types::{sampler, texture_2d, uniform_buffer},
        },
        renderer::{RenderDevice, RenderQueue},
        sync_world::RenderEntity,
        texture::GpuImage,
        view::{
            self, ExtractedView, RenderVisibleEntities, ViewUniform, ViewUniformOffset,
            ViewUniforms, VisibilityClass,
        },
    },
};

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
struct CustomRenderedEntity {
    image: Handle<Image>,
}

/// Holds a reference to our shader.
///
/// This is loaded at app creation time.
#[derive(Resource)]
struct CustomPhasePipeline {
    view_layout: BindGroupLayout,
    image_layout: BindGroupLayout,
    billboard_layout: BindGroupLayout,
    shader: Handle<Shader>,
}

#[derive(ShaderType, Clone)]
pub struct BillboardUniform {
    pub world_from_local: [Vec4; 3],
    pub size: Vec3,
}

impl FromWorld for CustomPhasePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let view_layout = render_device.create_bind_group_layout(
            "custom_view_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                uniform_buffer::<ViewUniform>(true),
            ),
        );

        let image_layout = render_device.create_bind_group_layout(
            "custom_image_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let billboard_layout = render_device.create_bind_group_layout(
            "custom_billboard_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX,
                uniform_buffer::<BillboardUniform>(false),
            ),
        );

        // Load and compile the shader in the background.
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/custom_phase_item.wgsl");

        Self {
            view_layout,
            image_layout,
            billboard_layout,
            shader,
        }
    }
}

type DrawCustomPhaseItemRenderCommand = (
    SetItemPipeline,
    SetCustomViewBindGroup<0>,
    SetCustomImageBindGroup<1>,
    SetCustomBillboardBindGroup<2>,
    DrawCustomPhaseItem,
);

struct SetCustomViewBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetCustomViewBindGroup<I> {
    type ViewQuery = (Read<ViewUniformOffset>, Read<ViewBindGroup>);
    type ItemQuery = ();
    type Param = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        (view_uniform, view_bind_group): ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &view_bind_group.value, &[view_uniform.offset]);
        RenderCommandResult::Success
    }
}

struct SetCustomImageBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetCustomImageBindGroup<I> {
    type ViewQuery = ();
    type ItemQuery = Read<CustomRenderedEntity>;
    type Param = SRes<ImageBindGroups>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(entity) = entity else {
            return RenderCommandResult::Skip;
        };

        let image_bind_groups = param.into_inner();
        let Some(bind_group) = image_bind_groups.values.get(&entity.image.id()) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, bind_group, &[]);

        RenderCommandResult::Success
    }
}

struct SetCustomBillboardBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetCustomBillboardBindGroup<I> {
    type ViewQuery = ();
    type ItemQuery = Read<BillboardBindGroup>;
    type Param = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(billboard_bind_group) = entity else {
            return RenderCommandResult::Skip;
        };

        pass.set_bind_group(I, &billboard_bind_group.value, &[]);

        RenderCommandResult::Success
    }
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
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // Borrow check workaround.
        let custom_phase_item_buffers = param.into_inner();

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
    /// The indices of the single triangle.
    ///
    /// As above, this is a [`RawBufferVec`] because `u32` values have trivial
    /// size and alignment.
    indices: RawBufferVec<u32>,
}

const QUAD_INDICES: [u32; 6] = [2, 0, 1, 1, 3, 2];

/// The entry point.
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(CustomPhaseItemPlugin)
        .add_systems(Startup, setup);

    app.run();
}

pub struct CustomPhaseItemPlugin;

impl Plugin for CustomPhaseItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<CustomRenderedEntity>::default());

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(
                    Render,
                    prepare_custom_phase_item_buffers.in_set(RenderSet::Prepare),
                )
                .add_systems(
                    ExtractSchedule,
                    extract_custom_phase_item_billboard_transforms,
                )
                .add_systems(
                    Render,
                    (
                        prepare_custom_phase_item_view_bind_group,
                        prepare_custom_phase_item_image_bind_group,
                        prepare_custom_phase_item_billboard_bind_group,
                    )
                        .in_set(RenderSet::PrepareBindGroups),
                )
                .add_systems(Render, queue_custom_phase_item.in_set(RenderSet::Queue));
        }
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<CustomPhasePipeline>()
                .init_resource::<SpecializedRenderPipelines<CustomPhasePipeline>>()
                .init_resource::<ImageBindGroups>()
                .add_render_command::<Transparent3d, DrawCustomPhaseItemRenderCommand>();
        }
    }
}

/// Spawns the objects in the scene.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a single entity that has custom rendering. It'll be extracted into
    // the render world via [`ExtractComponent`].
    commands.spawn((
        Visibility::default(),
        Transform::from_translation(vec3(0.5, 0.0, 0.0)),
        // This `Aabb` is necessary for the visibility checks to work.
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::splat(0.5),
        },
        CustomRenderedEntity {
            image: asset_server.load_with_settings(
                "sprites/bossa1.png",
                |s: &mut ImageLoaderSettings| {
                    s.sampler = ImageSampler::nearest();
                },
            ),
        },
    ));

    // Spawn the camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Creates the [`CustomPhaseItemBuffers`] resource.
///
/// This must be done in a startup system because it needs the [`RenderDevice`]
/// and [`RenderQueue`] to exist, and they don't until [`App::run`] is called.
fn prepare_custom_phase_item_buffers(mut commands: Commands) {
    commands.init_resource::<CustomPhaseItemBuffers>();
}

#[derive(Component)]
pub struct ViewBindGroup {
    value: BindGroup,
}

fn prepare_custom_phase_item_view_bind_group(
    mut commands: Commands,
    view_uniforms: Res<ViewUniforms>,
    views: Query<Entity, With<ExtractedView>>,
    custom_phase_pipeline: Res<CustomPhasePipeline>,
    render_device: Res<RenderDevice>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    for entity in &views {
        let view_bind_group = render_device.create_bind_group(
            "custom_view_bind_group",
            &custom_phase_pipeline.view_layout,
            &BindGroupEntries::single(view_binding.clone()),
        );

        commands.entity(entity).insert(ViewBindGroup {
            value: view_bind_group,
        });
    }
}

#[derive(Resource, Default)]
pub struct ImageBindGroups {
    values: HashMap<AssetId<Image>, BindGroup>,
}

fn prepare_custom_phase_item_image_bind_group(
    mut image_bind_groups: ResMut<ImageBindGroups>,
    custom_phase_items: Query<&CustomRenderedEntity>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    custom_phase_pipeline: Res<CustomPhasePipeline>,
    render_device: Res<RenderDevice>,
) {
    for custom_phase_item in &custom_phase_items {
        let asset_id = custom_phase_item.image.id();
        if let Some(gpu_image) = gpu_images.get(asset_id) {
            let bind_group = render_device.create_bind_group(
                "custom_image_bind_group",
                &custom_phase_pipeline.image_layout,
                &BindGroupEntries::sequential((&gpu_image.texture_view, &gpu_image.sampler)),
            );
            match image_bind_groups.values.entry(asset_id) {
                Entry::Occupied(mut oe) => {
                    oe.insert(bind_group);
                }
                Entry::Vacant(ve) => {
                    ve.insert(bind_group);
                }
            }
        }
    }
}

fn extract_custom_phase_item_billboard_transforms(
    mut commands: Commands,
    billboards: Extract<Query<(RenderEntity, &GlobalTransform)>>,
) {
    for (entity, transform) in &billboards {
        commands
            .entity(entity)
            .insert(ExtractedBillboardTransforms {
                world_from_local: Affine3::from(&transform.affine()),
            });
    }
}

#[derive(Component)]
pub struct ExtractedBillboardTransforms {
    world_from_local: Affine3,
}

#[derive(Component)]
pub struct BillboardBindGroup {
    value: BindGroup,
}

fn prepare_custom_phase_item_billboard_bind_group(
    mut commands: Commands,
    billboards: Query<(Entity, &ExtractedBillboardTransforms, &CustomRenderedEntity)>,
    custom_phase_pipeline: Res<CustomPhasePipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    for (entity, transforms, custom_phase_item) in &billboards {
        let asset_id = custom_phase_item.image.id();
        if let Some(gpu_image) = gpu_images.get(asset_id) {
            let size = gpu_image.size;
            let size = vec3(
                size.width as f32,
                size.height as f32,
                size.depth_or_array_layers as f32,
            );

            let uniform = BillboardUniform {
                world_from_local: transforms.world_from_local.to_transpose(),
                size,
            };
            let mut uniform_buffer = UniformBuffer::<BillboardUniform>::from(uniform);
            uniform_buffer.write_buffer(&render_device, &render_queue);

            if let Some(binding) = uniform_buffer.binding() {
                let billboard_bind_group = render_device.create_bind_group(
                    "custom_billboard_bind_group",
                    &custom_phase_pipeline.billboard_layout,
                    &BindGroupEntries::single(binding),
                );

                commands.entity(entity).insert(BillboardBindGroup {
                    value: billboard_bind_group,
                });
            }
        }
    }
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
        .id::<DrawCustomPhaseItemRenderCommand>();

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
            layout: vec![
                self.view_layout.clone(),
                self.image_layout.clone(),
                self.billboard_layout.clone(),
            ],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![],
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

impl FromWorld for CustomPhaseItemBuffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        // Create the vertex and index buffers.
        let mut ibo = RawBufferVec::new(BufferUsages::INDEX);

        for index in QUAD_INDICES {
            ibo.push(index);
        }

        // These two lines are required in order to trigger the upload to GPU.
        ibo.write_buffer(render_device, render_queue);

        CustomPhaseItemBuffers { indices: ibo }
    }
}
