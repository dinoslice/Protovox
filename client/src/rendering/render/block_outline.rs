use shipyard::{IntoIter, UniqueView, UniqueViewMut, View};
use wgpu::util::RenderEncoder;
use game::chunk::location::ChunkLocation;
use crate::components::LocalPlayer;
use crate::looking_at_block::LookingAtBlock;
use crate::rendering::base_face::BaseFace;
use crate::rendering::block_outline::BlockOutlineRenderState;
use crate::rendering::camera_uniform_buffer::CameraUniformBuffer;
use crate::rendering::depth_texture::DepthTexture;
use crate::rendering::face_data::FaceData;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::render::RenderContext;
use crate::rendering::world::WorldRenderState;
use crate::rendering::sized_buffer::SizedBuffer;
use crate::rendering::texture_atlas::TextureAtlas;

pub fn render_block_outline(
    mut ctx: UniqueViewMut<RenderContext>,
    depth_texture: UniqueView<DepthTexture>,
    world_rend_state: UniqueView<WorldRenderState>,
    camera_uniform_buffer: UniqueViewMut<CameraUniformBuffer>,
    base_face: UniqueView<BaseFace>,
    texture_atlas: UniqueView<TextureAtlas>,
    block_outline_render_state: UniqueView<BlockOutlineRenderState>,

    v_local_player: View<LocalPlayer>,
    v_looking_at_block: View<LookingAtBlock>,
) {
    let RenderContext { tex_view, encoder, .. } = ctx.as_mut();
    
    let Some(raycast) = (&v_local_player, &v_looking_at_block)
        .iter()
        .next()
        .and_then(|(_, look_at)| look_at.0.as_ref())
    else {
        return
    };
    
    let chunk_loc = ChunkLocation::from(&raycast.hit_position);

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("block_outline_render_pass"),
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
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    pass.set_pipeline(&world_rend_state.pipeline);
    
    pass.set_bind_group(0, &texture_atlas.bind_group, &[]);
    pass.set_bind_group(1, &camera_uniform_buffer.bind_group, &[]);
    
    pass.set_vertex_buffer(0, base_face.buffer.buffer.slice(..));

    pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, bytemuck::cast_slice(chunk_loc.0.as_ref()));

    let SizedBuffer { buffer, size } = &block_outline_render_state.buffer;
    
    let buffer_end = *size as usize * size_of::<FaceData>();
    
    pass.set_vertex_buffer(1, buffer.slice(0..buffer_end as _));

    pass.draw(0..base_face.buffer.size, 0..*size);
}