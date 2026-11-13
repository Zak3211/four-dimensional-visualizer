# CLAUDE.md

A native Rust app that rotates a 4D shape, projects it into a 3D "screen", and draws that with an orbit camera.

## Commands

```sh
cargo test --workspace                                  # all tests (geom4 holds the maths tests)
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings   # CI runs with -D warnings
cargo run --release -p viewer [-- flags]                # flags: see crates/viewer/src/main.rs
./scripts/screenshots.sh                                # regenerate docs/screenshots under Xvfb
```

CI (`.github/workflows/ci.yml`) runs fmt, clippy and tests on ubuntu-latest.

## Workflow

- Work goes on a branch and is delivered as a PR. Never commit to `main`.
- One jj commit per task by default. Use multiple commits only when work is split across implementation agents (a chain of commits, one jj workspace per agent, merged bottom-up by a single merge agent) or when asked. Every commit must build and pass fmt, clippy and tests on its own.
- Plans live in GitHub issues (PR 1 is issue #1).

## Layout

```
crates/geom4/src/     pure maths, f64, NO external dependencies (keep it that way)
  vec.rs              Vec3, Vec4: Add/Sub/Neg/Mul<f64>, dot, norm, to/from_array
  rotation.rs         Plane (6, Plane::ALL order), Mat4: identity, rotation, from_angles,
                      Mul<Mat4> (composition) and Mul<Vec4> (apply)
  projection.rs       Projection4 { Perspective { eye_w }, Orthographic }::project -> Option<Vec3>,
                      max_extent(radius) (bound on |projected point|)
  camera.rs           OrbitCamera { yaw, pitch, distance, fov_y }::project -> Option<ScreenPoint>
  mesh.rs             VertexId, FaceId, Triangle, Edge (sorted ends) newtypes;
                      Polygon { corners, triangles } (fan, built once); Mesh4 { vertices, faces }
                      (private, built by from_polygons -> Result<_, MeshError>): edges, triangles,
                      face_ids, vertex, face, face_centre, radius
  shapes.rs           tesseract()
  pipeline.rs         ProjectedMesh (point(VertexId), w(VertexId)), project_mesh,
                      back_to_front -> Vec<FaceId>
crates/viewer/src/    eframe 0.37 app (eframe is the only external dependency)
  main.rs             window setup and CLI flags (apply_args, parse_six)
  app.rs              VisualizerApp (+ cached edges, color_scale), RenderOptions,
                      control panel, camera input, min_camera_distance, wrap_angle
  render.rs           paint(): faces furthest first, filled from their stored triangles in one
                      egui::Mesh and coloured by mean w, then edges
```

Pipeline: 4D vertex → `Mat4` → `Projection4` → `OrbitCamera` → egui painter.

## Conventions and facts

- `Mat4` acts on column vectors. `from_angles` applies XY first and ZW last: `R = R_ZW ⋯ R_XY`. `rotation(plane, θ)` turns the plane's first axis towards its second.
- Polygons are the source of truth. Each face is convex and planar with corners in cyclic order (checked by per-shape tests, not by `from_polygons`). Edges are adjacent corner pairs, so triangulation diagonals never appear as edges.
- Faces are filled from their stored triangles (`Polygon::triangles`) in one `egui::Mesh`, which is not anti-aliased. Do not switch to `Shape::convex_polygon`: its feathering draws long spikes on faces seen nearly edge-on and produces NaN vertices for faces that project to a line (e.g. orthographic with zero angles).
- Ids are newtypes (`VertexId`, `FaceId`). Never pass raw `usize` indices across the `geom4` API.
- Eye distance: rotations preserve norms, so `|w| ≤ mesh.radius()`. The eye-w slider is bounded to `[1.1 r, 5 r]` (`VisualizerApp::eye_range`) and defaults to `2r`. Never hardcode it per shape.
- Painter's order is per **face**: `back_to_front` returns face ids sorted by mean corner depth. Colour is per face too.
- The colour scale is the largest face-centre norm (`color_scale`, √2 for the tesseract), not the vertex radius, so the blue-to-orange range is actually used.
- Camera floor: the 4D perspective can project points out to `r·e/√(e²−r²)` (`Projection4::max_extent`), which is 4.8 at e = 2.2 and r = 2. `min_camera_distance()` is 1.25× that, enforced every frame, so the camera never ends up inside the shape.
- `ScreenPoint` x and y are in `[-1, 1]` with y up. `render::paint` flips y for egui.
- Tesseract vertex `i`: coordinate `k` is +1 if bit `k` of `i` is set (x = bit 0, …, w = bit 3).

## Gotchas

- **eframe/egui 0.37 API:** `eframe::App` has `fn ui(&mut self, ui: &mut egui::Ui, frame)`, not `update(ctx, …)`. Panels are `egui::Panel::left(id).show(ui, …)` and `egui::CentralPanel::default().show(ui, …)`. `show_inside` is deprecated. Read the source in `~/.cargo/registry` rather than older docs.
- **Headless rendering works.** eframe's default glow (OpenGL) renderer runs under Xvfb with Mesa llvmpipe. The `libEGL warning: DRI3 …` lines are harmless. Use `scripts/screenshots.sh`, or `import -window root` from ImageMagick.
- On Linux the window libraries are `dlopen`ed at run time, so building needs only a C linker and no `-dev` packages.
- The `crates.io` web API (and so `cargo search`) may be blocked in Claude sessions, but the sparse index and downloads work, so builds are fine.

## Known limitations (PR 1)

- **Painter's algorithm is approximate.** Triangles that interpenetrate in 3D can be ordered wrongly, so faces default to 35% opacity. Edges are always drawn on top of faces.
- **4D occlusion is not handled.** Every face is drawn, including ones that are hidden in 4D.
- **(Item 4) The sliders are absolute angles composed in a fixed order, so they are not independent.** After a large XY angle, moving XW does not turn the shape "in XW as you see it now". Planned fix: keep an accumulated `Mat4` orientation, and have sliders and spin apply incremental rotations to it (`orientation = R_plane(δ) * orientation`). Re-orthonormalise it every frame or so with Gram–Schmidt on the rows to stop drift. The sliders then become rate or nudge controls, or show the angles extracted for display only.
- Vertices behind the 4D eye are dropped, and every triangle or edge touching them is skipped instead of clipped. The eye-w floor prevents this for the built-in shape.

## Verified environment risk

- **(Item 5) eframe on Xvfb and Mesa** was unverified in planning. It was checked in step 1 of PR 1 and works, and the screenshots in `docs/screenshots` were produced that way.

## Roadmap and ideas (not done)

Roughly in priority order:

1. **Accumulated orientation** (item 4 above). This is the most noticeable UX issue.
2. **The other regular polytopes:** 5-cell, 16-cell, 24-cell, 120-cell and 600-cell. Add a shape picker to the panel. `eye_range` and the colour scale already adapt through `radius()`.
3. **CI on macOS and Windows** (a build-only matrix), because the friend runs one of those.
4. **Release binaries** from a tag-triggered workflow, so the friend can run it without installing Rust.
5. **Mouse-driven 4D rotation:** for example, Shift-drag rotates XW/YW and Ctrl-drag rotates ZW. This pairs well with the accumulated orientation.
6. **Slicing track:** intersect the 3-cells (or faces) with a hyperplane `w = c` and show the 3D cross-section, with `c` on a slider.
7. **GPU track:** a depth buffer for correct opaque faces (an `egui_glow` paint callback or wgpu), then curved shapes (hypersphere, Clifford torus) by ray-marching implicit functions. That is also the point to switch `geom4` to `glam` f32 types and `bytemuck` vertex buffers. Until then, the maths is far too small for a linear-algebra crate to help.
8. **Smaller items:**
   - Persist settings (eframe's `persistence` feature).
   - Keyboard shortcuts (space for pause, R for reset).
   - Colour by cell instead of by w, as an option.
   - Only recompute the projection when the state has changed (cheap today, so do it only if it shows up in a profile).
   - A tolerance-based planarity and convexity check for user-supplied shapes (`from_polygons` already checks indices, corner count and repeats).
   - Store the eye distance separately from `Projection4`, so toggling to orthographic and back keeps the slider value.
   - Property tests over a few hundred fixed-seed random rotations (a hand-rolled LCG, with no new dependencies).
   - Near-plane clipping of triangles and edges in `pipeline` instead of dropping them whole. This would allow a closer camera and an eye inside the shape.
   - A `Projection4::perspective(eye_w)` constructor that rejects `eye_w <= 0`. Today it is only a `debug_assert`.
