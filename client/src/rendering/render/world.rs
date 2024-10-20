use shipyard::{UniqueView, UniqueViewMut};
use crate::chunks::chunk_manager::ChunkManager;
use crate::rendering::base_face::BaseFace;
use crate::rendering::camera_uniform_buffer::CameraUniformBuffer;
use crate::rendering::depth_texture::DepthTexture;
use crate::rendering::render::RenderContext;
use crate::rendering::renderer::RenderPipeline;
use crate::rendering::texture_atlas::TextureAtlas;

pub fn render_world(
    mut ctx: UniqueViewMut<RenderContext>,
    depth_texture: UniqueView<DepthTexture>,
    pipeline: UniqueView<RenderPipeline>,
    camera_uniform_buffer: UniqueViewMut<CameraUniformBuffer>,
    base_face: UniqueView<BaseFace>,
    texture_atlas: UniqueView<TextureAtlas>,
    chunk_manager: UniqueView<ChunkManager>,
) {
    let RenderContext { tex_view, encoder, .. } = ctx.as_mut();

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("world_render_pass"),
        color_attachments: &[
            // @location(0) in output of fragment shader
            Some(wgpu::RenderPassColorAttachment { // where to draw color to
                view: tex_view, // save the color texture view accessed earlier
                resolve_target: None, // texture to received resolved output, same as view unless multisampling
                ops: wgpu::Operations { // what to do with the colors on the view
                    load: wgpu::LoadOp::Clear(wgpu::Color { // clears the screen with a color
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
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

    pass.set_pipeline(&pipeline.0);

    // bind group is data constant through the draw call, index is the @group(n) used to access in the shader
    pass.set_bind_group(0, &texture_atlas.bind_group, &[]);
    pass.set_bind_group(1, &camera_uniform_buffer.bind_group, &[]);

    // assign a vertex buffer to a slot, slot corresponds to the desc used when creating the pipeline, slice(..) to use whole buffer
    pass.set_vertex_buffer(0, base_face.vertex_buffer.slice(..));

    let bakery = chunk_manager.baked_chunks();

    for chunk_loc in chunk_manager.loaded_locations() {
        let Some(buffer) = bakery.get(&chunk_loc) else {
            continue;
        };

        pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, bytemuck::cast_slice(chunk_loc.0.as_ref()));

        pass.set_vertex_buffer(1, buffer.buffer.slice(..));

        // draw the whole range of vertices, and all instances
        pass.draw(0..base_face.num_vertices, 0..buffer.size);
    }
}