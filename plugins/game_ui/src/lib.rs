use shipyard::{IntoIter, IntoWorkload, UniqueView, View, Workload};
use engine::components::{Entity, HeldBlock, LocalPlayer, SpectatorSpeed, Transform, Velocity};
use engine::gamemode::Gamemode;
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePluginMetadata};
use egui_systems::CurrentEguiFrame;
use engine::networking::server_handler::ServerHandler;
use egui_systems::DuringEgui;
use game::block::Block;

extern crate nalgebra_glm as glm;
mod egui_views;

pub struct GameUiPlugin;

impl DinoEnginePlugin for GameUiPlugin {
    fn render(&self) -> Option<Workload> {
        game_ui
            .into_workload()
            .order_egui()
            .into()
    }
    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "game_ui".ck().expect("valid identifier"),
            version: "0.1.0",
            dependencies: &[
                &engine::VoxelEngine,
                &egui_systems::EguiSystemsPlugin,
            ],
        }
    }
}

pub fn game_ui(
    egui_frame: UniqueView<CurrentEguiFrame>,

    // for player debug info
    (v_local_player, v_entity): (View<LocalPlayer>, View<Entity>),
    (v_transform, v_velocity): (View<Transform>, View<Velocity>),
    v_held_block: View<HeldBlock>,
    (v_gamemode, v_spectator_speed): (View<Gamemode>, View<SpectatorSpeed>),

    opt_server_handler: Option<UniqueView<ServerHandler>>,
) {
    let ctx = egui_frame.ctx();

    let vec3_fmt = |title: &'static str, v: &glm::Vec3| format!("{title}: [{:.2}, {:.2}, {:.2}]", v.x, v.y, v.z);

    let (_, local_transform, velocity, held_block, gamemode, spec_speed) = (&v_local_player, &v_transform, &v_velocity, &v_held_block, &v_gamemode, &v_spectator_speed)
        .iter()
        .next()
        .expect("LocalPlayer didn't have transform & held block");

    let mut other_pos = (!&v_local_player, &v_entity, &v_transform).iter()
        .map(|e| &e.2.position)
        .peekable();

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

    // reticle
    ctx.layer_painter(egui::LayerId::background())
        .circle_filled(
            ctx.screen_rect().center(),
            2.5,
            egui::Color32::from_rgba_premultiplied(192, 192, 192, 128),
        );
}