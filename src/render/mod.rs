use bevy::{
    app::{App, Plugin},
    asset::{load_internal_asset, Handle},
    core_pipeline::core_3d::Transparent3d,
    prelude::{IntoSystemConfigs, Shader},
    render::{
        render_phase::AddRenderCommand, render_resource::SpecializedRenderPipelines,
        ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

mod draw;
mod extract;
mod pipeline;
mod prepare;
mod queue;

pub const BILLBOARD_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(307062806978783518533214479195188549290);

#[derive(Debug, Default)]
pub struct BillboardRenderPlugin;

impl Plugin for BillboardRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            BILLBOARD_SHADER_HANDLE,
            "billboard.wgsl",
            Shader::from_wgsl
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Some(render_app) => render_app,
            None => return,
        };

        // The first thing we need is a pipeline for our usecase
        // This will define the layouts -> what is available to the shader?
        render_app.init_resource::<pipeline::BillboardPipeline>();
        render_app.init_resource::<SpecializedRenderPipelines<pipeline::BillboardPipeline>>();

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

        // RenderCommand
        render_app.add_render_command::<Transparent3d, draw::DrawBillboardRenderCommand>();
    }
}
