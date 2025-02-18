use shipyard::{AllStoragesView, IntoIter, Unique, UniqueView, View};
use crate::camera::Camera;
use crate::components::{LocalPlayer, Transform};
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::shader_cam::ShaderCam;

#[derive(Unique)]
pub struct CameraUniformBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl CameraUniformBuffer {
    pub fn new(g_ctx: &GraphicsContext) -> Self {
        // buffer to hold the camera matrix
        let buffer = g_ctx.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("camera_uniform_buffer"),
                // use the buffer in a uniform in a bind group, copy_dst -> it can be written to in bind group
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                size: size_of::<ShaderCam>() as _,
                mapped_at_creation: false,
            }
        );

        let bind_group_layout = g_ctx.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
            label: Some("camera_bind_group"),
        });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update_buffer(&self, g_ctx: &GraphicsContext, uniform: &ShaderCam) {
        g_ctx.queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(uniform));
    }
}

pub fn initialize_camera_uniform_buffer(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    storages.add_unique(CameraUniformBuffer::new(&g_ctx));
}

pub fn update_camera_uniform_buffer(
    g_ctx: UniqueView<GraphicsContext>,
    cam_uniform_buffer: UniqueView<CameraUniformBuffer>,
    v_local_player: View<LocalPlayer>,
    v_camera: View<Camera>,
    v_transform: View<Transform>,
) {
    let (_, render_cam, transform) = (&v_local_player, &v_camera, &v_transform)
        .iter()
        .next()
        .expect("TODO: local player did not have camera to render to");

    let shader_cam = ShaderCam::from_camera_and_transform(render_cam, transform).expect("view & proj matrices should be invertible");

    cam_uniform_buffer.update_buffer(&g_ctx, &shader_cam);
}