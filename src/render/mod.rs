use bevy::{
    app::{App, Plugin},
    core_pipeline::core_3d::Transparent3d,
    prelude::IntoSystemConfigs,
    render::{render_phase::AddRenderCommand, ExtractSchedule, Render, RenderApp, RenderSet},
};

mod draw;
mod extract;
mod pipeline;
mod prepare;
mod queue;

#[derive(Debug, Default)]
pub struct BillboardRenderPlugin;

impl Plugin for BillboardRenderPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Some(render_app) => render_app,
            None => return,
        };

        // The first thing we need is a pipeline for our usecase
        // This will define the layouts -> what is available to the shader?
        render_app.init_resource::<pipeline::BillboardPipeline>();

        // Next we need to extract our stuff into the render world
        render_app.add_systems(ExtractSchedule, (extract::extract_billboard,));

        // Queue
        render_app.add_systems(Render, (queue::queue_billboard,).in_set(RenderSet::Queue));

        // Prepare
        render_app.add_systems(
            Render,
            (
                prepare::prepare_billboard_view_bind_groups,
                prepare::prepare_billboard_texture_bind_groups,
            )
                .in_set(RenderSet::PrepareBindGroups),
        );

        // TODO: RenderCommand
        render_app.add_render_command::<Transparent3d, draw::DrawBillboardRenderCommand>();
    }
}
