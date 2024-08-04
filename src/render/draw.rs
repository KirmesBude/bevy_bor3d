use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::{query::ROQueryItem, system::lifetimeless::Read},
    render::{
        render_phase::{RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass},
        view::ViewUniformOffset,
    },
};

use super::prepare::{BillboardMeta, BillboardTextureBindGroup, BillboardViewBindGroup};

pub type DrawBillboardRenderCommand = (
    SetItemPipeline,
    SetBillboardViewGroup<0>,
    SetBillboardTextureBindGroup<1>,
    DrawBillboard,
);

/* TODO: Do I need custom SetItemPipeline? */

pub struct SetBillboardViewGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent3d> for SetBillboardViewGroup<I> {
    type Param = ();
    type ViewQuery = (Read<ViewUniformOffset>, Read<BillboardViewBindGroup>);
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _item: &Transparent3d,
        (view_uniform, billboard_view_bind_group): ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<()>,
        _param: (),
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &billboard_view_bind_group.value, &[view_uniform.offset]);

        RenderCommandResult::Success
    }
}

pub struct SetBillboardTextureBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent3d> for SetBillboardTextureBindGroup<I> {
    type Param = ();
    type ViewQuery = ();
    type ItemQuery = Read<BillboardTextureBindGroup>;

    #[inline]
    fn render<'w>(
        _item: &Transparent3d,
        _view: (),
        texture_bind_group: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _param: (),
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(texture_bind_group) = texture_bind_group else {
            return RenderCommandResult::Failure;
        };
        pass.set_bind_group(I, &texture_bind_group.value, &[]);

        RenderCommandResult::Success
    }
}

pub struct DrawBillboard;
impl RenderCommand<Transparent3d> for DrawBillboard {
    type Param = ();
    type ViewQuery = ();
    type ItemQuery = Read<BillboardMeta>;

    #[inline]
    fn render<'w>(
        item: &Transparent3d,
        _view: (),
        billboard_meta: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _param: (),
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(billboard_meta) = billboard_meta else {
            return RenderCommandResult::Failure;
        };

        pass.set_vertex_buffer(0, billboard_meta.vertices.buffer().unwrap().slice(..));
        pass.draw(item.batch_range.clone(), 0..1);

        RenderCommandResult::Success
    }
}
