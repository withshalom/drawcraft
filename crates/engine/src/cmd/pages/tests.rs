use serde_json::{Value, json};

use crate::Session;

fn run(s: &mut Session, id: &str, p: Value) -> Value {
    s.execute(id, &p).unwrap_or_else(|e| panic!("{id}: {e}"))
}

/// A 100 × 100 document with a black 20 × 10 rectangle at (10, 10) on its only layer.
fn doc() -> Session {
    let mut s = Session::new();
    run(&mut s, "file.new", json!({"width": 100, "height": 100}));
    run(&mut s, "shape.rectangle", json!({"x": 10, "y": 10, "width": 20, "height": 10}));
    run(&mut s, "paint.setFill", json!({"color": "#000000"}));
    s
}

fn pages(s: &mut Session) -> Vec<(u64, String)> {
    let l = run(s, "pages.list", json!({}));
    l["pages"].as_array().unwrap().iter().map(|p| (p["id"].as_u64().unwrap(), p["name"].as_str().unwrap().to_string())).collect()
}

#[test]
fn a_document_turns_into_pages_and_back() {
    let mut s = doc();
    assert_eq!(run(&mut s, "pages.list", json!({}))["paged"], false);
    assert!(s.execute("pages.disable", &json!({})).is_err(), "not a pages document yet");
    assert_eq!(run(&mut s, "pages.enable", json!({}))["pages"], 1);
    assert!(s.execute("pages.enable", &json!({})).is_err(), "already pages");
    assert_eq!(run(&mut s, "pages.list", json!({}))["paged"], true);
    run(&mut s, "pages.disable", json!({}));
    assert_eq!(run(&mut s, "pages.list", json!({}))["paged"], false);
    // Each is one undo step.
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(run(&mut s, "pages.list", json!({}))["paged"], true);
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(run(&mut s, "pages.list", json!({}))["paged"], false);
}

#[test]
fn pages_are_inserted_in_order_and_moved() {
    let mut s = doc();
    run(&mut s, "pages.enable", json!({}));
    let r = run(&mut s, "pages.insert", json!({"count": 2}));
    assert_eq!(r["numbers"], json!([2, 3]));
    let d = &s.doc().unwrap().doc;
    assert_eq!(d.artboards.len(), 3);
    // In a row right of the first, the first page's size, 20 pt apart.
    assert_eq!(d.artboards[1].rect, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    assert_eq!(d.artboards[2].rect, vectorcraft_geom::Rect::new(240.0, 0.0, 340.0, 100.0));
    let names: Vec<String> = pages(&mut s).into_iter().map(|p| p.1).collect();
    assert_eq!(&names[1..], ["Page 2", "Page 3"], "each new page named by its number");
    let ids: Vec<u64> = pages(&mut s).iter().map(|p| p.0).collect();
    // A page before the first, of its own size.
    let r = run(&mut s, "pages.insert", json!({"after": 0, "width": 50, "height": 60}));
    assert_eq!(r["numbers"], json!([1]));
    let now: Vec<u64> = pages(&mut s).iter().map(|p| p.0).collect();
    assert_eq!(&now[1..], &ids[..]);
    // Moving a page changes the order, not where the artboard is.
    let before = s.doc().unwrap().doc.artboards.clone();
    run(&mut s, "pages.move", json!({"page": 1, "to": 4}));
    let moved: Vec<u64> = pages(&mut s).iter().map(|p| p.0).collect();
    assert_eq!(moved, [ids[0], ids[1], ids[2], now[0]]);
    assert_eq!(s.doc().unwrap().doc.artboards, before);
    // Bad numbers are errors, not clamps.
    assert!(s.execute("pages.move", &json!({"page": 9, "to": 1})).is_err());
    assert!(s.execute("pages.insert", &json!({"after": 9})).is_err());
    assert!(s.execute("pages.insert", &json!({"count": 0})).is_err());
    assert!(s.execute("pages.insert", &json!({"count": 101})).is_err());
    assert!(s.execute("pages.insert", &json!({"width": "wide"})).is_err());
}

/// Adding a page to a document without pages makes it a pages document, its artboards its first
/// pages, in one undo step (the page bar's Add Page in draw mode).
#[test]
fn inserting_a_page_into_an_artboards_document_turns_it_into_pages_in_one_step() {
    let mut s = doc();
    run(&mut s, "artboard.new", json!({"x": 200, "y": 0, "width": 100, "height": 100}));
    let r = run(&mut s, "pages.insert", json!({}));
    assert_eq!(r["numbers"], json!([3]));
    let l = run(&mut s, "pages.list", json!({}));
    assert_eq!(l["paged"], true);
    assert_eq!(l["pages"].as_array().unwrap().len(), 3);
    // Artboards with their default names take their pages' names; a chosen name stays.
    let names: Vec<&str> = l["pages"].as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["Page 1", "Page 2", "Page 3"]);
    run(&mut s, "edit.undo", json!({}));
    run(&mut s, "artboard.setProps", json!({"index": 0, "name": "Cover"}));
    run(&mut s, "pages.enable", json!({}));
    let l = run(&mut s, "pages.list", json!({}));
    let names: Vec<&str> = l["pages"].as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["Cover", "Page 2"]);
    run(&mut s, "edit.undo", json!({}));
    run(&mut s, "edit.undo", json!({}));
    run(&mut s, "pages.insert", json!({}));
    run(&mut s, "edit.undo", json!({}));
    let l = run(&mut s, "pages.list", json!({}));
    assert_eq!(l["paged"], false, "one undo step takes the page and the table");
    assert_eq!(l["pages"].as_array().unwrap().len(), 2);
    assert_eq!(l["pages"][0]["name"], "Artboard 1", "and the names it gave");
    // Moving a page does the same.
    run(&mut s, "pages.move", json!({"page": 2, "to": 1}));
    let l = run(&mut s, "pages.list", json!({}));
    assert_eq!(l["paged"], true);
    assert_eq!(l["pages"][0]["index"], 1, "the second artboard is page 1 now");
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(run(&mut s, "pages.list", json!({}))["paged"], false);
}

#[test]
fn a_deleted_artboard_leaves_the_page_order() {
    let mut s = doc();
    run(&mut s, "pages.enable", json!({}));
    run(&mut s, "pages.insert", json!({"count": 2}));
    run(&mut s, "artboard.delete", json!({"index": 1}));
    assert_eq!(pages(&mut s).len(), 2);
    // A new artboard made with the artboard command is a page too, last.
    run(&mut s, "artboard.new", json!({}));
    assert_eq!(pages(&mut s).len(), 3);
}

#[test]
fn master_layers_repeat_on_every_page_in_output_only() {
    let mut s = doc();
    run(&mut s, "pages.enable", json!({}));
    run(&mut s, "pages.insert", json!({"count": 2}));
    let layer = s.doc().unwrap().current_layer().unwrap();
    run(&mut s, "pages.setMaster", json!({}));
    let l = run(&mut s, "pages.list", json!({}));
    // With no `master`, it toggles: twice is back where it was.
    run(&mut s, "pages.setMaster", json!({}));
    assert_eq!(run(&mut s, "pages.list", json!({}))["masterLayers"], json!([]));
    run(&mut s, "pages.setMaster", json!({"master": true}));
    let l2 = run(&mut s, "pages.list", json!({}));
    assert_eq!(l2, l);
    assert_eq!(l["masterLayers"][0]["id"], layer.0);
    assert!(s.execute("pages.setMaster", &json!({"id": 999_999})).is_err(), "not a layer");
    // Each page's PNG shows the rectangle at (10, 10) to (30, 20) of that page.
    let d = s.doc().unwrap().doc.clone();
    let enc = crate::cmd::fileio::encode_all(&d, "png", &json!({"useArtboards": true, "ppi": 72, "antiAlias": "none"})).unwrap();
    assert_eq!(enc.files.len(), 3);
    for (_, png) in &enc.files {
        let img = image::load_from_memory(png).unwrap().to_rgba8();
        assert_eq!(img.dimensions(), (100, 100));
        let inside = img.get_pixel(20, 15).0;
        let outside = img.get_pixel(60, 60).0;
        assert!(inside[0] < 64 && inside[3] > 200, "the master art on every page: {inside:?}");
        assert!(outside[0] > 192 || outside[3] < 64, "and nothing else: {outside:?}");
    }
    // The document, and the file it saves, hold the art once.
    let saved = run(&mut s, "document.save", json!({}));
    let bytes = vectorcraft_format::base64_decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let back = vectorcraft_format::load(&bytes).unwrap();
    assert_eq!(back.pages, d.pages);
    let art: usize = back.layers.iter().map(|l| l.children().map_or(0, Vec::len)).sum();
    assert_eq!(art, 1);
    // Back to an ordinary layer: the other pages are empty again.
    run(&mut s, "pages.setMaster", json!({"master": false}));
    let d = s.doc().unwrap().doc.clone();
    let enc = crate::cmd::fileio::encode_all(&d, "png", &json!({"useArtboards": true, "ppi": 72, "antiAlias": "none"})).unwrap();
    let img = image::load_from_memory(&enc.files[1].1).unwrap().to_rgba8();
    let p = img.get_pixel(20, 15).0;
    assert!(p[0] > 192 || p[3] < 64, "{p:?}");
}

#[test]
fn native_files_keep_the_page_table_and_stay_current_vectorcraft_files() {
    let mut s = doc();
    run(&mut s, "pages.enable", json!({}));
    run(&mut s, "pages.insert", json!({}));
    let saved = run(&mut s, "document.save", json!({}));
    let bytes = vectorcraft_format::base64_decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let head: Value = serde_json::from_slice(vectorcraft_format::json_part(&bytes)).unwrap();
    assert_eq!(head["format"], "vectorcraft");
    assert_eq!(head["version"], vectorcraft_format::VERSION, "no format version bump for pages");
    assert!(head["document"]["pages"]["order"].is_array());
    // Reopened, it is a pages document and not an old file.
    let mut o = Session::new();
    run(&mut o, "document.open", json!({"name": "Poster.vectorcraft", "dataBase64": saved["dataBase64"]}));
    assert!(!o.doc().unwrap().converted);
    assert_eq!(o.doc().unwrap().doc.pages, s.doc().unwrap().doc.pages);
    // A document that never used pages writes no `pages` key at all.
    let plain = run(&mut doc(), "document.save", json!({}));
    let bytes = vectorcraft_format::base64_decode(plain["dataBase64"].as_str().unwrap()).unwrap();
    let head: Value = serde_json::from_slice(vectorcraft_format::json_part(&bytes)).unwrap();
    assert!(head["document"].get("pages").is_none());
}
