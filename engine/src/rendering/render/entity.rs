use crate::rendering::camera_uniform_buffer::CameraUniformBuffer;
use crate::rendering::depth_texture::DepthTexture;
use crate::rendering::entity::EntityRenderState;
use crate::rendering::render::RenderContext;
use shipyard::{UniqueView, UniqueViewMut};

pub fn render_entity(
    mut ctx: UniqueViewMut<RenderContext>,
    depth_texture: UniqueView<DepthTexture>,
    entity_rend_state: UniqueView<EntityRenderState>,
    camera_uniform_buffer: UniqueViewMut<CameraUniformBuffer>,
) {
    let RenderContext { tex_view, encoder, .. } = ctx.as_mut();

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("entity_render_pass"),
        color_attachments: &[
            // @location(0) in output of fragment shader
            Some(wgpu::RenderPassColorAttachment { // where to draw color to
                view: tex_view, // save the color texture view accessed earlier
                resolve_target: None, // texture to received resolved output, same as view unless multisampling
                ops: wgpu::Operations { // what to do with the colors on the view
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store, // store the result of this pass, don't discard it
                },
            })
        ],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth_texture.0.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    pass.set_pipeline(&entity_rend_state.pipeline);

    // bind group is data constant through the draw call, index is the @group(n) used to access in the shader
    pass.set_bind_group(0, &camera_uniform_buffer.bind_group, &[]);

    // assign a vertex buffer to a slot, slot corresponds to the desc used when creating the pipeline, slice(..) to use whole buffer
    pass.set_vertex_buffer(0, entity_rend_state.base_face.buffer.slice(..));
}