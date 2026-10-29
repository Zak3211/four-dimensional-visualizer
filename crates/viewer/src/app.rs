//! App state and the control panel.

use eframe::egui;
use geom4::{Mesh4, OrbitCamera, Plane, Projection4, Vec4};
use std::f64::consts::PI;

/// What gets drawn and how.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderOptions {
    pub show_faces: bool,
    pub show_edges: bool,
    pub face_opacity: f32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            show_faces: true,
            show_edges: true,
            // Painter's algorithm is approximate; translucency hides most errors.
            face_opacity: 0.35,
        }
    }
}

pub struct VisualizerApp {
    pub mesh: Mesh4,
    /// Cached `mesh.feature_edges()`.
    pub edges: Vec<(usize, usize)>,
    /// Cached `mesh.face_vertices()`, for per-face colour.
    pub faces: Vec<Vec<usize>>,
    /// Largest face-centre norm: the most a face's mean w can reach under any
    /// rotation. Colours are scaled by it so the full range is used.
    pub color_scale: f64,
    /// Slider angles in radians, one per `Plane::ALL` entry, kept in `[-π, π)`.
    pub angles: [f64; 6],
    /// Auto-spin rates in rad/s, one per plane.
    pub spin: [f64; 6],
    pub projection: Projection4,
    pub camera: OrbitCamera,
    pub render: RenderOptions,
}

/// Default spin: a slow turn through w so the inner and outer cubes swap.
pub const DEFAULT_SPIN: [f64; 6] = [0.0, 0.0, 0.35, 0.0, 0.2, 0.0];
pub const MAX_SPIN: f64 = 2.0;
const MIN_PITCH: f64 = -1.5;
const MAX_PITCH: f64 = 1.5;
const MIN_DISTANCE: f64 = 3.0;
const MAX_DISTANCE: f64 = 40.0;

impl VisualizerApp {
    pub fn new(mesh: Mesh4) -> Self {
        let edges = mesh.feature_edges();
        let faces = mesh.face_vertices();
        let color_scale = faces
            .iter()
            .map(|c| {
                let sum = c
                    .iter()
                    .fold(Vec4::default(), |acc, &i| acc + mesh.vertices[i]);
                (sum * (1.0 / c.len() as f64)).norm()
            })
            .fold(0.0, f64::max)
            .max(f64::EPSILON);
        let mut app = Self {
            mesh,
            edges,
            faces,
            color_scale,
            angles: [0.0; 6],
            spin: DEFAULT_SPIN,
            projection: Projection4::Orthographic,
            camera: OrbitCamera::default(),
            render: RenderOptions::default(),
        };
        app.reset();
        app
    }

    /// Allowed eye distances `[1.1 r, 5 r]` for mesh radius `r`. The floor keeps
    /// every rotated vertex in front of the 4D eye, since `|w| <= r`.
    pub fn eye_range(&self) -> std::ops::RangeInclusive<f64> {
        let r = self.mesh.radius().max(f64::EPSILON);
        1.1 * r..=5.0 * r
    }

    pub fn default_eye_w(&self) -> f64 {
        2.0 * self.mesh.radius().max(f64::EPSILON)
    }

    /// Closest the camera may get: outside everything the current 4D
    /// projection can produce, so no vertex ends up behind the camera.
    pub fn min_camera_distance(&self) -> f64 {
        let extent = self.projection.max_extent(self.mesh.radius());
        (1.25 * extent).clamp(MIN_DISTANCE, MAX_DISTANCE)
    }

    /// Back to the start-up view.
    pub fn reset(&mut self) {
        self.angles = [0.0; 6];
        self.spin = DEFAULT_SPIN;
        self.projection = Projection4::Perspective {
            eye_w: self.default_eye_w(),
        };
        self.camera = OrbitCamera::default();
        self.render = RenderOptions::default();
    }

    /// Advances the angles by the spin rates over `dt` seconds.
    pub fn advance(&mut self, dt: f64) {
        for (a, s) in self.angles.iter_mut().zip(self.spin) {
            *a = wrap_angle(*a + s * dt);
        }
    }

    pub fn spinning(&self) -> bool {
        self.spin.iter().any(|&s| s != 0.0)
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("4D rotation");
        egui::Grid::new("rotation_grid")
            .num_columns(3)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label("");
                ui.label("angle");
                ui.label("spin (rad/s)");
                ui.end_row();
                for (k, plane) in Plane::ALL.iter().enumerate() {
                    ui.label(plane.label());
                    ui.add(egui::Slider::new(&mut self.angles[k], -PI..=PI).fixed_decimals(2));
                    ui.add(
                        egui::Slider::new(&mut self.spin[k], -MAX_SPIN..=MAX_SPIN)
                            .fixed_decimals(2),
                    );
                    ui.end_row();
                }
            });
        ui.horizontal(|ui| {
            if ui.button("Stop spin").clicked() {
                self.spin = [0.0; 6];
            }
            if ui.button("Zero angles").clicked() {
                self.angles = [0.0; 6];
            }
        });

        ui.separator();
        ui.heading("4D → 3D");
        let mut perspective = matches!(self.projection, Projection4::Perspective { .. });
        ui.horizontal(|ui| {
            ui.radio_value(&mut perspective, true, "Perspective");
            ui.radio_value(&mut perspective, false, "Orthographic");
        });
        let range = self.eye_range();
        match (perspective, &mut self.projection) {
            (true, Projection4::Perspective { eye_w }) => {
                ui.add(
                    egui::Slider::new(eye_w, range)
                        .text("eye w")
                        .fixed_decimals(2),
                );
            }
            (true, Projection4::Orthographic) => {
                self.projection = Projection4::Perspective {
                    eye_w: self.default_eye_w(),
                };
            }
            (false, _) => self.projection = Projection4::Orthographic,
        }

        ui.separator();
        ui.heading("Display");
        ui.checkbox(&mut self.render.show_faces, "Faces");
        ui.add_enabled(
            self.render.show_faces,
            egui::Slider::new(&mut self.render.face_opacity, 0.0..=1.0).text("opacity"),
        );
        ui.checkbox(&mut self.render.show_edges, "Edges");

        ui.separator();
        if ui.button("Reset").clicked() {
            self.reset();
        }
        ui.add_space(8.0);
        ui.label("Drag to orbit, scroll to zoom.");
        ui.label("Colour shows w after rotation: blue is −w, orange is +w.");
    }

    fn handle_camera_input(&mut self, response: &egui::Response, ui: &egui::Ui) {
        if response.dragged() {
            let d = response.drag_delta();
            self.camera.yaw -= 0.008 * f64::from(d.x);
            self.camera.pitch =
                (self.camera.pitch + 0.008 * f64::from(d.y)).clamp(MIN_PITCH, MAX_PITCH);
        }
        if response.hovered() {
            let scroll = f64::from(ui.input(|i| i.smooth_scroll_delta().y));
            if scroll != 0.0 {
                self.camera.distance = (self.camera.distance * (-0.002 * scroll).exp())
                    .clamp(self.min_camera_distance(), MAX_DISTANCE);
            }
        }
    }
}

/// Wraps an angle into `[-π, π)`.
pub fn wrap_angle(a: f64) -> f64 {
    if (-PI..PI).contains(&a) {
        a // exact: no rounding for values already in range
    } else {
        (a + PI).rem_euclid(2.0 * PI) - PI
    }
}

impl eframe::App for VisualizerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let dt = f64::from(ui.input(|i| i.stable_dt)).min(0.1);
        self.advance(dt);
        // The eye slider can move the projected shape out past the camera.
        self.camera.distance = self.camera.distance.max(self.min_camera_distance());

        egui::Panel::left("controls")
            .resizable(false)
            .default_size(330.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui));
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_gray(18)))
            .show(ui, |ui| {
                let (response, painter) =
                    ui.allocate_painter(ui.available_size(), egui::Sense::drag());
                self.handle_camera_input(&response, ui);
                crate::render::paint(&painter, response.rect, self);
            });

        if self.spinning() {
            ui.ctx().request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_angle_stays_in_range() {
        for a in [-10.0, -PI, 0.0, 3.0, PI, 7.0, 100.0] {
            let w = wrap_angle(a);
            assert!((-PI..PI).contains(&w), "{a} -> {w}");
            // Same angle: equal sine and cosine.
            assert!((a.sin() - w.sin()).abs() < 1e-9 && (a.cos() - w.cos()).abs() < 1e-9);
        }
    }

    #[test]
    fn eye_range_keeps_the_mesh_in_front_of_the_eye() {
        let app = VisualizerApp::new(geom4::tesseract());
        assert!(*app.eye_range().start() > app.mesh.radius());
        assert!(app.eye_range().contains(&app.default_eye_w()));
        assert_eq!(
            app.projection,
            Projection4::Perspective {
                eye_w: app.default_eye_w()
            }
        );
    }

    #[test]
    fn camera_stays_outside_the_projected_shape() {
        let mut app = VisualizerApp::new(geom4::tesseract());
        for eye in [
            *app.eye_range().start(),
            app.default_eye_w(),
            *app.eye_range().end(),
        ] {
            app.projection = Projection4::Perspective { eye_w: eye };
            let extent = app.projection.max_extent(app.mesh.radius());
            assert!(app.min_camera_distance() > extent, "eye {eye}");
        }
    }

    #[test]
    fn color_scale_is_the_face_centre_radius() {
        let app = VisualizerApp::new(geom4::tesseract());
        assert!((app.color_scale - 2f64.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn advance_applies_spin() {
        let mut app = VisualizerApp::new(geom4::tesseract());
        app.spin = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
        app.advance(0.5);
        assert!((app.angles[2] - 0.5).abs() < 1e-12);
        assert_eq!(app.angles[0], 0.0);
    }
}
