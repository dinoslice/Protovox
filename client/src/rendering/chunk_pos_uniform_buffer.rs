use crate::rendering::graphics_context::GraphicsContext;

pub struct ChunkPosUniformBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl ChunkPosUniformBuffer {
    pub fn new(g_ctx: &GraphicsContext) -> Self {
        let buffer = g_ctx.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("chunk_pos_uniform_buffer"),
                size: std::mem::size_of::<[i32; 3]>() as _,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let bind_group_layout = g_ctx.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("chunk_pos_uniform_bind_group_layout"),
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

        // bind group for the camera uniform, resource is the whole contents of the buffer
        let bind_group = g_ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("chunk_pos_uniform_bind_group"),
        });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update_buffer(&self, g_ctx: &GraphicsContext, chunk_pos: &[f32; 3]) {
        g_ctx.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(chunk_pos));
    }
}