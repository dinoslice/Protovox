use shipyard::{AllStoragesView, Unique, UniqueView};
use wgpu::{TextureAspect, TextureUsages};
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::texture::Texture;

pub mod world;
pub mod block_outline;
pub mod skybox;
pub mod entity;

#[derive(Unique)]
pub struct RenderContext {
    pub output: wgpu::SurfaceTexture,
    pub tex_view: wgpu::TextureView,
    pub multisample: wgpu::Texture,
    pub multisample_view: wgpu::TextureView,
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

    let multisample = g_ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Multisample Texture"),
        size: output.texture.size(),
        mip_level_count: output.texture.mip_level_count(),
        sample_count: 4,
        dimension: output.texture.dimension(),
        format: output.texture.format(),
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let multisample_view = multisample.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Multisample Texture View"),
        format: Some(multisample.format()),
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(multisample.mip_level_count()),
        base_array_layer: 0,
        array_layer_count: None,
    });

    storages.add_unique(RenderContext { output, tex_view, multisample, multisample_view, encoder });

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