//! `.cdr` and CMX files through Open and Place. The drawings are made here in TraceDraw's model and
//! written by TraceDraw's `.cdr` writer: no file made with any other application is used.

use super::*;
use std::sync::Arc;
use tracedraw_core as td;
use vectorcraft_color::Paint;
use vectorcraft_doc::appearance::AppearanceItem;
use vectorcraft_doc::{Node, NodeKind};

const K: f64 = 72.0 / 25.4;

fn rect_shape(doc: &mut td::Document, rect: td::Rect, fill: td::Color, outline: Option<(td::Color, f64)>) -> td::Shape {
    let id = doc.ids_mut().shape();
    let mut s = td::Shape::new(id, td::ShapeKind::Rect { rect, radius: 0.0, corners: None });
    s.fill = td::Fill::Solid(fill);
    s.stroke = outline.map(|(color, width)| td::Stroke { color, width, ..td::Stroke::default() });
    s
}

/// Two A5 pages: a red rectangle with a blue 1 mm outline at the bottom left of the first, an
/// artistic text on the second.
fn drawing() -> td::Document {
    let mut doc = td::Document::new("Test", td::Size::new(148.0, 210.0));
    let red = td::Color::Rgb { r: 1.0, g: 0.0, b: 0.0 };
    let blue = td::Color::Rgb { r: 0.0, g: 0.0, b: 1.0 };
    let r = rect_shape(&mut doc, td::Rect::new(10.0, 10.0, 60.0, 40.0), red, Some((blue, 1.0)));
    doc.pages[0].layers[0].shapes.push(r);
    let page = doc.ids_mut().page();
    let layer = doc.ids_mut().layer();
    let id = doc.ids_mut().shape();
    let mut text = td::Shape::new(
        id,
        td::ShapeKind::Text {
            spans: vec![td::TextSpan::new("Hello pages", "Sans", 24.0)],
            origin: td::Point::new(20.0, 100.0),
            frame: None,
            align: td::TextAlign::Left,
            para: td::ParagraphStyle::default(),
            on_path: None,
        },
    );
    text.fill = td::Fill::Solid(td::Color::BLACK);
    let mut l = td::Layer::new(layer, "Layer 1");
    l.shapes.push(text);
    doc.pages.push(td::Page { id: page, name: "Back".into(), size: td::Size::new(148.0, 210.0), layers: vec![l], guides: vec![], background: None });
    doc
}

fn cdr(doc: &td::Document) -> Vec<u8> {
    tracedraw_cdr::write::document_to_cdr(doc)
}

/// Every node under `n`, depth first.
fn all(n: &Arc<Node>, out: &mut Vec<Arc<Node>>) {
    out.push(n.clone());
    for c in n.children().into_iter().flatten() {
        all(c, out);
    }
}

fn nodes(doc: &vectorcraft_doc::Document) -> Vec<Arc<Node>> {
    let mut out = Vec::new();
    for l in &doc.layers {
        all(l, &mut out);
    }
    out
}

fn rgb(c: &vectorcraft_color::Color) -> (i32, i32, i32) {
    let h = c.to_hex();
    let h = h.trim_start_matches('#');
    let p = |i: usize| i32::from_str_radix(&h[i..i + 2], 16).unwrap();
    (p(0), p(2), p(4))
}

fn near(a: (i32, i32, i32), b: (i32, i32, i32)) -> bool {
    (a.0 - b.0).abs() <= 3 && (a.1 - b.1).abs() <= 3 && (a.2 - b.2).abs() <= 3
}

#[test]
fn pages_open_side_by_side_as_a_pages_document() {
    let mut s = Session::new();
    let result = open_bytes(&mut s, "drawing.cdr", &cdr(&drawing()), None).unwrap();
    assert_eq!(result["format"], "cdr");
    let doc = &s.doc().unwrap().doc;
    assert_eq!(doc.artboards.len(), 2);
    let (a, b) = (&doc.artboards[0], &doc.artboards[1]);
    assert!((a.rect.width() - 148.0 * K).abs() < 0.5 && (a.rect.height() - 210.0 * K).abs() < 0.5, "{:?}", a.rect);
    assert!(b.rect.x0 > a.rect.x1, "the second page is right of the first: {:?} {:?}", a.rect, b.rect);
    assert_eq!(b.name, "Back");
    let pages = doc.pages.as_ref().expect("a pages document");
    assert_eq!(pages.order, vec![a.id, b.id]);
    // One "Layer 1" holds both pages' objects.
    let names: Vec<_> = doc.layers.iter().filter_map(|l| l.name.clone()).collect();
    assert_eq!(names.iter().filter(|n| *n == "Layer 1").count(), 1, "{names:?}");
}

#[test]
fn a_rectangle_keeps_its_place_fill_and_outline() {
    let mut s = Session::new();
    open_bytes(&mut s, "drawing.cdr", &cdr(&drawing()), None).unwrap();
    let doc = &s.doc().unwrap().doc;
    let path = nodes(doc).into_iter().find(|n| matches!(n.kind, NodeKind::Path { .. })).expect("the rectangle");
    let NodeKind::Path { path: data, .. } = &path.kind else { panic!("a path") };
    let r = kurbo::Shape::bounding_box(&data.to_bezpath());
    // 10..60 mm across, 10..40 mm up from the page's bottom: y down from its top here.
    let page_h = 210.0 * K;
    assert!((r.x0 - 10.0 * K).abs() < 0.5 && (r.x1 - 60.0 * K).abs() < 0.5, "{r:?}");
    assert!((r.y1 - (page_h - 10.0 * K)).abs() < 0.5 && (r.y0 - (page_h - 40.0 * K)).abs() < 0.5, "{r:?}");
    let items = &path.appearance.items;
    let fill = items.iter().find_map(|i| if let AppearanceItem::Fill(f) = i { Some(f) } else { None }).expect("a fill");
    let Paint::Solid { color, .. } = &fill.paint else { panic!("solid fill: {:?}", fill.paint) };
    assert!(near(rgb(color), (255, 0, 0)), "{color:?}");
    let stroke = items.iter().find_map(|i| if let AppearanceItem::Stroke(s) = i { Some(s) } else { None }).expect("an outline");
    assert!((stroke.width - K).abs() < 0.05, "1 mm: {}", stroke.width);
    let Paint::Solid { color, .. } = &stroke.paint else { panic!("solid outline") };
    assert!(near(rgb(color), (0, 0, 255)), "{color:?}");
}

#[test]
fn artistic_text_comes_in_as_type_on_its_page() {
    let mut s = Session::new();
    open_bytes(&mut s, "drawing.cdr", &cdr(&drawing()), None).unwrap();
    let doc = &s.doc().unwrap().doc;
    let text = nodes(doc).into_iter().find_map(|n| if let NodeKind::Text(t) = &n.kind { Some(t.clone()) } else { None }).expect("the text");
    assert_eq!(text.plain_text(), "Hello pages");
    let size = text.runs[0].style.size;
    assert!((size - 24.0).abs() < 0.01, "{size}");
    // On the second page, its baseline 100 mm up from the page's bottom.
    let at = text.xf * kurbo::Point::ZERO;
    let page = doc.artboards[1].rect;
    assert!(page.contains(at), "{at:?} on {page:?}");
    assert!((at.y - (210.0 - 100.0) * K).abs() < 1.0, "{at:?}");
}

#[test]
fn a_cdr_file_is_found_by_its_content() {
    let mut s = Session::new();
    let result = open_bytes(&mut s, "renamed.bin", &cdr(&drawing()), None).unwrap();
    assert_eq!(result["format"], "cdr");
    let f = format("cdr").unwrap();
    assert!(f.read && !f.write);
    assert!(OPEN_EXTS.contains(&"cdr") && OPEN_EXTS.contains(&"cmx") && PLACE_EXTS.contains(&"cdr"));
}

#[test]
fn master_layers_become_master_layers() {
    let mut doc = drawing();
    let layer = doc.ids_mut().layer();
    let mut m = td::Layer::new(layer, "Letterhead");
    let r = rect_shape(&mut doc, td::Rect::new(0.0, 200.0, 148.0, 210.0), td::Color::Rgb { r: 0.0, g: 0.5, b: 0.0 }, None);
    m.shapes.push(r);
    doc.master.push(m);
    let mut s = Session::new();
    open_bytes(&mut s, "master.cdr", &cdr(&doc), None).unwrap();
    let doc = &s.doc().unwrap().doc;
    let master = doc.layers.iter().find(|l| l.name.as_deref() == Some("Letterhead")).expect("the master layer");
    assert!(doc.is_master_layer(master.id));
    assert!(!master.children().unwrap().is_empty());
}

#[test]
fn damaged_files_open_or_fail_with_a_message() {
    let bytes = cdr(&drawing());
    let cuts = [0, 4, 12, 64, 200, bytes.len() / 3, bytes.len() / 2, bytes.len() - 1];
    for cut in cuts {
        let mut s = Session::new();
        let _ = open_bytes(&mut s, "cut.cdr", &bytes[..cut.min(bytes.len())], None);
    }
    // Flipped bytes through the middle of the file.
    for at in (0..bytes.len()).step_by((bytes.len() / 97).max(1)) {
        let mut b = bytes.clone();
        b[at] ^= 0x5a;
        let mut s = Session::new();
        let _ = open_bytes(&mut s, "flipped.cdr", &b, None);
    }
}
