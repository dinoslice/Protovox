use egui::Ui;
use glm::TVec3;
use shipyard::{IntoIter, IntoWorkload, IntoWorkloadTrySystem, UniqueView, UniqueViewMut, View, Workload};
use game::location::WorldLocation;
use crate::components::{LocalPlayer, SpectatorSpeed, Transform, Velocity};
use crate::rendering::{camera_uniform_buffer, EguiRenderer};
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::render::{create_new_render_context, gizmos, submit_rendered_frame, world, RenderContext};
use crate::world_gen::WorldGenerator;
use crate::world_gen_debugger::params::WorldGenVisualizerParams;
use crate::world_gen_debugger::spline_editor::SplineEditor;

pub fn render() -> Workload {
    (
        (
            camera_uniform_buffer::update_camera_uniform_buffer,
            create_new_render_context
                .into_workload_try_system()
                .expect("failed to convert to try_system?"),
        ).into_workload(),
        (
            world::render_world,
            gizmos::render_line_gizmos,
            render_egui,
            submit_rendered_frame,
        ).into_sequential_workload()
    ).into_sequential_workload()
}

pub fn render_egui(
    mut ctx: UniqueViewMut<RenderContext>,
    g_ctx: UniqueView<GraphicsContext>,
    mut egui_renderer: UniqueViewMut<EguiRenderer>,

    // for player debug info
    v_local_player: View<LocalPlayer>,
    v_transform: View<Transform>,
    v_velocity: View<Velocity>,
    v_spectator_speed: View<SpectatorSpeed>,

    world_gen: UniqueView<WorldGenerator>,

    mut spline: UniqueViewMut<SplineEditor>,

    mut vis_params: UniqueViewMut<WorldGenVisualizerParams>,
) {
    let RenderContext { tex_view, encoder, .. } = ctx.as_mut();

    let vec3_fmt = |title: &'static str, v: &glm::Vec3| format!("{title}: [{:.2}, {:.2}, {:.2}]", v.x, v.y, v.z);

    let (local_transform, velocity, spec_speed, ..) = (&v_transform, &v_velocity, &v_spectator_speed, &v_local_player)
        .iter()
        .next()
        .expect("LocalPlayer didn't have transform & held block");

    egui_renderer.draw(&g_ctx, encoder, tex_view, |ctx| {
        egui::Window::new("Debug")
            .default_open(true)
            .show(ctx, |ui| {
                ui.heading("LocalPlayer");
                ui.label(vec3_fmt("Position", &local_transform.position));
                ui.label(vec3_fmt("Velocity", &velocity.0));
            });

        egui::Window::new("Spline Editor")
            .resizable(true)
            .show(ctx, |ui| spline.ui(ui));

        egui::Window::new("BlockData")
            .default_open(true)
            .show(ctx, |ui| {
                ui.label(format!("{:#?}", world_gen.biome_generator.generate_block_data(&WorldLocation(local_transform.position))));
            });

        egui::Window::new("Visualization Parameters")
            .show(ctx, |ui| {
                ui.add(vis_params.as_mut())
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



                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui|
                            ui.label(format!("Speed: {:.2}", spec_speed.curr_speed))
                        );
                    });
            });
    });
}