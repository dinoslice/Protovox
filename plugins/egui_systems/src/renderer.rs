use egui::{epaint, Context};
use egui_wgpu::Renderer;
use egui_winit::State;
use shipyard::Unique;
use winit::event::WindowEvent;
use winit::window::Window;

#[derive(Unique)]
pub struct EguiRenderer {
    pub(crate) state: State,
    pub(crate) renderer: Renderer,
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
    
    pub fn register_native_texture(&mut self, device: &wgpu::Device, texture: &wgpu::TextureView, texture_filter: wgpu::FilterMode) -> epaint::TextureId {
        self.renderer.register_native_texture(device, texture, texture_filter)
    }
}