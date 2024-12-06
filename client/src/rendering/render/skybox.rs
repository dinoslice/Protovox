use shipyard::{UniqueView, UniqueViewMut};
use crate::rendering::camera_uniform_buffer::CameraUniformBuffer;
use crate::rendering::render::RenderContext;
use crate::rendering::skybox::Skybox;

pub fn render_skybox(
    mut ctx: UniqueViewMut<RenderContext>,
    camera_uniform_buffer: UniqueView<CameraUniformBuffer>,
    skybox: UniqueView<Skybox>
) {
    let RenderContext { tex_view, encoder, .. } = ctx.as_mut();

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("skybox render pass"),
        color_attachments: &[
            // @location(0) in output of fragment shader
            Some(wgpu::RenderPassColorAttachment { // where to draw color to
                view: tex_view, // save the color texture view accessed earlier
                resolve_target: None, // texture to received resolved output, same as view unless multisampling
                ops: wgpu::Operations { // what to do with the colors on the view
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 1.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store, // store the result of this pass, don't discard it
                },
            })
        ],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    pass.set_pipeline(&skybox.render_pipeline);

    pass.set_bind_group(0, &skybox.bind_group, &[]);
    pass.set_bind_group(1, &camera_uniform_buffer.bind_group, &[]);

    pass.draw(0..3, 0..1);
}