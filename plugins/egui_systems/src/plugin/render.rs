use egui_wgpu::ScreenDescriptor;
use shipyard::{AllStoragesView, UniqueView, UniqueViewMut};
use engine::rendering::graphics_context::GraphicsContext;
use engine::rendering::render::RenderContext;
use crate::CurrentEguiFrame;
use crate::renderer::EguiRenderer;

pub fn render_start(g_ctx: UniqueView<GraphicsContext>, mut renderer: UniqueViewMut<EguiRenderer>, storages: AllStoragesView) {
    let screen_descriptor = make_screen_descriptor(&g_ctx);

    let context = renderer.context().clone();

    context.set_pixels_per_point(screen_descriptor.pixels_per_point);

    let raw_input = renderer.state.take_egui_input(&g_ctx.window);

    context.begin_frame(raw_input);

    storages.add_unique(CurrentEguiFrame(context));
}

pub fn render_end(g_ctx: UniqueView<GraphicsContext>, mut renderer: UniqueViewMut<EguiRenderer>, mut rend_ctx: UniqueViewMut<RenderContext>, storages: AllStoragesView) {
    let Ok(CurrentEguiFrame(context)) = storages.remove_unique::<CurrentEguiFrame>() else {
        tracing::error!("Egui context doesn't exist to render to.");
        return;
    };

    let RenderContext { tex_view, encoder, .. } = rend_ctx.as_mut();

    let full_output = context.end_frame();

    renderer.state.handle_platform_output(&g_ctx.window, full_output.platform_output);

    let tris = context
        .tessellate(full_output.shapes, context.pixels_per_point());

    for (id, image_delta) in &full_output.textures_delta.set {
        renderer.renderer.update_texture(&g_ctx.device, &g_ctx.queue, *id, image_delta);
    }

    let screen_descriptor = make_screen_descriptor(&g_ctx);

    renderer.renderer.update_buffers(&g_ctx.device, &g_ctx.queue, encoder, &tris, &screen_descriptor);

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: tex_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        label: Some("egui main render pass"),
        occlusion_query_set: None,
    });

    renderer.renderer.render(&mut render_pass, &tris, &screen_descriptor);

    drop(render_pass);

    for x in &full_output.textures_delta.free {
        renderer.renderer.free_texture(x)
    }
}


fn make_screen_descriptor(g_ctx: &GraphicsContext) -> ScreenDescriptor {
    ScreenDescriptor {
        size_in_pixels: [g_ctx.config.width, g_ctx.config.height],
        pixels_per_point: g_ctx.window.scale_factor() as _,
    }
}