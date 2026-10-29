//! Native viewer for a tesseract projected from 4D into a navigable 3D view.
//!
//! Optional flags set the start-up state (handy for reproducible screenshots):
//! `--angles a,b,c,d,e,f` (radians, XY XZ XW YZ YW ZW), `--spin a,b,c,d,e,f`,
//! `--eye W`, `--ortho`, `--no-faces`, `--no-edges`.

mod app;
mod render;

use app::VisualizerApp;
use eframe::egui;
use geom4::Projection4;

fn main() -> eframe::Result<()> {
    let mut app = VisualizerApp::new(geom4::tesseract());
    if let Err(msg) = apply_args(&mut app, std::env::args().skip(1)) {
        eprintln!("{msg}");
        std::process::exit(2);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("4D visualizer")
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native("4D visualizer", options, Box::new(|_cc| Ok(Box::new(app))))
}

fn apply_args(app: &mut VisualizerApp, args: impl Iterator<Item = String>) -> Result<(), String> {
    let usage = "usage: viewer [--angles a,b,c,d,e,f] [--spin a,b,c,d,e,f] [--eye W] [--ortho] [--no-faces] [--no-edges]";
    let mut args = args.peekable();
    while let Some(flag) = args.next() {
        let mut value = || {
            args.next()
                .ok_or_else(|| format!("{flag} needs a value\n{usage}"))
        };
        match flag.as_str() {
            "--angles" => app.angles = parse_six(&value()?)?,
            "--spin" => app.spin = parse_six(&value()?)?,
            "--eye" => {
                let w: f64 = value()?.parse().map_err(|e| format!("--eye: {e}"))?;
                let range = app.eye_range();
                if !range.contains(&w) {
                    return Err(format!("--eye must be in {range:?} for this shape"));
                }
                app.projection = Projection4::Perspective { eye_w: w };
            }
            "--ortho" => app.projection = Projection4::Orthographic,
            "--no-faces" => app.render.show_faces = false,
            "--no-edges" => app.render.show_edges = false,
            "-h" | "--help" => return Err(usage.to_string()),
            other => return Err(format!("unknown flag {other}\n{usage}")),
        }
    }
    Ok(())
}

fn parse_six(s: &str) -> Result<[f64; 6], String> {
    let v: Vec<f64> = s
        .split(',')
        .map(|x| x.trim().parse::<f64>().map_err(|e| format!("{x:?}: {e}")))
        .collect::<Result<_, _>>()?;
    v.try_into()
        .map_err(|v: Vec<f64>| format!("expected 6 comma-separated numbers, got {}", v.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> impl Iterator<Item = String> + '_ {
        s.split_whitespace().map(String::from)
    }

    #[test]
    fn flags_set_state() {
        let mut app = VisualizerApp::new(geom4::tesseract());
        apply_args(
            &mut app,
            args("--angles 0,0,0.9,0,0,0 --spin 0,0,0,0,0,0 --ortho --no-faces"),
        )
        .unwrap();
        assert_eq!(app.angles[2], 0.9);
        assert!(!app.spinning());
        assert_eq!(app.projection, Projection4::Orthographic);
        assert!(!app.render.show_faces && app.render.show_edges);
    }

    #[test]
    fn bad_flags_are_rejected() {
        let mut app = VisualizerApp::new(geom4::tesseract());
        assert!(apply_args(&mut app, args("--angles 1,2,3")).is_err());
        assert!(apply_args(&mut app, args("--eye 1.5")).is_err()); // below 1.1 * radius
        assert!(apply_args(&mut app, args("--spin")).is_err());
        assert!(apply_args(&mut app, args("--bogus")).is_err());
    }
}
