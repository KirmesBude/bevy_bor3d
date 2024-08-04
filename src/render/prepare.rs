use bevy::{
    math::Vec2,
    prelude::{Commands, Component, Entity, Query, Res, With},
    render::{
        render_asset::RenderAssets,
        render_resource::{BindGroup, BindGroupEntries, BufferUsages, BufferVec, ShaderType},
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        view::{ExtractedView, ViewUniforms},
    },
};

use super::{extract::ExtractedBillboard, pipeline::BillboardPipeline};

#[derive(Component)]
pub struct BillboardViewBindGroup {
    pub value: BindGroup,
}

pub fn prepare_billboard_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    billboard_pipeline: Res<BillboardPipeline>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<Entity, With<ExtractedView>>,
) {
    let Some(view_binding) = view_uniforms.uniforms.binding() else {
        return;
    };

    for entity in &views {
        let view_bind_group = render_device.create_bind_group(
            "billboard_view_bind_group",
            &billboard_pipeline.view_layout,
            &BindGroupEntries::with_indices(((0, view_binding.clone()),)),
        );

        commands.entity(entity).insert(BillboardViewBindGroup {
            value: view_bind_group,
        });
    }
}

const QUAD_INDICES: [usize; 6] = [0, 2, 3, 0, 1, 2];

const QUAD_VERTEX_POSITIONS: [Vec2; 4] = [
    Vec2::new(-0.5, -0.5),
    Vec2::new(0.5, -0.5),
    Vec2::new(0.5, 0.5),
    Vec2::new(-0.5, 0.5),
];

const QUAD_UVS: [Vec2; 4] = [
    Vec2::new(0., 1.),
    Vec2::new(1., 1.),
    Vec2::new(1., 0.),
    Vec2::new(0., 0.),
];

/* TODO: Having this as a component is not efficient I guess */
#[derive(Component)]
pub struct BillboardTextureBindGroup {
    pub value: BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, ShaderType)]
pub struct BillboardVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Component)]
pub struct BillboardMeta {
    pub vertices: BufferVec<BillboardVertex>,
}

impl Default for BillboardMeta {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
        }
    }
}

pub fn prepare_billboard_texture_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    billboard_pipeline: Res<BillboardPipeline>,
    extracted_billboard_q: Query<(Entity, &ExtractedBillboard)>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    for (entity, extracted_billboard) in &extracted_billboard_q {
        let Some(gpu_image) = gpu_images.get(extracted_billboard.texture) else {
            continue;
        };

        let texture_bind_group = render_device.create_bind_group(
            "billboard_texture_bind_group",
            &billboard_pipeline.material_layout,
            &BindGroupEntries::with_indices((
                (0, &gpu_image.texture_view),
                (1, &gpu_image.sampler),
            )),
        );

        commands.entity(entity).insert(BillboardTextureBindGroup {
            value: texture_bind_group,
        });

        /* Quad vertices*/
        let uvs = QUAD_UVS;
        //let quad_size = gpu_image.size;
        // Apply size and global transform
        let positions = QUAD_VERTEX_POSITIONS.map(|quad_pos| quad_pos.extend(0.).into());

        let mut billboard_meta = BillboardMeta::default();
        for i in QUAD_INDICES {
            billboard_meta.vertices.push(BillboardVertex {
                position: positions[i],
                uv: uvs[i].into(),
            });
        }
        billboard_meta
            .vertices
            .write_buffer(&render_device, &render_queue);
        commands.entity(entity).insert(billboard_meta);
    }
}
