use shipyard::{AllStoragesView, Unique, UniqueView};
use crate::rendering::graphics_context::GraphicsContext;

pub mod world;
pub mod block_outline;
pub mod skybox;
pub mod entity;

#[derive(Unique)]
pub struct RenderContext {
    pub output: wgpu::SurfaceTexture,
    pub tex_view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,
}

pub fn create_new_render_context(storages: AllStoragesView, g_ctx: UniqueView<GraphicsContext>) -> Result<(), wgpu::SurfaceError> {
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

pub fn submit_rendered_frame(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    let Ok(RenderContext { output, encoder, .. }) = storages.remove_unique::<RenderContext>() else {
        tracing::error!("Render context doesn't exist to render to.");
        return;
    };

    g_ctx.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}