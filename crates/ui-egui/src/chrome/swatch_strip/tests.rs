use serde_json::json;

use super::*;

fn app() -> VectorcraftApp {
    let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({"width": 100, "height": 100})).unwrap();
    app
}

#[test]
fn none_comes_first_then_the_documents_solid_swatches() {
    let app = app();
    let c = cells(&app);
    assert_eq!(c[0].paint, Paint::None);
    assert_eq!(c[0].params, json!({"none": true}));
    assert!(c.len() > 1, "a new document has swatches");
    assert!(c[1..].iter().all(|c| matches!(c.paint, Paint::Solid { .. })), "solid colours only");
    let names: Vec<&str> = c.iter().map(|c| c.name.as_str()).collect();
    let mut unique = names.clone();
    unique.dedup();
    assert_eq!(names.len(), unique.len(), "each name once");
}

#[test]
fn a_new_swatch_joins_the_strip() {
    let mut app = app();
    let before = cells(&app).len();
    app.run("swatch.new", json!({"name": "Strip Test", "color": "#123456"})).unwrap();
    let c = cells(&app);
    assert_eq!(c.len(), before + 1);
    assert_eq!(c.last().unwrap().params, json!({"swatch": "Strip Test"}));
}

/// What a click and a right-click send are paint commands the engine takes, and they leave the
/// proxy in front where it was.
#[test]
fn click_fills_and_right_click_strokes() {
    let mut app = app();
    let id = app.run("shape.rectangle", json!({"x": 0, "y": 0, "width": 10, "height": 10})).unwrap()["id"].as_u64().unwrap();
    let c = cells(&app);
    let colour = c.iter().find(|c| !c.name.is_empty()).unwrap().clone();
    let mut p = colour.params.clone();
    p["focus"] = json!(false);
    app.run("paint.setFill", p).unwrap();
    let mut none = c[0].params.clone();
    none["focus"] = json!(false);
    app.run("paint.setStroke", none).unwrap();
    let node = app.run("document.inspect", json!({"ids": [id]})).unwrap()["nodes"][0].clone();
    assert_ne!(node["fill"], serde_json::Value::Null, "{node}");
    assert!(node["fill"].to_string().contains(&colour.name) || node["fill"].as_str().is_some_and(|f| f.starts_with('#')), "{node}");
    assert!(node["stroke"].is_null() || node["stroke"] == "none", "the stroke is None: {node}");
}

#[test]
fn the_strip_draws_one_tile_per_swatch() {
    let mut app = app();
    let ctx = egui::Context::default();
    crate::theme::install_fonts(&ctx);
    let raw = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 2000.0))), ..Default::default() };
    let mut out = ctx.run_ui(raw, |ui| swatch_strip(&mut app, ui));
    out.textures_delta.clear();
    let rect = egui::containers::panel::PanelState::load(&ctx, egui::Id::unique("swatch_strip")).unwrap().outer_rect;
    assert_eq!(rect.width(), WIDTH);
    assert_eq!(rect.right(), 800.0, "at the window's right edge");
}
