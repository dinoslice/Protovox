use egui::{Align2, Area, Color32, Frame, LayerId, Order, RichText, Stroke, Vec2};
use shipyard::{AllStoragesView, Unique, UniqueView, UniqueViewMut};
use egui_systems::CurrentEguiFrame;
use engine::application::exit::ExitRequested;
use engine::application::pause::{toggle_pause_menu, IsPaused, ToggleGuiPressed};
use engine::rendering::gui_bundle::GuiBundle;

pub fn draw_pause_menu(
    mut open: UniqueViewMut<IsPaused>,
    egui_frame: UniqueView<CurrentEguiFrame>,
    storages: AllStoragesView,
    gui_bundle: GuiBundle,
) {
    if !open.is_paused() {
        return;
    }

    let ctx = egui_frame.ctx();

    ctx.layer_painter(LayerId::new(Order::Background, "background_defocus".into()))
        .rect_filled(ctx.screen_rect(), 0.0, Color32::from_rgba_premultiplied(96, 96, 96, 128));

    Area::new("pause_area".into())
        .order(Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| Frame::window(&ctx.style())
            .fill(Color32::from_rgba_unmultiplied(64, 64, 64, 128))
            .stroke(Stroke::new(4.0, Color32::BLACK))
            .rounding(6.0)
            .inner_margin(10.0)
            .show(ui, |ui| ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Pause Menu")
                        .heading()
                        .color(Color32::from_gray(32))
                );

                if ui.button("Resume").clicked() {
                    storages.add_unique(ToggleGuiPressed);
                    toggle_pause_menu(storages, open, gui_bundle);
                    return;
                }

                if ui.button("Quit").clicked() {
                    storages.add_unique(ExitRequested);
                }
            }))
        );
}