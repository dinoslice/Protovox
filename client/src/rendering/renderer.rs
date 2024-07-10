use glm::Vec3;
use na::{Unit, UnitQuaternion};
use rand::Rng;
use wgpu::util::DeviceExt;
use winit::window::Window;
use server::block::Block;
use server::chunk::data::ChunkData;
use server::chunk::pos::ChunkPos;
use crate::camera::Camera;
use crate::rendering::camera_uniform_buffer::CameraUniformBuffer;
use crate::rendering::face_data::{FaceData, FaceType};
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::instance::Instance;
use crate::rendering::texture::Texture;
use crate::rendering::vertex::Vertex;

pub struct Renderer<'a> {
    pub graphics_context: GraphicsContext<'a>,
    pub pipeline: wgpu::RenderPipeline,

    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,

    pub face_buffer: wgpu::Buffer,
    pub num_faces: u32,

    pub diffuse_texture: Texture,
    pub diffuse_bind_group: wgpu::BindGroup,

    pub camera_uniform_buffer: CameraUniformBuffer,

    pub depth_texture: Texture,
}

impl<'a> Renderer<'a> {
    pub fn new(window: &'a Window) -> Self {
        // 1. establishing a connection to the GPU
        let graphics_context = GraphicsContext::new(window);

        // TODO: each vertex can be compressed into 5 bits
        let base_face = &[
            Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [0.0, 0.0, 1.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0] },
            Vertex { position: [1.0, 0.0, 1.0], tex_coords: [1.0, 1.0] },
        ];

        // holds vertices, available in shader
        let vertex_buffer = graphics_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(base_face),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let num_vertices = base_face.len() as _;


        // TODO: this can be reduced once culling is implemented
        const FACE_BUFFER_MAX_SIZE: u64 = std::mem::size_of::<FaceData>() as u64 * 6 * 32 * 64 * 32;

        let face_buffer = graphics_context.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Face Buffer"),
                size: FACE_BUFFER_MAX_SIZE,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // only needed in vertex buffer,
                mapped_at_creation: false,
            }
        );

        let num_faces = 0;


        // 4. load textures into bind group
        let diffuse_texture = Texture::from_bytes(&graphics_context.device, &graphics_context.queue, include_bytes!("../../assets/cobblestone.png"), "cobblestone.png").unwrap();

        // bind group -> data constant through one draw call
        let diffuse_bind_group_layout =
            graphics_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0, // corresponds to @binding(n) in the shader
                        visibility: wgpu::ShaderStages::FRAGMENT, // use this bind group in the fragment shader
                        ty: wgpu::BindingType::Texture { // it's a texture, texture_2d<f32> in shader
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2, // _2d
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // sampler in shader
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), // sample type for texture must be filterable
                        count: None, // not an array
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = graphics_context.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &diffuse_bind_group_layout, // layout defined above
                entries: &[ // matches the entries defined above
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view), // assign the data into the bind group
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        let camera_uniform_buffer = CameraUniformBuffer::new(&graphics_context);

        // 5. pipeline / instructions for GPU

        // loads a shader and returns a handle to the compiled shader
        let shader = graphics_context.device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let depth_texture = Texture::create_depth_texture(&graphics_context.device, &graphics_context.config, "depth texture");

        // pipeline describes the GPU's actions on a set of data, like a shader program
        let render_pipeline_layout = graphics_context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&diffuse_bind_group_layout, &camera_uniform_buffer.bind_group_layout], // layouts of the bind groups, matches @group(n) in shader
            push_constant_ranges: &[],
        });

        let pipeline = graphics_context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[ // format of the vertex buffers used, indices correspond to slot when setting the buffer before rendering
                    Vertex::buffer_desc(), FaceData::buffer_desc()
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: graphics_context.config.format,
                    blend: Some(wgpu::BlendState::REPLACE), // blending, if set to replace this overwrites the contents
                    write_mask: wgpu::ColorWrites::ALL, // write to all channels (rgba)
                })],
            }),
            primitive: wgpu::PrimitiveState { // how to interpret vertices when converting to triangles
                topology: wgpu::PrimitiveTopology::TriangleStrip, // 3 vert per triangle
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // counter-clockwise ordered faces are front
                cull_mode: Some(wgpu::Face::Back), // backface culling
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // draw pixels front to back based on the depth texture
                stencil: wgpu::StencilState::default(), // usually stored in same texture as depth texture
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1, // multisampling
                mask: !0, // use all samples
                alpha_to_coverage_enabled: false, // for anti-aliasing
            },
            multiview: None, // for rendering to array textures
        });

        Self {
            graphics_context,
            pipeline,
            vertex_buffer,
            num_vertices,
            face_buffer,
            num_faces,
            diffuse_texture,
            diffuse_bind_group,
            camera_uniform_buffer,
            depth_texture,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.graphics_context.resize(new_size);
            self.depth_texture = Texture::create_depth_texture(&self.graphics_context.device, &self.graphics_context.config, "depth_texture");
        }
    }

    pub fn reconfigure(&mut self) {
        self.graphics_context.resize(self.graphics_context.size);
    }

    pub fn aspect(&self) -> f32 {
        self.graphics_context.config.width as f32 / self.graphics_context.config.height as f32
    }

    pub fn render(&mut self, camera: &Camera, faces: &[FaceData]) -> Result<(), wgpu::SurfaceError> {
        // get a surface texture to render to
        let output = self.graphics_context.surface.get_current_texture()?;

        // view of the texture, so we can control how the render code interacts with the texture
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // command encoder creates the commands to send to the GPU, commands stored in command buffer
        let mut encoder = self.graphics_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render encoder"),
        });

        // TODO: make faces take an option, which doesn't update if None
        self.graphics_context.queue.write_buffer(&self.face_buffer, 0, bytemuck::cast_slice(faces));
        self.num_faces = faces.len() as _;

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
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.pipeline);

        // update the camera buffer
        self.camera_uniform_buffer.update_buffer(&self.graphics_context, &camera.as_uniform());

        // bind group is data constant through the draw call, index is the @group(n) used to access in the shader
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_uniform_buffer.bind_group, &[]);

        // assign a vertex buffer to a slot, slot corresponds to the desc used when creating the pipeline, slice(..) to use whole buffer
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.face_buffer.slice(0..self.num_faces as u64 * std::mem::size_of::<FaceData>() as u64));

        // draw the whole range of vertices, and all instances
        render_pass.draw(0..self.num_vertices, 0..self.num_faces);

        // finish the command buffer & submit to GPU
        drop(render_pass);
        self.graphics_context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}