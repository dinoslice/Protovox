use shipyard::{IntoIter, IntoWorkload, IntoWorkloadTrySystem, UniqueView, UniqueViewMut, View, Workload};
use game::location::WorldLocation;
use crate::components::{LocalPlayer, Transform};
use crate::rendering::{camera_uniform_buffer, EguiRenderer};
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::render::{create_new_render_context, gizmos, submit_rendered_frame, world, RenderContext};
use crate::world_gen::WorldGenerator;
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

    world_gen: UniqueView<WorldGenerator>,

    mut spline: UniqueViewMut<SplineEditor>
) {
    let RenderContext { tex_view, encoder, .. } = ctx.as_mut();

    let pos_fmt = |v: &glm::Vec3| format!("Position: [{:.2}, {:.2}, {:.2}]", v.x, v.y, v.z);

    let local_pos = (&v_local_player, &v_transform)
        .iter()
        .next()
        .expect("LocalPlayer didn't have transform")
        .1
        .position;

    egui_renderer.draw(&g_ctx, encoder, tex_view, |ctx| {
        egui::Window::new("Debug")
            .default_open(true)
            .show(ctx, |ui| {
                ui.heading("LocalPlayer");
                ui.label(pos_fmt(&local_pos));
            });

        egui::Window::new("Spline Editor")
            .resizable(true)
            .show(ctx, |ui| spline.ui(ui));

        egui::Window::new("BlockData")
            .default_open(true)
            .show(ctx, |ui| {
                ui.label(format!("{:#?}", world_gen.biome_generator.generate_block_data(&WorldLocation(local_pos))));
            });
    });
}