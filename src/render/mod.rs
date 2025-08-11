use bevy::{
    app::{App, Plugin},
    asset::{AssetId, Handle, load_internal_asset, weak_handle},
    core_pipeline::core_3d::{CORE_3D_DEPTH_FORMAT, Transparent3d},
    ecs::{
        component::Component,
        entity::Entity,
        query::{ROQueryItem, With},
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{
            Commands, Query, Res, ResMut, SystemParamItem,
            lifetimeless::{Read, SRes},
        },
        world::{FromWorld, World},
    },
    image::{BevyDefault, Image},
    math::{Affine3, Vec3, Vec4, vec3},
    platform::collections::HashMap,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, BlendState,
            BufferUsages, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState,
            FragmentState, IndexFormat, MultisampleState, PipelineCache, PrimitiveState,
            RawBufferVec, RenderPipelineDescriptor, SamplerBindingType, Shader, ShaderDefVal,
            ShaderStages, ShaderType, SpecializedRenderPipeline, SpecializedRenderPipelines,
            TextureFormat, TextureSampleType, UniformBuffer, VertexState,
            binding_types::{sampler, texture_2d_array, uniform_buffer},
        },
        renderer::{RenderDevice, RenderQueue},
        sync_world::RenderEntity,
        texture::GpuImage,
        view::{
            ExtractedView, Msaa, RenderVisibleEntities, ViewUniform, ViewUniformOffset,
            ViewUniforms,
        },
    },
    transform::components::GlobalTransform,
    utils::default,
};

use crate::{Billboard, Sprite3d};

pub struct Sprite3dRenderPlugin;

pub const SPRITE3D_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("0213e1f3-e431-4d98-943b-a65de5c99d86");
pub const SPRITE3D_VIEW_BINDINGS_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("8f4b5886-a51a-4851-8326-c0cf4855787d");
pub const SPRITE3D_BINDINGS_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("72e50b9a-71da-43b1-8292-abc4927d904e");
pub const SPRITE3D_TYPES_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("cf67db3e-d1ab-428a-a1d6-6ecac1c6f5d2");

impl Plugin for Sprite3dRenderPlugin {
    fn build(&self, app: &mut App) {
        // Loading all shaders
        load_internal_asset!(
            app,
            SPRITE3D_SHADER_HANDLE,
            "sprite3d.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SPRITE3D_VIEW_BINDINGS_SHADER_HANDLE,
            "sprite3d_view_bindings.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SPRITE3D_BINDINGS_SHADER_HANDLE,
            "sprite3d_bindings.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SPRITE3D_TYPES_SHADER_HANDLE,
            "sprite3d_types.wgsl",
            Shader::from_wgsl
        );

        // Schedule systems
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(ExtractSchedule, extract_sprite3d)
                .add_systems(
                    Render,
                    (
                        prepare_sprite3d_view_bind_group,
                        prepare_sprite3d_image_bind_group,
                        prepare_sprite3d_bind_group,
                    )
                        .in_set(RenderSet::PrepareBindGroups),
                )
                .add_systems(Render, queue_sprite3d.in_set(RenderSet::Queue));
        }
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<Sprite3dPipeline>()
                .init_resource::<SpecializedRenderPipelines<Sprite3dPipeline>>()
                .init_resource::<Sprite3dRenderBuffer>()
                .init_resource::<ImageBindGroups>()
                .add_render_command::<Transparent3d, DrawSprite3dRenderCommand>();
        }
    }
}

// Pipeline stuff
//

#[derive(Debug, Resource)]
struct Sprite3dPipeline {
    view_layout: BindGroupLayout,
    image_layout: BindGroupLayout,
    sprite3d_layout: BindGroupLayout,
    shader: Handle<Shader>,
}

impl FromWorld for Sprite3dPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let view_layout = render_device.create_bind_group_layout(
            "bor3d_sprite3d_view_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                uniform_buffer::<ViewUniform>(true),
            ),
        );

        let image_layout = render_device.create_bind_group_layout(
            "bor3d_sprite3d_image_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d_array(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let sprite3d_layout = render_device.create_bind_group_layout(
            "bor3d_sprite3d_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX,
                uniform_buffer::<Sprite3dUniform>(false),
            ),
        );

        let shader = SPRITE3D_SHADER_HANDLE;
        Self {
            view_layout,
            image_layout,
            sprite3d_layout,
            shader,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Sprite3dPipelineKey {
    msaa: Msaa,
    billboard: ExtractedBillboard,
}

impl SpecializedRenderPipeline for Sprite3dPipeline {
    type Key = Sprite3dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("bor3d_sprite3d_render_pipeline".into()),
            layout: vec![
                self.view_layout.clone(),
                self.image_layout.clone(),
                self.sprite3d_layout.clone(),
            ],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![ShaderDefVal::UInt("BILLBOARD".into(), key.billboard as u32)],
                entry_point: "vertex".into(),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![ShaderDefVal::UInt("BILLBOARD".into(), key.billboard as u32)], // TODO: For some reason I need to add it here too?
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
                depth_compare: CompareFunction::Greater,
                stencil: default(),
                bias: default(),
            }),
            multisample: MultisampleState {
                count: key.msaa.samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}

#[derive(ShaderType, Clone)]
pub struct Sprite3dUniform {
    pub world_from_local: [Vec4; 3],
    pub size: Vec3,
}

// GPU stuff
//

const QUAD_INDICES_LEN: usize = 6;
const QUAD_INDICES: [u32; QUAD_INDICES_LEN] = [2, 0, 1, 1, 3, 2];

#[derive(Resource)]
struct Sprite3dRenderBuffer {
    indices: RawBufferVec<u32>,
}

impl FromWorld for Sprite3dRenderBuffer {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let mut index_buffer = RawBufferVec::new(BufferUsages::INDEX);

        for index in QUAD_INDICES {
            index_buffer.push(index);
        }

        index_buffer.write_buffer(render_device, render_queue);

        Self {
            indices: index_buffer,
        }
    }
}

// Render Commands
//

type DrawSprite3dRenderCommand = (
    SetItemPipeline, // TODO: What does this do? Do I need this?
    SetSprite3dViewBindGroup<0>,
    SetSprite3dImageBindGroup<1>,
    SetSprite3dBindGroup<2>,
    DrawSprite3d,
);

#[derive(Component)]
struct Sprite3dViewBindGroup {
    value: BindGroup,
}

struct SetSprite3dViewBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetSprite3dViewBindGroup<I> {
    type ViewQuery = (Read<ViewUniformOffset>, Read<Sprite3dViewBindGroup>);
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

#[derive(Resource, Default)]
pub struct ImageBindGroups {
    values: HashMap<AssetId<Image>, BindGroup>,
}

struct SetSprite3dImageBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetSprite3dImageBindGroup<I> {
    type ViewQuery = ();
    type ItemQuery = Read<ExtractedSprite3d>;
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
        let Some(bind_group) = image_bind_groups.values.get(&entity.asset_id) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, bind_group, &[]);

        RenderCommandResult::Success
    }
}

#[derive(Component)]
pub struct Sprite3dBindGroup {
    value: BindGroup,
}

struct SetSprite3dBindGroup<const I: usize>;
impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetSprite3dBindGroup<I> {
    type ViewQuery = ();
    type ItemQuery = Read<Sprite3dBindGroup>;
    type Param = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(sprite3d_bind_group) = entity else {
            return RenderCommandResult::Skip;
        };

        pass.set_bind_group(I, &sprite3d_bind_group.value, &[]);

        RenderCommandResult::Success
    }
}

struct DrawSprite3d;
impl<P> RenderCommand<P> for DrawSprite3d
where
    P: PhaseItem,
{
    type Param = SRes<Sprite3dRenderBuffer>;

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
        let sprite3d_render_buffers = param.into_inner();

        // Tell the GPU where the indices are.
        pass.set_index_buffer(
            sprite3d_render_buffers.indices.buffer().unwrap().slice(..),
            0,
            IndexFormat::Uint32,
        );

        // Draw one quad (4 vertices?).
        pass.draw_indexed(0..(QUAD_INDICES_LEN as u32), 0, 0..1);

        RenderCommandResult::Success
    }
}

// Extract
//

#[derive(Component)]
struct ExtractedSprite3d {
    transform: GlobalTransform,
    asset_id: AssetId<Image>,
    billboard: ExtractedBillboard,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
enum ExtractedBillboard {
    None = 0,
    Forward = 1,
    LookAt = 2,
}

impl From<Option<&Billboard>> for ExtractedBillboard {
    fn from(value: Option<&Billboard>) -> Self {
        match value {
            Some(billboard) => match billboard {
                Billboard::Forward => Self::Forward,
                Billboard::LookAt => Self::LookAt,
            },
            None => Self::None,
        }
    }
}

fn extract_sprite3d(
    mut commands: Commands,
    sprite3d_query: Extract<
        Query<(
            RenderEntity,
            &Sprite3d,
            &GlobalTransform,
            Option<&Billboard>,
        )>,
    >,
) {
    for (entity, sprite3d, transform, billboard) in &sprite3d_query {
        commands.entity(entity).insert(ExtractedSprite3d {
            transform: *transform,
            asset_id: sprite3d.image.id(),
            billboard: billboard.into(),
        });
    }
}

// Prepare
//

fn prepare_sprite3d_view_bind_group(
    mut commands: Commands,
    view_uniforms: Res<ViewUniforms>,
    views: Query<Entity, With<ExtractedView>>,
    sprite3d_pipeline: Res<Sprite3dPipeline>,
    render_device: Res<RenderDevice>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    for entity in &views {
        let view_bind_group = render_device.create_bind_group(
            "bor3d_sprite3d_view_bind_group",
            &sprite3d_pipeline.view_layout,
            &BindGroupEntries::single(view_binding.clone()),
        );

        commands.entity(entity).insert(Sprite3dViewBindGroup {
            value: view_bind_group,
        });
    }
}

fn prepare_sprite3d_image_bind_group(
    mut image_bind_groups: ResMut<ImageBindGroups>,
    custom_phase_items: Query<&ExtractedSprite3d>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    sprite3d_pipeline: Res<Sprite3dPipeline>,
    render_device: Res<RenderDevice>,
) {
    for custom_phase_item in &custom_phase_items {
        let asset_id = custom_phase_item.asset_id;
        if let Some(gpu_image) = gpu_images.get(asset_id) {
            let bind_group = render_device.create_bind_group(
                "bor3d_sprite3d_image_bind_group",
                &sprite3d_pipeline.image_layout,
                &BindGroupEntries::sequential((&gpu_image.texture_view, &gpu_image.sampler)),
            );

            image_bind_groups.values.insert(asset_id, bind_group);
        }
    }
}

fn prepare_sprite3d_bind_group(
    mut commands: Commands,
    sprite3d_query: Query<(Entity, &ExtractedSprite3d)>,
    sprite3d_pipeline: Res<Sprite3dPipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    for (entity, extracted_sprite3d) in &sprite3d_query {
        let asset_id = extracted_sprite3d.asset_id;
        let transform = extracted_sprite3d.transform;
        if let Some(gpu_image) = gpu_images.get(asset_id) {
            let size = gpu_image.size;
            let size = vec3(
                size.width as f32,
                size.height as f32,
                size.depth_or_array_layers as f32,
            );

            let uniform = Sprite3dUniform {
                world_from_local: Affine3::from(&transform.affine()).to_transpose(),
                size,
            };
            let mut uniform_buffer = UniformBuffer::<Sprite3dUniform>::from(uniform);
            uniform_buffer.write_buffer(&render_device, &render_queue);

            if let Some(binding) = uniform_buffer.binding() {
                let sprite3d_bind_group = render_device.create_bind_group(
                    "bor3d_sprite3d_bind_group",
                    &sprite3d_pipeline.sprite3d_layout,
                    &BindGroupEntries::single(binding),
                );

                commands.entity(entity).insert(Sprite3dBindGroup {
                    value: sprite3d_bind_group,
                });
            }
        }
    }
}

// Queue
//

fn queue_sprite3d(
    pipeline_cache: Res<PipelineCache>,
    sprite3d_pipeline: Res<Sprite3dPipeline>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    transparent_draw_functions: Res<DrawFunctions<Transparent3d>>,
    mut specialized_render_pipelines: ResMut<SpecializedRenderPipelines<Sprite3dPipeline>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    extraced_sprite3d_query: Query<&ExtractedSprite3d>,
) {
    let sprite3d_draw_function = transparent_draw_functions
        .read()
        .id::<DrawSprite3dRenderCommand>();

    // Render phases are per-view, so we need to iterate over all views so that
    // the entity appears in them. (In this example, we have only one view, but
    // it's good practice to loop over all views anyway.)
    for (view, view_visible_entities, msaa) in views.iter() {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let range_finder = view.rangefinder3d();

        // Find all the custom rendered entities that are visible from this
        // view.
        for (render_entity, visible_entity) in view_visible_entities.get::<Sprite3d>().iter() {
            let Ok(extracted_sprite3d) = extraced_sprite3d_query.get(*render_entity) else {
                continue;
            };

            // Ordinarily, the [`SpecializedRenderPipeline::Key`] would contain
            // some per-view settings, such as whether the view is HDR, but for
            // simplicity's sake we simply hard-code the view's characteristics,
            // with the exception of number of MSAA samples.
            let key = Sprite3dPipelineKey {
                msaa: *msaa,
                billboard: extracted_sprite3d.billboard,
            };

            let pipeline_id =
                specialized_render_pipelines.specialize(&pipeline_cache, &sprite3d_pipeline, key);

            let distance =
                range_finder.distance_translation(&extracted_sprite3d.transform.translation());
            // TODO: How to determine this?
            let indexed = false;
            transparent_phase.add(Transparent3d {
                distance,
                pipeline: pipeline_id,
                entity: (*render_entity, *visible_entity),
                draw_function: sprite3d_draw_function,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed,
            });
        }
    }
}
