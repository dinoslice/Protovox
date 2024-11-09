use shipyard::{IntoIter, UniqueView, UniqueViewMut, View};
use game::block::Block;
use crate::components::{Entity, HeldBlock, LocalPlayer, Transform};
use crate::networking::server_handler::ServerHandler;
use crate::rendering::egui::EguiRenderer;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::render::RenderContext;

pub fn render_egui(
    mut ctx: UniqueViewMut<RenderContext>,
    g_ctx: UniqueView<GraphicsContext>,
    mut egui_renderer: UniqueViewMut<EguiRenderer>,

    // for player debug info
    v_local_player: View<LocalPlayer>,
    v_entity: View<Entity>,
    v_transform: View<Transform>,
    v_held_block: View<HeldBlock>,

    opt_server_handler: Option<UniqueView<ServerHandler>>,
) {
    let RenderContext { tex_view, encoder, .. } = ctx.as_mut();

    let pos_fmt = |v: &glm::Vec3| format!("Position: [{:.2}, {:.2}, {:.2}]", v.x, v.y, v.z);
    
    let (_, local_transform, held_block) = (&v_local_player, &v_transform, &v_held_block)
        .iter()
        .next()
        .expect("LocalPlayer didn't have transform & held block");

    let mut other_pos = (!&v_local_player, &v_entity, &v_transform).iter()
        .map(|e| &e.2.position)
        .peekable();

    egui_renderer.draw(&g_ctx, encoder, tex_view, |ctx| {
        egui::Window::new("Entities")
            .default_open(true)
            .show(ctx, |ui| {
                ui.heading("LocalPlayer");
                ui.label(pos_fmt(&local_transform.position));
                
                if other_pos.peek().is_some() {
                    ui.heading("Entities");
                    
                    for pos in other_pos {
                        ui.label(pos_fmt(pos));
                    }
                }
            });

        if let Some(server_handler) = opt_server_handler {
            egui::Window::new("ServerHandler")
                .default_open(true)
                .show(ctx, |ui| {
                    ui.label(format!("Address: {}", server_handler.local_addr));
                });
        }

        egui::Area::new("hotbar_box".into())
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(ui.visuals().window_fill())
                    .rounding(5.0)
                    .outer_margin(egui::Margin::same(5.0))
                    .inner_margin(egui::Margin::same(5.0))
                    .show(ui, |ui| {
                        ui.style_mut()
                            .text_styles
                            .get_mut(&egui::TextStyle::Body)
                            .expect("style to exist")
                            .size = 15.0;
                        
                        let hotbar_text = match held_block.0 {
                            Block::Air => "None".into(),
                            b => format!("{b:?}"),
                        };
                        
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.label(hotbar_text);
                        });
                    });
            });
    });
}