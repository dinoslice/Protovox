use egui::{Align2, Area, Color32, Frame, LayerId, Order, RichText, Stroke, Vec2};
use shipyard::{AllStoragesView, Unique, UniqueView, UniqueViewMut};
use egui_systems::CurrentEguiFrame;
use engine::application::exit::ExitRequested;
use engine::input::action_map::Action;
use engine::input::InputManager;
use crate::gui_bundle::GuiBundle;

#[derive(Unique)]
pub struct ToggleGuiPressed;

#[derive(Default, Unique)]
pub struct PauseState(pub(crate) bool);

pub fn listen_for_toggle_pause(input: UniqueView<InputManager>, storages: AllStoragesView) {
    if input.just_pressed().get_action(Action::ToggleGui) {
        storages.add_unique(ToggleGuiPressed);
    }
}

pub fn toggle_pause_menu(storages: AllStoragesView, mut open: UniqueViewMut<PauseState>, mut gui_bundle: GuiBundle) {
    let Ok(ToggleGuiPressed) = storages.remove_unique() else {
        return;
    };

    open.0 = !open.0;
    gui_bundle.set_capture(!open.0, true);
}

pub fn draw_pause_menu(
    mut open: UniqueViewMut<PauseState>,
    egui_frame: UniqueView<CurrentEguiFrame>,
    storages: AllStoragesView,
    gui_bundle: GuiBundle,
) {
    if !open.0 {
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