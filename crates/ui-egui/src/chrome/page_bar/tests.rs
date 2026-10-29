use serde_json::json;

use super::*;

fn app() -> VectorcraftApp {
    let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({"width": 100, "height": 100})).unwrap();
    app
}

/// Draw the bar for one headless frame: the texts drawn.
fn frame(app: &mut VectorcraftApp) -> Vec<String> {
    fn texts(s: &egui::Shape, out: &mut Vec<String>) {
        match s {
            egui::Shape::Text(t) => out.push(t.galley.text().to_string()),
            egui::Shape::Vec(v) => v.iter().for_each(|s| texts(s, out)),
            _ => {}
        }
    }
    let ctx = egui::Context::default();
    crate::theme::install_fonts(&ctx);
    let raw = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 400.0))), ..Default::default() };
    let mut out = ctx.run_ui(raw, |ui| page_bar(app, ui));
    out.textures_delta.clear();
    let mut v = vec![];
    out.shapes.iter().for_each(|c| texts(&c.shape, &mut v));
    v
}

#[test]
fn an_artboards_document_shows_its_artboards_as_pages() {
    let mut app = app();
    app.run("artboard.new", json!({"name": "Back"})).unwrap();
    let p = pages(&app);
    assert_eq!(p.iter().map(|p| p.index).collect::<Vec<_>>(), [0, 1]);
    assert_eq!(p[1].name, "Back");
    let texts = frame(&mut app);
    assert!(texts.iter().any(|t| t == "Page 1 of 2"), "{texts:?}");
    assert!(texts.iter().any(|t| t == "Back"), "a tab per page: {texts:?}");
}

#[test]
fn the_bar_follows_the_page_order_and_the_active_page() {
    let mut app = app();
    app.run("artboard.new", json!({"name": "Two"})).unwrap();
    run(&mut app, Action::Move { page: 2, to: 1 });
    let p = pages(&app);
    // The default-named artboard took its page's name as the document became a pages document.
    assert_eq!(p.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["Two", "Page 2"]);
    assert!(app.session.active().unwrap().doc.is_paged(), "moving a page made it a pages document");
    run(&mut app, Action::Go(0));
    assert_eq!(current(&app, &p), 1, "artboard 0 is page 2 now");
    assert!(frame(&mut app).iter().any(|t| t == "Page 2 of 2"));
}

#[test]
fn add_page_inserts_after_the_page_and_goes_to_it() {
    let mut app = app();
    run(&mut app, Action::Insert(1));
    let p = pages(&app);
    assert_eq!(p.len(), 2);
    assert_eq!(p[1].name, "Page 2");
    assert_eq!(current(&app, &p), 1, "the new page is the active one");
    // Before the first.
    run(&mut app, Action::Insert(0));
    let p = pages(&app);
    assert_eq!(p[0].name, "Page 1");
    assert_eq!(current(&app, &p), 0);
    // One undo step per insert.
    app.run("edit.undo", json!({})).unwrap();
    assert_eq!(pages(&app).len(), 2);
}

#[test]
fn rename_and_delete_run_the_artboard_commands() {
    let mut app = app();
    run(&mut app, Action::Insert(1));
    run(&mut app, Action::Rename(1, "Back cover".into()));
    assert_eq!(pages(&app)[1].name, "Back cover");
    run(&mut app, Action::Delete(1));
    assert_eq!(pages(&app).len(), 1);
    // The last page can't go: the error reaches the status bar.
    run(&mut app, Action::Delete(0));
    assert_eq!(pages(&app).len(), 1);
    assert!(!app.ui.status.is_empty());
}

/// In Draw mode the status bar leaves out its artboard navigator: the page bar does that work.
#[test]
fn draw_mode_drops_the_status_bars_artboard_navigator() {
    let mut app = app();
    let status = |app: &mut VectorcraftApp| {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let raw = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 400.0))), ..Default::default() };
        let mut out = ctx.run_ui(raw, |ui| crate::chrome::status_bar(app, ui));
        out.textures_delta.clear();
        out.shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Text(t) => Some(t.galley.text().to_string()),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert!(status(&mut app).iter().any(|t| t.contains("Artboard 1")), "Illustrate: the navigator shows");
    app.run("window.mode", json!({"mode": "draw"})).unwrap();
    assert!(!status(&mut app).iter().any(|t| t.contains("Artboard 1")), "Draw: the page bar has it");
}

#[test]
fn no_document_no_bar() {
    let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), crate::Services::default());
    assert!(frame(&mut app).is_empty());
}
