use bevy::{
    app::{App, Plugin},
    render::{ExtractSchedule, RenderApp},
};
use pipeline::BillboardPipeline;

mod extract;
mod pipeline;

#[derive(Debug, Default)]
pub struct BillboardRenderPlugin;

impl Plugin for BillboardRenderPlugin {
    fn build(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Some(render_app) => render_app,
            None => return,
        };

        // The first thing we need is a pipeline, but that we need to do in finish, because we need RenderDevice

        // Next we need to extract our stuff into the render world
        render_app.add_systems(ExtractSchedule, (extract::extract_billboard,));

        // TODO: Next step
    }

    fn finish(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Some(render_app) => render_app,
            None => return,
        };

        // The first thing we need is a pipeline for our usecase
        // This will define the layouts -> what is available to the shader?
        render_app.init_resource::<BillboardPipeline>();
    }
}
