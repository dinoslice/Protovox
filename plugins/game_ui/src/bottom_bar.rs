use egui::{Align2, Area, Color32, Frame, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};
use shipyard::{IntoIter, UniqueView, View};
use egui_systems::CurrentEguiFrame;
use engine::components::{Health, LocalPlayer, Mana, SpectatorSpeed};
use engine::gamemode::Gamemode;

pub fn bottom_bar(
    egui_frame: UniqueView<CurrentEguiFrame>,
    v_local_player: View<LocalPlayer>,
    v_gamemode: View<Gamemode>,
    v_spectator_speed: View<SpectatorSpeed>,
    v_health: View<Health>,
    v_mana: View<Mana>,
) {
    let (gamemode, spec_speed, health, mana, ..) = (&v_gamemode, &v_spectator_speed, &v_health, &v_mana, &v_local_player)
        .iter()
        .next()
        .expect("local player should exist");

    let default_height = SegmentedBar::default().size.y;
    let half_width = SegmentedBar::default().size.x;
    let half_segments = SegmentedBar::default().segments;

    Area::new("bottom_bars".into())
        .anchor(Align2::CENTER_BOTTOM, Vec2::new(0.0, -10.0))
        .show(egui_frame.ctx(), |ui| {
            match gamemode {
                Gamemode::Survival => {
                    ui.horizontal_centered(|ui|  {
                        let size = Vec2::new(half_width, default_height);

                        ui.add(&SegmentedBar {
                            percentage: health.percentage_clamped(),
                            segments: half_segments,
                            filled: Color32::from_rgb(242, 48, 48),
                            size,
                            .. Default::default()
                        })
                            |
                            ui.add(&SegmentedBar {
                                percentage: mana.percentage_clamped(),
                                segments: half_segments,
                                filled: Color32::from_rgb(64, 220, 237),
                                size,
                                .. Default::default()
                            })
                    }).response
                }
                Gamemode::Spectator => {
                    ui.add(&SegmentedBar {
                        percentage: (spec_speed.curr_speed / 384.0).powf(0.45),
                        segments: half_segments * 2,
                        size: Vec2::new(half_width * 2.0, default_height),
                        filled: Color32::from_rgb(35, 35, 35),
                        .. Default::default()
                    })
                }
            }
        });
}

struct SegmentedBar {
    pub percentage: f32,
    pub segments: u16,
    pub spacing: f32,
    pub size: Vec2,

    pub unfilled: Color32,
    pub filled: Color32,
}

impl Default for SegmentedBar {
    fn default() -> Self {
        Self {
            percentage: 0.0,
            segments: 10,
            spacing: 2.0,
            size: Vec2::new(200.0, 10.0),
            unfilled: Color32::from_rgb(100, 100, 100),
            filled: Color32::from_rgb(255, 50, 50),
        }
    }
}

impl Widget for &SegmentedBar {
    fn ui(self, ui: &mut Ui) -> Response {
        debug_assert!((0.0..=1.0).contains(&self.percentage));

        let segments = self.segments as f32;

        let segment_width = (self.size.x - self.spacing * (segments - 1.0)) / segments;
        let filled_segments = ((segments * self.percentage).round() as u16).min(self.segments);

        Frame::none()
            .rounding(5.0)
            .show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());
                let painter = ui.painter();

                for i in 0..self.segments {
                    let x_start = rect.min.x + i as f32 * (segment_width + self.spacing);
                    let segment_rect = Rect {
                        min: Pos2::new(x_start, rect.min.y),
                        max: Pos2::new(x_start + segment_width, rect.max.y),
                    };

                    let color = if i < filled_segments {
                        self.filled
                    } else {
                        self.unfilled
                    };
                    painter.rect_filled(segment_rect, 2.0, color);
                }

                response
            })
            .response
    }
}