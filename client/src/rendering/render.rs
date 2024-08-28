use glm::TVec3;
use shipyard::{UniqueView, UniqueViewMut};
use game::chunk::location::ChunkLocation;
use crate::rendering::chunk_mesh::ChunkMesh;
use crate::rendering::face_data::FaceData;
use crate::rendering::base_face::BaseFace;
use crate::camera::Camera;
use crate::chunks::chunk_manager::ChunkManager;
use crate::rendering::camera_uniform_buffer::CameraUniformBuffer;
use crate::rendering::depth_texture::DepthTexture;
use crate::rendering::face_buffer::FaceBuffer;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::renderer::RenderPipeline;
use crate::rendering::texture_atlas::TextureAtlas;

pub fn render(
    g_ctx: UniqueView<GraphicsContext>,
    depth_texture: UniqueView<DepthTexture>,
    pipeline: UniqueView<RenderPipeline>,
    camera: UniqueView<Camera>,
    camera_uniform_buffer: UniqueViewMut<CameraUniformBuffer>,
    base_face: UniqueView<BaseFace>,
    texture_atlas: UniqueView<TextureAtlas>,
    chunk_mesh: UniqueView<ChunkMesh>,
    face_buffer: UniqueView<FaceBuffer>
) -> Result<(), wgpu::SurfaceError> {
    let chunk_targets = &[
        (ChunkLocation(TVec3::new(0, 0, 0)), chunk_mesh.faces.as_slice()),
    ];

    // get a surface texture to render to
    let output = g_ctx.surface.get_current_texture()?;

    // view of the texture, so we can control how the render code interacts with the texture
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    // command encoder creates the commands to send to the GPU, commands stored in command buffer
    let mut encoder = g_ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render encoder"),
    });

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[
            // @location(0) in output of fragment shader
            Some(wgpu::RenderPassColorAttachment { // where to draw color to
                view: &view, // save the color texture view accessed earlier
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

    render_pass.set_pipeline(&pipeline.0);

    // update the camera buffer
    camera_uniform_buffer.update_buffer(&g_ctx, &camera.as_uniform());

    // bind group is data constant through the draw call, index is the @group(n) used to access in the shader
    render_pass.set_bind_group(0, &texture_atlas.bind_group, &[]);
    render_pass.set_bind_group(1, &camera_uniform_buffer.bind_group, &[]);

    // assign a vertex buffer to a slot, slot corresponds to the desc used when creating the pipeline, slice(..) to use whole buffer
    render_pass.set_vertex_buffer(0, base_face.vertex_buffer.slice(..));

    let mut offset = 0;

    for (chunk_loc, faces) in chunk_targets.iter().filter(|(_, f)| !f.is_empty()) {
        let chunk_faces_size_bytes = faces.len() as u64 * std::mem::size_of::<FaceData>() as u64;

        g_ctx.queue.write_buffer(&face_buffer.0, offset, bytemuck::cast_slice(faces));

        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, bytemuck::cast_slice(chunk_loc.0.as_ref()));

        render_pass.set_vertex_buffer(1, face_buffer.0.slice(
            offset..offset + chunk_faces_size_bytes
        ));

        // draw the whole range of vertices, and all instances
        render_pass.draw(0..base_face.num_vertices, 0..faces.len() as _);
        offset += chunk_faces_size_bytes;
    }


    // finish the command buffer & submit to GPU
    drop(render_pass);
    g_ctx.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}

pub fn render_new(
    g_ctx: UniqueView<GraphicsContext>,
    depth_texture: UniqueView<DepthTexture>,
    pipeline: UniqueView<RenderPipeline>,
    camera: UniqueView<Camera>,
    camera_uniform_buffer: UniqueViewMut<CameraUniformBuffer>,
    base_face: UniqueView<BaseFace>,
    texture_atlas: UniqueView<TextureAtlas>,
    chunk_manager: UniqueView<ChunkManager>,
) -> Result<(), wgpu::SurfaceError> {
    // get a surface texture to render to
    let output = g_ctx.surface.get_current_texture()?;

    // view of the texture, so we can control how the render code interacts with the texture
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    // command encoder creates the commands to send to the GPU, commands stored in command buffer
    let mut encoder = g_ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render encoder"),
    });

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[
            // @location(0) in output of fragment shader
            Some(wgpu::RenderPassColorAttachment { // where to draw color to
                view: &view, // save the color texture view accessed earlier
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

    render_pass.set_pipeline(&pipeline.0);

    // update the camera buffer
    camera_uniform_buffer.update_buffer(&g_ctx, &camera.as_uniform());

    // bind group is data constant through the draw call, index is the @group(n) used to access in the shader
    render_pass.set_bind_group(0, &texture_atlas.bind_group, &[]);
    render_pass.set_bind_group(1, &camera_uniform_buffer.bind_group, &[]);

    // assign a vertex buffer to a slot, slot corresponds to the desc used when creating the pipeline, slice(..) to use whole buffer
    render_pass.set_vertex_buffer(0, base_face.vertex_buffer.slice(..));

    let bakery = chunk_manager.baked_chunks();

    for chunk_loc in chunk_manager.loaded_locations() {
        let Some(buffer) = bakery.get(&chunk_loc) else {
            continue;
        };

        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, bytemuck::cast_slice(chunk_loc.0.as_ref()));

        render_pass.set_vertex_buffer(1, buffer.buffer.slice(
            // 0..buffer.size
            ..
        ));

        // draw the whole range of vertices, and all instances
        render_pass.draw(0..base_face.num_vertices, 0..buffer.size);
    }


    // finish the command buffer & submit to GPU
    drop(render_pass);
    g_ctx.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}