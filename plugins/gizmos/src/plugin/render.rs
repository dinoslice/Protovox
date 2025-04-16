use shipyard::{UniqueView, UniqueViewMut};
use engine::rendering::camera_uniform_buffer::CameraUniformBuffer;
use engine::rendering::depth_texture::DepthTexture;
use engine::rendering::render::RenderContext;
use crate::plugin::line_render_state::GizmosLineRenderState;

pub fn render_line_gizmos(
    mut ctx: UniqueViewMut<RenderContext>,
    gizmos_line_render_state: UniqueView<GizmosLineRenderState>,
    depth_texture: UniqueView<DepthTexture>,
    camera_uniform_buffer: UniqueViewMut<CameraUniformBuffer>
) {
    let RenderContext { multisample_view, tex_view, encoder, .. } = ctx.as_mut();

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("line_gizmos_render_pass"),
        color_attachments: &[
            // @location(0) in output of fragment shader
            Some(wgpu::RenderPassColorAttachment { // where to draw color to
                view: multisample_view, // save the color texture view accessed earlier
                resolve_target: Some(tex_view), // texture to received resolved output, same as view unless multisampling
                ops: wgpu::Operations { // what to do with the colors on the view
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store, // store the result of this pass, don't discard it
                },
            })
        ],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth_texture.0.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    pass.set_pipeline(&gizmos_line_render_state.pipeline);
    pass.set_bind_group(0, &camera_uniform_buffer.bind_group, &[]);
    pass.set_vertex_buffer(0, gizmos_line_render_state.sized_buffer.buffer.slice(..));
    pass.draw(0..gizmos_line_render_state.sized_buffer.size, 0..1);
}