use vectorcraft_geom::{Rect, shapes};

use super::*;
use crate::{Appearance, Artboard};

fn artboard(id: u32, x: f64) -> Artboard {
    let mut a = Document::new(100.0, 100.0).artboards.remove(0);
    a.id = id;
    a.name = format!("A{id}");
    a.rect = Rect::new(x, 0.0, x + 100.0, 100.0);
    a
}

/// A document of three 100 × 100 artboards in a row (x = 0, 200, 400) and two layers, the
/// second ("Header") holding one rectangle at (10, 10).
fn three_pages() -> (Document, NodeId, NodeId) {
    let mut d = Document::new(100.0, 100.0);
    d.artboards = vec![artboard(1, 0.0), artboard(2, 200.0), artboard(3, 400.0)];
    let header = d.alloc_id();
    let mut layer = Node::layer(header, "Header", crate::LayerColor::Preset(1));
    let rect = Node::path(d.alloc_id(), shapes::rectangle(Rect::new(10.0, 10.0, 30.0, 20.0)), Appearance::default_art());
    let rect_id = rect.id;
    if let Some(ch) = layer.children_mut() {
        ch.push(Arc::new(rect));
    }
    d.layers.push(Arc::new(layer));
    d.drop_lookup_cache();
    (d, header, rect_id)
}

fn header_rects(d: &Document, header: NodeId) -> Vec<(NodeId, Rect)> {
    let l = d.layers.iter().find(|l| l.id == header).unwrap();
    l.children().unwrap().iter().map(|n| (n.id, n.geometric_bounds().unwrap())).collect()
}

#[test]
fn page_order_follows_the_table_then_the_rest() {
    let (mut d, _, _) = three_pages();
    assert_eq!(d.page_order(), [0, 1, 2], "no table: document order");
    // Listed first (3 then 1), an unknown id skipped, the unlisted artboard (2) after them.
    d.pages = Some(Pages { order: vec![3, 99, 1, 3], master_layers: vec![] });
    assert_eq!(d.page_order(), [2, 0, 1]);
    // An artboard deleted later is simply no longer a page.
    d.artboards.remove(2);
    assert_eq!(d.page_order(), [0, 1]);
}

#[test]
fn master_layers_are_existing_top_level_layers() {
    let (mut d, header, rect) = three_pages();
    assert!(d.master_layer_ids().is_empty() && !d.is_master_layer(header));
    d.pages = Some(Pages { order: vec![], master_layers: vec![header, rect, NodeId(12345)] });
    // The rectangle isn't a layer and 12345 isn't there: only the layer counts.
    assert_eq!(d.master_layer_ids(), [header]);
    assert!(d.is_master_layer(header) && !d.is_master_layer(rect));
}

#[test]
fn ordinary_documents_are_drawn_as_they_are() {
    let (mut d, header, _) = three_pages();
    assert!(matches!(d.with_master_pages(), Cow::Borrowed(_)), "no table");
    d.pages = Some(Pages::default());
    assert!(matches!(d.with_master_pages(), Cow::Borrowed(_)), "no master layer");
    d.pages = Some(Pages { order: vec![], master_layers: vec![header] });
    d.artboards.truncate(1);
    assert!(matches!(d.with_master_pages(), Cow::Borrowed(_)), "one page");
}

#[test]
fn master_art_repeats_on_every_page_in_page_order() {
    let (mut d, header, rect) = three_pages();
    // Page order 1, 3, 2: the offsets are measured from artboard 1, whatever the order.
    d.pages = Some(Pages { order: vec![1, 3, 2], master_layers: vec![header] });
    assert_eq!(d.master_offsets(), [Vec2::new(400.0, 0.0), Vec2::new(200.0, 0.0)]);
    let drawn = d.with_master_pages();
    let rects = header_rects(&drawn, header);
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0], (rect, Rect::new(10.0, 10.0, 30.0, 20.0)), "the original first, where it was");
    assert_eq!(rects[1].1, Rect::new(410.0, 10.0, 430.0, 20.0));
    assert_eq!(rects[2].1, Rect::new(210.0, 10.0, 230.0, 20.0));
    // The repeats have ids of their own, new to the document.
    let ids: std::collections::HashSet<NodeId> = rects.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids.len(), 3);
    assert!(rects[1..].iter().all(|(id, _)| d.node(*id).is_none()));
    // The document itself still holds the art once.
    assert_eq!(header_rects(&d, header).len(), 1);
}

#[test]
fn drawing_twice_adds_no_more_repeats() {
    let (mut d, header, _) = three_pages();
    d.pages = Some(Pages { order: vec![], master_layers: vec![header] });
    let once = d.with_master_pages().into_owned();
    assert!(once.master_layer_ids().is_empty(), "the drawn copy lists no master layers");
    let twice = once.with_master_pages();
    assert!(matches!(twice, Cow::Borrowed(_)));
    assert_eq!(header_rects(&twice, header).len(), 3);
}

#[test]
fn pages_save_only_when_used_and_round_trip() {
    let (mut d, header, _) = three_pages();
    let plain = serde_json::to_value(&d).unwrap();
    assert!(plain.get("pages").is_none(), "an artboards document saves as before");
    d.pages = Some(Pages { order: vec![2, 1], master_layers: vec![header] });
    let v = serde_json::to_value(&d).unwrap();
    assert_eq!(v["pages"], serde_json::json!({ "order": [2, 1], "masterLayers": [header.0] }));
    let back: Document = serde_json::from_value(v).unwrap();
    assert_eq!(back.pages, d.pages);
    // An empty table is still a pages document, written as an empty object.
    d.pages = Some(Pages::default());
    assert_eq!(serde_json::to_value(&d).unwrap()["pages"], serde_json::json!({}));
}

#[test]
fn a_file_s_table_is_bounded() {
    let many = Pages { order: (0..(MAX_PAGES as u32 + 50)).chain([1, 1]).collect(), master_layers: vec![NodeId(1); 3] };
    let b = many.bounded();
    assert_eq!(b.order.len(), MAX_PAGES);
    assert_eq!(b.master_layers, [NodeId(1)]);
}
