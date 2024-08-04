use bevy::{
    core_pipeline::core_3d::Transparent3d,
    render::render_phase::{
        RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
    },
};

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
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _item: &Transparent3d,
        _view: (),
        _entity: Option<()>,
        _param: (),
        _pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        /* TODO: Set bindgroup on pass */

        RenderCommandResult::Success
    }
}

pub struct SetBillboardTextureBindGroup<const I: usize>;
impl<const I: usize> RenderCommand<Transparent3d> for SetBillboardTextureBindGroup<I> {
    type Param = ();
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _item: &Transparent3d,
        _view: (),
        _entity: Option<()>,
        _param: (),
        _pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        /* TODO: Set bindgroup on pass */

        RenderCommandResult::Success
    }
}

pub struct DrawBillboard;
impl RenderCommand<Transparent3d> for DrawBillboard {
    type Param = ();
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _item: &Transparent3d,
        _view: (),
        _entity: Option<()>,
        _param: (),
        _pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        /* TODO: Set index/vertex and draw on pass */

        RenderCommandResult::Success
    }
}
