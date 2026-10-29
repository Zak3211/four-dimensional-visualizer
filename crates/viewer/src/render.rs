//! Draws the projected mesh with egui's painter.

use crate::app::VisualizerApp;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke};
use geom4::{back_to_front, project_mesh, Mat4, ScreenPoint};

/// Paints faces (furthest first) and then edges into `rect`.
pub fn paint(painter: &egui::Painter, rect: Rect, app: &VisualizerApp) {
    if rect.width() < 1.0 || rect.height() < 1.0 {
        return;
    }
    let aspect = f64::from(rect.width() / rect.height());
    let rot = Mat4::from_angles(&app.angles);
    let pm = project_mesh(&app.mesh, &rot, &app.projection, &app.camera, aspect);
    let scale = app.color_scale;

    let to_screen = |p: ScreenPoint| {
        Pos2::new(
            rect.center().x + p.x as f32 * rect.width() / 2.0,
            rect.center().y - p.y as f32 * rect.height() / 2.0,
        )
    };

    if app.render.show_faces && app.render.face_opacity > 0.0 {
        let mut mesh = egui::Mesh::default();
        for t in back_to_front(&app.mesh, &pm) {
            // Colour by the whole face's mean w, so a face is one colour.
            let corners = &app.faces[app.mesh.face_of_triangle[t]];
            let mean_w =
                corners.iter().map(|&i| pm.vertex_w[i]).sum::<f64>() / corners.len() as f64;
            let color = w_color(mean_w / scale, app.render.face_opacity);
            let base = mesh.vertices.len() as u32;
            for &i in &app.mesh.triangles[t] {
                let p = pm.points[i].expect("back_to_front only returns fully projected faces");
                mesh.colored_vertex(to_screen(p), color);
            }
            mesh.add_triangle(base, base + 1, base + 2);
        }
        painter.add(egui::Shape::mesh(mesh));
    }

    if app.render.show_edges {
        for &(a, b) in &app.edges {
            if let (Some(pa), Some(pb)) = (pm.points[a], pm.points[b]) {
                let mean_w = (pm.vertex_w[a] + pm.vertex_w[b]) / 2.0;
                let color = w_color(mean_w / scale, 1.0);
                painter.line_segment([to_screen(pa), to_screen(pb)], Stroke::new(1.6, color));
            }
        }
    }
}

/// Blue for `t = -1`, through pale grey, to orange for `t = +1`.
fn w_color(t: f64, opacity: f32) -> Color32 {
    let t = ((t.clamp(-1.0, 1.0) + 1.0) / 2.0) as f32;
    let lerp = |a: f32, b: f32, s: f32| a + (b - a) * s;
    let (blue, mid, orange) = (
        [70.0, 140.0, 255.0],
        [225.0, 225.0, 230.0],
        [255.0, 150.0, 50.0],
    );
    let rgb: Vec<u8> = (0..3)
        .map(|k| {
            let v = if t < 0.5 {
                lerp(blue[k], mid[k], t * 2.0)
            } else {
                lerp(mid[k], orange[k], t * 2.0 - 1.0)
            };
            v.round() as u8
        })
        .collect();
    let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(rgb[0], rgb[1], rgb[2], alpha)
}
