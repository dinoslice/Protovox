use egui::{lerp, pos2, vec2, Color32, CursorIcon, FontId, Key, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Window};
use shipyard::Unique;
use splines::{Easing, Spline};
use splines::easings::InOutSine;

fn norm_v(x: &Vec2) -> Vec2 {
    *x * 2.0 - Vec2::splat(1.0)
}

fn denorm_v(x: &Vec2) -> Vec2 {
    (*x + Vec2::splat(1.0)) * 0.5
}

#[derive(Debug, Unique, Default)]
pub struct SplineEditor {
    pub points: Vec<Vec2>,
    editing: Option<PointEditing>
}

#[derive(Debug)]
struct PointEditing {
    pub to_replace: Vec2,
    pub editing: Vec2,
}

impl PointEditing {
    pub fn new(point: Vec2) -> Self {
        let editing = Vec2 {
            y: -point.y,
            .. point
        };
        
        Self {
            to_replace: point,
            editing,
        }
    }
}

impl SplineEditor {
    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        let plot_rect = ui.max_rect();
        let lines = 11;
        let margin = 20.0;

        // Calculate available grid area and cell dimensions
        let grid_width = plot_rect.width() - 2.0 * margin;
        let grid_height = plot_rect.height() - 2.0 * margin;
        
        let grid_size = vec2(grid_width, grid_height);
        
        let cell_width = grid_width / (lines - 1) as f32;
        let cell_height = grid_height / (lines - 1) as f32;

        let font_id = FontId::new(10.0, egui::FontFamily::Monospace);

        // Main container with horizontal layout
        ui.horizontal(|ui| {
            // Left container for Y-axis labels
            ui.vertical(|ui| {
                ui.add_space(margin); // Align with grid top
                for i in 0..lines {
                    let y_value = 1.0 - ( i as f32 * 2.0 / (lines - 1) as f32 );
                    let label_y = plot_rect.top() + margin + i as f32 * cell_height;

                    ui.painter().text(
                        pos2(plot_rect.left() + 5.0, label_y), // Position slightly inside for visibility
                        egui::Align2::LEFT_CENTER,
                        format!("{:.1}", y_value),
                        font_id.clone(),
                        Color32::WHITE,
                    );

                    if i < lines - 1 {
                        ui.add_space(cell_height); // Align labels with grid cells
                    }
                }
            });

            // Grid area for drawing the plot
            ui.vertical(|ui| {
                let grid_rect = Rect::from_min_size(
                    plot_rect.min + vec2(margin, margin),
                    vec2(grid_width, grid_height),
                );

                // Draw grid lines
                for i in 0..lines {
                    let x = grid_rect.left() + i as f32 * cell_width;
                    let y = grid_rect.top() + i as f32 * cell_height;

                    // Vertical grid lines
                    ui.painter().line_segment(
                        [pos2(x, grid_rect.top()), pos2(x, grid_rect.bottom())],
                        Stroke::new(1.0, Color32::from_gray(100)),
                    );

                    // Horizontal grid lines
                    ui.painter().line_segment(
                        [pos2(grid_rect.left(), y), pos2(grid_rect.right(), y)],
                        Stroke::new(1.0, Color32::from_gray(100)),
                    );
                }

                // Draw lines between points
                if self.points.len() > 1 {
                    let mut sorted = self.points.clone();
                    sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).expect("not nan"));

                    let mut eased_points = Vec::new();

                    for w in sorted.windows(2) {
                        let start = denorm_v(&w[0]);
                        let end = denorm_v(&w[1]);

                        let res = 20;

                        for i in 0..res {
                            let t = i as f32 / (res - 1) as f32;

                            let interpolated_point = Vec2 {
                                x: lerp(start.x..=end.x, t),
                                y: lerp(start.y..=end.y, InOutSine::ease(t)),
                            };

                            eased_points.push(grid_rect.min + grid_size * interpolated_point);
                        }
                    }

                    let line = sorted.iter().map(|v| grid_rect.min + grid_size * denorm_v(v)).collect();
                    ui.painter().add(Shape::line(line, Stroke::new(1.0, Color32::GOLD)));

                    ui.painter().add(Shape::line(eased_points, Stroke::new(2.0, Color32::GREEN)));
                }

                // Handle dragging and adding points
                self.points.retain_mut(|point| {
                    let screen_pos = grid_rect.min + grid_size * denorm_v(point);

                    let response = ui.allocate_rect(
                        Rect::from_center_size(screen_pos, Vec2::splat(10.0)),
                        Sense::click_and_drag(),
                    );

                    if response.hovered() {
                        ui.ctx().set_cursor_icon(CursorIcon::Grab);
                    }
                    if response.dragged() && self.editing.is_none() {
                        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                        if let Some(pos) = response.interact_pointer_pos() {
                            let pos = (pos.to_vec2() - grid_rect.min.to_vec2()) / grid_size;
                            
                            *point = norm_v(&pos).clamp(Vec2::splat(-1.0), Vec2::splat(1.0));
                        }
                    }

                    ui.painter().circle_filled(screen_pos, 5.0, Color32::LIGHT_BLUE);

                    // If clicked on a point, select it for editing
                    if ui.input(|i| i.key_pressed(Key::E)) && response.hovered() {
                        self.editing = Some(PointEditing::new(*point));
                    }

                    !(ui.input(|i| i.key_pressed(Key::Delete)) && response.hovered())
                });

                // Add a new point when clicking in the grid
                if ui.input(|i| i.pointer.primary_clicked()) {
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        let norm_pos = norm_v(&((pos.to_vec2() - grid_rect.min.to_vec2()) / grid_size));
                        
                        if norm_pos.x.abs() < 1.0 && norm_pos.y.abs() < 1.0 {
                            self.points.push(norm_pos);
                        }
                    }
                }
            });
        });

        // X-axis labels
        ui.horizontal(|ui| {
            for i in 0..lines {
                let x_value = -1.0 + i as f32 * 2.0 / (lines - 1) as f32;
                let label_x = plot_rect.left() + margin + i as f32 * cell_width;

                ui.painter().text(
                    pos2(label_x, plot_rect.bottom() - 15.0),
                    egui::Align2::CENTER_TOP,
                    format!("{x_value:.1}"),
                    font_id.clone(),
                    Color32::WHITE,
                );

                if i < lines - 1 {
                    ui.add_space(cell_width);
                }
            }
        });
        
        let SplineEditor { points, editing: opt_editing } = self;

        // Show the popup for editing the selected point
        if let Some(PointEditing { to_replace, editing }) = opt_editing {
            let mut close_window = false;
            
            Window::new("Edit Point")
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Edit the point's coordinates");

                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut editing.x).range(-1.0..=1.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut editing.y).range(-1.0..=1.0));
                    });

                    if ui.button("Save").clicked() {
                        let Some(p) = points.iter_mut().find(|p| **p == *to_replace) else {
                            unreachable!("oops")
                        };
                        
                        *p = Vec2 {
                            y: -editing.y,
                            .. *editing
                        };

                        close_window = true;
                    }

                    if ui.button("Cancel").clicked() {
                        close_window = true;
                    }
                });
            
            if close_window {
                *opt_editing = None;
            }
        }

        ui.allocate_rect(ui.max_rect(), Sense::hover())
    }

    pub fn make_spline<E: Easing>(&self) -> Spline<E> {
        let points = self.points.iter().map(|v| glm::Vec2::new(v.x, -v.y));
        Spline::new(points).expect("valid spline")
    }
}

fn easing_sine(t: f32) -> f32 {
    use std::f32::consts::PI;

    -(t * PI).cos() * 0.5 + 0.5
}

fn easing_parametric(t: f32) -> f32 {
    let sqr = t * t;
    sqr / (2.0 * (sqr - t) + 1.0)
}