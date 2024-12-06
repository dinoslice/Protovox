use shipyard::{AllStoragesView, IntoWorkload, IntoWorkloadTrySystem, Unique, UniqueView, Workload};
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::block_outline::update_block_outline_buffer;
use crate::rendering::camera_uniform_buffer::update_camera_uniform_buffer;

mod world;
mod gizmos;
mod block_outline;
mod egui;
mod skybox;

pub fn render() -> Workload {
    (
        (
            update_block_outline_buffer,
            update_camera_uniform_buffer,
            create_new_render_context
                .into_workload_try_system()
                .expect("failed to convert to try_system?"),
        ).into_workload(),
        (
            skybox::render_skybox,
            world::render_world,
            gizmos::render_line_gizmos,
            block_outline::render_block_outline,
            egui::render_egui,
            submit_rendered_frame,
        ).into_sequential_workload()
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

fn submit_rendered_frame(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    let Ok(RenderContext { output, encoder, .. }) = storages.remove_unique::<RenderContext>() else {
        tracing::error!("Render context doesn't exist to render to.");
        return;
    };

    g_ctx.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}