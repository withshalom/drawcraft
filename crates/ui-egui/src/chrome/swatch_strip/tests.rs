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
    assert!(c.iter().any(|c| c.params == json!({"swatch": "Strip Test"})));
}

/// What a click and a right-click send are paint commands the engine takes, and they leave the
/// proxy in front where it was.
#[test]
fn click_fills_and_right_click_strokes() {
    let mut app = app();
    let id = app.run("shape.rectangle", json!({"x": 0, "y": 0, "width": 10, "height": 10})).unwrap()["id"].as_u64().unwrap();
    let c = cells(&app);
    // A colour unlike the default fill (white), so the test shows the click took.
    let colour =
        c.iter().rev().find(|c| matches!(&c.paint, Paint::Solid { color, .. } if color.to_hex().to_lowercase() != "#ffffff")).unwrap().clone();
    let Paint::Solid { color, .. } = &colour.paint else { unreachable!() };
    let mut p = colour.params.clone();
    p["focus"] = json!(false);
    app.run("paint.setFill", p).unwrap();
    let mut none = c[0].params.clone();
    none["focus"] = json!(false);
    app.run("paint.setStroke", none).unwrap();
    let node = app.run("document.inspect", json!({"ids": [id]})).unwrap()["nodes"][0].clone();
    assert_eq!(node["fill"].as_str().unwrap().to_lowercase(), color.to_hex().to_lowercase(), "{node}");
    assert_eq!(node["stroke"], "None", "{node}");
}

#[test]
fn the_strip_draws_one_tile_per_swatch() {
    let mut app = app();
    let ctx = egui::Context::default();
    crate::theme::install_fonts(&ctx);
    let raw = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 2000.0))), ..Default::default() };
    let mut out = ctx.run_ui(raw, |ui| swatch_strip(&mut app, ui));
    out.textures_delta.clear();
    fn tiles(s: &egui::Shape, out: &mut Vec<egui::Rect>) {
        match s {
            egui::Shape::Rect(r) if (r.rect.width() - CELL).abs() < 0.5 && (r.rect.height() - CELL).abs() < 0.5 => out.push(r.rect),
            egui::Shape::Vec(v) => v.iter().for_each(|s| tiles(s, out)),
            _ => {}
        }
    }
    let mut rects = vec![];
    out.shapes.iter().for_each(|c| tiles(&c.shape, &mut rects));
    rects.dedup();
    assert!(!rects.is_empty());
    assert!(rects.iter().all(|r| r.left() >= 800.0 - WIDTH && r.right() <= 800.0), "at the window's right edge: {rects:?}");
    let n = cells(&app).len();
    let rows: std::collections::BTreeSet<i32> = rects.iter().map(|r| r.top().round() as i32).collect();
    assert_eq!(rows.len(), n, "one tile per swatch, one above the other");
}
