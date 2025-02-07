use egui::Context;
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use shipyard::{AllStoragesView, Unique, UniqueView};
use winit::event::WindowEvent;
use winit::window::Window;
use engine::rendering::graphics_context::GraphicsContext;

#[derive(Unique)]
pub struct EguiRenderer {
    pub state: State,
    pub renderer: Renderer,
}

impl EguiRenderer {
    pub fn context(&self) -> &Context {
        self.state.egui_ctx()
    }

    pub fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        output_depth_format: Option<wgpu::TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> EguiRenderer {
        let egui_context = Context::default();

        let egui_state = State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );
        let egui_renderer = Renderer::new(
            device,
            output_color_format,
            output_depth_format,
            msaa_samples,
        );

        EguiRenderer {
            state: egui_state,
            renderer: egui_renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn draw(&mut self, g_ctx: &GraphicsContext, encoder: &mut wgpu::CommandEncoder, window_surface_view: &wgpu::TextureView, run_ui: impl FnOnce(&Context)) {
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [g_ctx.config.width, g_ctx.config.height],
            pixels_per_point: g_ctx.window.scale_factor() as _,
        };

        self.context().set_pixels_per_point(screen_descriptor.pixels_per_point);

        let raw_input = self.state.take_egui_input(&g_ctx.window);

        let full_output = self.context().run(raw_input, run_ui);

        self.state.handle_platform_output(&g_ctx.window, full_output.platform_output);

        let tris = self.context()
            .tessellate(full_output.shapes, self.context().pixels_per_point());

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(&g_ctx.device, &g_ctx.queue, *id, image_delta);
        }

        self.renderer.update_buffers(&g_ctx.device, &g_ctx.queue, encoder, &tris, &screen_descriptor);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            label: Some("egui main render pass"),
            occlusion_query_set: None,
        });

        self.renderer.render(&mut render_pass, &tris, &screen_descriptor);

        drop(render_pass);

        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}