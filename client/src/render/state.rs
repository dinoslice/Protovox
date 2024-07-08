use std::time::Duration;
use glm::Vec3;
use na::{Perspective3, Unit, UnitQuaternion};
use pollster::FutureExt;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::PhysicalKey;
use winit::window::Window;
use crate::render::camera::Camera;
use crate::input::InputManager;
use crate::render::instance::Instance;
use crate::render::texture::Texture;
use crate::render::vertex::Vertex;

pub struct State<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    // must be dropped after the device since it has unsafe references to the window
    pub window: &'a Window,

    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub diffuse_texture: Texture,

    pub camera: Camera,

    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,

    pub input_manager: InputManager,

    pub instances: Vec<Instance>,
    pub instance_buffer: wgpu::Buffer,

    pub depth_texture: Texture,
}

impl<'a> State<'a> {
    pub fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();


        // 1. establishing a connection to the GPU

        // handle to the GPU, interfaces with Vulkan, DX12, etc.; main purpose to create adapters and surfaces
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                .. Default::default()
            }
        );

        // surface where rendered frames can be presented, i.e. a window
        let surface = instance.create_surface(window).expect("failed to create surface");

        // handle to the actual GPU, can get info about GPU & create device & queue
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false, // switch to software system instead of hardware
                compatible_surface: Some(&surface), // find an adapter that supports the surface
            }
        ).block_on().expect("failed to request adapter");

        // device is an open connection to gpu, queue executes command buffers
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(), // what additional features of the GPU are needed
                required_limits: wgpu::Limits::default(), // limit properties of the gpu to support different architectures
                label: None,
            },
            None // trace path for api call tracing
        ).block_on().expect("failed to request device");

        // get list of formats, texture usages, presentation modes, etc.
        // presentation mode -> vsync, fifo, etc.
        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        // defines how the surface creates its underlying surface textures
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, // texture used as output of a render pass
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2, // max frames that should be queued in advance
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: Vec::new(),
        };


        // 2. load vertices & indices
        const VERTICES: &[Vertex] = &[
            Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], },
            Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], },
            Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], },
            Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], },
            Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], },
        ];

        // using indices eliminates duplicate vertices
        const INDICES: &[u16] = &[
            0, 1, 4,
            1, 2, 4,
            2, 3, 4,
        ];

        // holds vertices, available in shader
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let num_indices = INDICES.len() as u32;


        // 3. instancing to avoid duplicate meshes
        const NUM_INSTANCES_PER_ROW: u32 = 10;
        const INSTANCE_DISPLACEMENT: Vec3 = Vec3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);

        let instances: Vec<_> = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = Vec3::new(x as f32, 0.0, z as f32) - INSTANCE_DISPLACEMENT;

                let rotation = if position == Vec3::zeros() {
                    UnitQuaternion::from_axis_angle(&Vec3::z_axis(), 0.0)
                } else {
                    UnitQuaternion::from_axis_angle(&Unit::new_normalize(position), std::f32::consts::FRAC_PI_4)
                };

                Instance {
                    position, rotation,
                }
            })
        }).collect();

        let instance_data: Vec<_> = instances.iter().map(Instance::as_raw).collect();

        // buffer with matrices representing each instance's position & rotation
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX, // only needed in vertex buffer
            }
        );


        // 4. load textures into bind group
        let diffuse_texture = Texture::from_bytes(&device, &queue, include_bytes!("../../assets/tree.png"), "tree.png").unwrap();

        // bind group -> data constant through one draw call
        let diffuse_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let diffuse_bind_group = device.create_bind_group(
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


        // 5. camera to view scene from
        let camera = Camera {
            position: Vec3::new(0.0, 1.0, 2.0),
            yaw: -90.0f32.to_radians(),
            pitch: -20.0f32.to_radians(),
            speed: 4.0,
            perspective: Perspective3::new(
                config.width as f32 / config.height as f32,
                45.0f32.to_radians(),
                0.1,
                100.0
            )
        };

        let input_manager = InputManager::with_mouse_sensitivity(0.4);

        // buffer to hold the camera matrix
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&camera.as_uniform()),
                // use the buffer in a uniform in a bind group, copy_dst -> it can be written to in bind group
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        // bind group for the camera, resource is the whole contents of the buffer
        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX, // only need the camera in the vertex shader
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform, // var<uniform> in wgsl
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
            }
        );

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });


        // 6. pipeline / instructions for GPU

        // loads a shader and returns a handle to the compiled shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth texture");

        // pipeline describes the GPU's actions on a set of data, like a shader program
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&diffuse_bind_group_layout, &camera_bind_group_layout], // layouts of the bind groups, matches @group(n) in shader
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[ // format of the vertex buffers used, indices correspond to slot when setting the buffer before rendering
                    Vertex::buffer_desc(), Instance::buffer_desc()
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE), // blending, if set to replace this overwrites the contents
                    write_mask: wgpu::ColorWrites::ALL, // write to all channels (rgba)
                })],
            }),
            primitive: wgpu::PrimitiveState { // how to interpret vertices when converting to triangles
                topology: wgpu::PrimitiveTopology::TriangleList, // 3 vert per triangle
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
            surface,
            device,
            queue,
            config,
            size,
            window,
            pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_bind_group,
            diffuse_texture,
            camera,
            camera_buffer,
            camera_bind_group,
            instances,
            instance_buffer,
            depth_texture,
            input_manager,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.camera.perspective.set_aspect(self.config.width as f32 / self.config.height as f32);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(key),
                    state,
                    ..
                },
                ..
            } => self.input_manager.action_map.process_input(key, *state == ElementState::Pressed),
            WindowEvent::MouseWheel { delta, .. } => {
                self.input_manager.mouse_manager.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.input_manager.mouse_manager.pressed = *state == ElementState::Pressed;
                true
            }
            _ => false, // returns false if the event hasn't been processed, so it can be further processed later
        }
    }

    pub fn update(&mut self, delta_time: &Duration) {
        self.camera.update_with_input(&mut self.input_manager, delta_time);
        // update the camera buffer
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&self.camera.as_uniform()));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // get a surface texture to render to
        let output = self.surface.get_current_texture()?;

        // view of the texture, so we can control how the render code interacts with the texture
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // command encoder creates the commands to send to the GPU, commands stored in command buffer
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

        // bind group is data constant through the draw call, index is the @group(n) used to access in the shader
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

        // assign a vertex buffer to a slot, slot corresponds to the desc used when creating the pipeline, slice(..) to use whole buffer
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        // indices are u16, use the whole model defined by the indices with slice(..), only one index buffer at a time
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // draw the whole range of indices, and all instances
        render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);

        // finish the command buffer & submit to GPU
        drop(render_pass);
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}