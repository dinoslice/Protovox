use shipyard::{IntoIter, UniqueView, UniqueViewMut, View};
use game::block::Block;
use crate::components::{Entity, HeldBlock, LocalPlayer, SpectatorSpeed, Transform, Velocity};
use crate::gamemode::Gamemode;
use crate::networking::chat::{ChatRecord, CurrentChatInput};
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
    v_velocity: View<Velocity>,
    v_held_block: View<HeldBlock>,
    (v_gamemode, v_spectator_speed, mut current_chat_msg, mut chat_log): (View<Gamemode>, View<SpectatorSpeed>, UniqueViewMut<CurrentChatInput>, UniqueViewMut<ChatRecord>),

    opt_server_handler: Option<UniqueView<ServerHandler>>,
) {
    let RenderContext { tex_view, encoder, .. } = ctx.as_mut();

    let vec3_fmt = |title: &'static str, v: &glm::Vec3| format!("{title}: [{:.2}, {:.2}, {:.2}]", v.x, v.y, v.z);
    
    let (_, local_transform, velocity, held_block, gamemode, spec_speed) = (&v_local_player, &v_transform, &v_velocity, &v_held_block, &v_gamemode, &v_spectator_speed)
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
                ui.label(vec3_fmt("Position", &local_transform.position));
                ui.label(vec3_fmt("Velocity", &velocity.0));
                
                if other_pos.peek().is_some() {
                    ui.heading("Entities");
                    
                    for pos in other_pos {
                        ui.label(vec3_fmt("Position", pos));
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

        egui::Area::new("chat_box".into())
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                let style = ui.style_mut();

                style.visuals.window_fill = egui::Color32::TRANSPARENT;
                style.visuals.override_text_color = Some(egui::Color32::WHITE);
                ui.set_width(100.0);

                ui.vertical(|ui| {
                    for (sender, message) in chat_log.record.iter().rev().take(10).rev() {
                        ui.label(format!("{}: {}", sender, message));
                    }
                });
                ui.horizontal(|ui| {
                    let text_edit = ui.text_edit_singleline(&mut current_chat_msg.0);
                    if (text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) || ui.button("Submit Message").clicked() {
                        text_edit.request_focus();
                        chat_log.unsent.push(current_chat_msg.0.clone());
                        current_chat_msg.0.clear();
                    }
                })
            });

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
                            .size = 17.5;

                        

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| match gamemode {
                            Gamemode::Survival => {
                                let hotbar_text = match held_block.0 {
                                    Block::Air => "None".into(),
                                    b => format!("{b:?}"),
                                };
                                
                                ui.label(hotbar_text);
                            }
                            Gamemode::Spectator => {
                                ui.label(format!("Speed: {:.2}", spec_speed.curr_speed));
                            }
                        });
                    });
            });
    });
}