use shipyard::{AllStoragesView, IntoIter, IntoWorkload, Unique, UniqueView, UniqueViewMut, View, Workload};
use crate::rendering::base_face::BaseFace;
use crate::camera::Camera;
use crate::chunks::chunk_manager::ChunkManager;
use crate::components::{LocalPlayer, Transform};
use crate::rendering::camera_uniform_buffer::CameraUniformBuffer;
use crate::rendering::depth_texture::DepthTexture;
use crate::rendering::gizmos::line_render_state::GizmosLineRenderState;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::renderer::RenderPipeline;
use crate::rendering::texture_atlas::TextureAtlas;

mod world;
mod gizmos;

pub fn render() -> Workload {
    (
        update_camera_uniform_buffer,
        create_new_render_context,
        world::render_world,
        gizmos::render_line_gizmos,
        submit_rendered_frame,
    ).into_sequential_workload()
}

#[derive(Unique)]
struct RenderContext {
    pub output: wgpu::SurfaceTexture,
    pub tex_view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,
}

fn create_new_render_context(storages: AllStoragesView, g_ctx: UniqueView<GraphicsContext>) -> Result<(), wgpu::SurfaceError> {
    // get a surface texture to render to
    let output = g_ctx.surface.get_current_texture()?;

    // view of the texture, so we can control how the render code interacts with the texture
    let tex_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    // command encoder creates the commands to send to the GPU, commands stored in command buffer
    let encoder = g_ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render encoder"),
    });

    storages.add_unique(RenderContext { output, tex_view, encoder });

    Ok(())
}

fn update_camera_uniform_buffer(
    g_ctx: UniqueView<GraphicsContext>,
    cam_uniform_buffer: UniqueView<CameraUniformBuffer>,
    v_local_player: View<LocalPlayer>,
    v_camera: View<Camera>,
    v_transform: View<Transform>,
) {
    let (_, render_cam, transform) = (&v_local_player, &v_camera, &v_transform)
        .iter()
        .next()
        .expect("TODO: local player did not have camera to render to");

    cam_uniform_buffer.update_buffer(&g_ctx, &render_cam.as_uniform(transform));
}

fn submit_rendered_frame(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    let Ok(RenderContext { output, encoder, .. }) = storages.remove_unique::<RenderContext>() else {
        tracing::error!("Render context doesn't exist to render to.");
        return;
    };

    g_ctx.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}