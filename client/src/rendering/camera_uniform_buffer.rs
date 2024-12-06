use bytemuck::{Pod, Zeroable};
use glm::Mat4;
use shipyard::{AllStoragesView, IntoIter, Unique, UniqueView, View};
use wgpu::util::DeviceExt;
use crate::camera::Camera;
use crate::components::{LocalPlayer, Transform};
use crate::rendering::graphics_context::GraphicsContext;

#[derive(Unique)]
pub struct CameraUniformBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl CameraUniformBuffer {
    pub fn new(g_ctx: &GraphicsContext) -> Self {
        Self::new_with_initial_buffer(g_ctx, &[0.0; (4 * 4 * 4)])
    }

    // TODO: don't initialize?
    pub fn new_with_initial_buffer(g_ctx: &GraphicsContext, initial_uniform: &[f32]) -> Self {
        // buffer to hold the camera matrix
        let buffer = g_ctx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera_uniform_buffer"),
                contents: bytemuck::cast_slice(initial_uniform),
                // use the buffer in a uniform in a bind group, copy_dst -> it can be written to in bind group
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group_layout = g_ctx.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::all(), // only need the camera in the vertex shader
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

    pub fn update_buffer(&self, g_ctx: &GraphicsContext, uniform: &[f32]) {
        g_ctx.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(uniform));
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

    #[repr(C)]
    #[derive(Pod, Zeroable, Copy, Clone)]
    struct ShaderCamera {
        view: [[f32; 4]; 4],
        view_proj: [[f32; 4]; 4],
        inv_proj: [[f32; 4]; 4],
        inv_view: [[f32; 4]; 4]
    }

    let view_mat = render_cam.view_matrix(transform.position, transform.pitch, transform.yaw);
    let view_inv = view_mat.try_inverse().expect("TODO: handler error better");
    let proj_mat = render_cam.perspective.as_matrix();
    let inv_proj = proj_mat.try_inverse().expect("TODO: handle error better");
    let view_proj = proj_mat * view_mat;

    let camera = ShaderCamera {
        view: view_mat.into(),
        view_proj: view_proj.into(),
        inv_proj: inv_proj.into(),
        inv_view: view_inv.into()
    };

    cam_uniform_buffer.update_buffer(&g_ctx, bytemuck::try_cast_slice(&[camera]).expect("camera is too long"));
}