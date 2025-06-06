use egui::Window;
use shipyard::{IntoIter, UniqueView, View};
use egui_systems::CurrentEguiFrame;
use engine::components::{Entity, HeldBlock, LocalPlayer, Transform, Velocity};
use engine::inventory::PlayerInventory;
use engine::networking::server_handler::ServerHandler;
use game::inventory::Inventory;

pub fn debug_ui(
    egui_frame: UniqueView<CurrentEguiFrame>,

    // for player debug info
    v_local_player: View<LocalPlayer>,
    v_entity: View<Entity>,
    v_transform: View<Transform>,
    v_velocity: View<Velocity>,
    v_inventory: View<PlayerInventory>,
    held: UniqueView<HeldBlock>,

    opt_server_handler: Option<UniqueView<ServerHandler>>,
) {
    let ctx = egui_frame.ctx();

    let vec3_fmt = |title: &'static str, v: &glm::Vec3| format!("{title}: [{:.2}, {:.2}, {:.2}]", v.x, v.y, v.z);

    let (_, local_transform, velocity, inventory) = (&v_local_player, &v_transform, &v_velocity, &v_inventory)
        .iter()
        .next()
        .expect("LocalPlayer didn't have transform & held block");

    let mut other_pos = (!&v_local_player, &v_entity, &v_transform).iter()
        .map(|e| &e.2.position)
        .peekable();

    Window::new("Entities")
        .default_open(true)
        .show(ctx, |ui| {
            ui.heading("LocalPlayer");
            ui.label(vec3_fmt("Position", &local_transform.position));
            ui.label(vec3_fmt("Velocity", &velocity.0));
            
            let held_item = inventory.as_slice().get(held.0).expect("in range").as_ref().map(|it| it.item.title.as_str());
            
            ui.label(format!("{held_item:?}")); // TODO: stop displaying held

            if other_pos.peek().is_some() {
                ui.heading("Entities");

                for pos in other_pos {
                    ui.label(vec3_fmt("Position", pos));
                }
            }
        });

    if let Some(server_handler) = opt_server_handler {
        Window::new("ServerHandler")
            .default_open(true)
            .show(ctx, |ui| {
                ui.label(format!("Address: {}", server_handler.local_addr));
            });
    }
}