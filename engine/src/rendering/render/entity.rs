use crate::rendering::camera_uniform_buffer::CameraUniformBuffer;
use crate::rendering::depth_texture::DepthTexture;
use crate::rendering::entity::EntityRenderState;
use crate::rendering::render::RenderContext;
use shipyard::{Get, IntoIter, IntoWithId, UniqueView, UniqueViewMut, View};
use wgpu::ShaderStages;
use wgpu::util::RenderEncoder;
use crate::components::Transform;
use crate::entity::model::Model;
use crate::entity::ModelView;
use crate::rendering::model_render::ModelMap;

pub fn render_entity(
    mut ctx: UniqueViewMut<RenderContext>,
    depth_texture: UniqueView<DepthTexture>,
    entity_rend_state: UniqueView<EntityRenderState>,
    camera_uniform_buffer: UniqueViewMut<CameraUniformBuffer>,
    model_map: UniqueView<ModelMap>,
    v_model: View<ModelView>,
    v_transform: View<Transform>,
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
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    pass.set_pipeline(&entity_rend_state.pipeline);

    pass.set_bind_group(0, &camera_uniform_buffer.bind_group, &[]);
    pass.set_bind_group(1, &model_map.bind_group, &[]);

    for (id, model) in v_model.iter().with_id() {
        let Ok(transform) = v_transform.get(id) else {
            tracing::warn!("No transform was associated with entity model. Only models with transforms attached will be rendered.");
            continue;
        };

        let Some((model, len)) = model_map.map.get(&model.0) else {
            continue;
        };

        for part in model.iter() {
            if let Some(buffer) = &part.buffer {
                let mut data = <Transform as Into<[f32; 9]>>::into(transform.clone()).to_vec();
                data.push(*len as f32);
                pass.set_push_constants(ShaderStages::VERTEX, 0, bytemuck::cast_slice(&data));

                pass.set_vertex_buffer(0, buffer.slice(..));

                pass.draw(0..buffer.size, 0..1);
            }
        }
    }
}