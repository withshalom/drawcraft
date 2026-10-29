//! Page commands (draw mode). A page is an artboard; a pages document has a page order and master
//! layers over its artboards ([`vectorcraft_doc::pages`]). The artboard commands keep working on
//! the pages.

use serde_json::{Value, json};
use vectorcraft_doc::pages::{MAX_MASTER_LAYERS, MAX_PAGES, Pages};
use vectorcraft_doc::{Artboard, Document};
use vectorcraft_geom::Rect;

use super::{CommandSpec, Result, Session, bad, bool_or, check_numbers, check_switches, cmd, has_doc, index, number_or, ok};
use crate::EngineError;

/// Gap between a new page and the artboard left of it, in points (as New Artboard leaves).
const PAGE_GAP: f64 = 20.0;
/// Most pages one Insert Pages adds.
const MAX_INSERT: u64 = 100;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "pages.enable",
            "Use Pages",
            [],
            None,
            "{} make the document a pages document: its artboards become its pages, in their order, and layers can be made master layers (pages.setMaster) whose art repeats on every page; one undo step → {pages: count}",
            not_paged,
            enable
        ),
        cmd!(
            "pages.disable",
            "Use Artboards Only",
            [],
            None,
            "{} make the document an artboards document again: the artboards stay where they are, the page order and master layers are dropped (master layers become ordinary layers, their art once, on the first page); one undo step",
            paged,
            disable
        ),
        cmd!(
            query "pages.list",
            "List Pages",
            [],
            None,
            "{} → {paged: bool, pages: [{number (1-based), index (the artboard's index), id (the artboard's id), name, x, y, width, height}], masterLayers: [{id, name}]} the pages in page order (an artboards document: its artboards, in order)",
            has_doc,
            list
        ),
        cmd!(
            "pages.insert",
            "Insert Pages",
            [],
            None,
            "{after?: page number (1-based; default: the last page; 0: before the first), count?: 1 to 100 (default 1), width?, height? (default: the size of the page they follow, else the first page)} add pages: new artboards placed right of the rightmost artboard, in a row, and put in the page order after page `after`; a document without pages becomes a pages document (its artboards its first pages, as pages.enable makes them) in the same step; one undo step → {numbers: the new pages' numbers, indices: their artboards' indices}",
            has_doc,
            insert
        ),
        cmd!(
            "pages.move",
            "Move Page",
            [],
            None,
            "{page: number (1-based), to: number (1-based; past the end: last)} change a page's place in the page order (the artboard doesn't move on the canvas); a document without pages becomes a pages document in the same step; one undo step",
            has_doc,
            move_page
        ),
        cmd!(
            "pages.setMaster",
            "Toggle Master Layer",
            [],
            None,
            "{id?: a top-level layer's id (default: the current layer), master?: bool (default: the opposite of what it is)} make a layer a master layer, whose art repeats on every page at the same place relative to each page's top-left corner as on the first page (canvas, export and print draw the repeats; the document holds the art once), or an ordinary layer again; at most 64 master layers; one undo step",
            paged,
            set_master
        ),
    ]
}

fn paged(s: &Session) -> std::result::Result<(), String> {
    let st = s.active().ok_or("no document open")?;
    if st.doc.is_paged() { Ok(()) } else { Err("the document doesn't use pages (pages.enable)".into()) }
}

fn not_paged(s: &Session) -> std::result::Result<(), String> {
    let st = s.active().ok_or("no document open")?;
    if st.doc.is_paged() { Err("the document already uses pages".into()) } else { Ok(()) }
}

fn need_paged(d: &Document, cmd: &str) -> Result<()> {
    if d.is_paged() { Ok(()) } else { Err(bad(cmd, "the document doesn't use pages: run pages.enable first")) }
}

/// The page table of `d`, made from its artboards' order when it lists none yet.
fn table(d: &Document) -> Pages {
    let order = d.page_order().into_iter().filter_map(|i| d.artboards.get(i)).map(|a| a.id).collect();
    let master_layers = d.master_layer_ids();
    Pages { order, master_layers }
}

fn enable(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "pages.enable";
    let _ = p;
    if s.doc()?.doc.is_paged() {
        return Err(bad(C, "the document already uses pages"));
    }
    if s.doc()?.doc.artboards.len() > MAX_PAGES {
        return Err(bad(C, format!("a document of pages has at most {MAX_PAGES} pages")));
    }
    let n = s.edit("Use Pages", |d, _| {
        d.pages = Some(Pages { order: d.artboards.iter().map(|a| a.id).collect(), master_layers: vec![] });
        Ok(d.artboards.len())
    })?;
    Ok(json!({ "pages": n }))
}

fn disable(s: &mut Session, _: &Value) -> Result<Value> {
    need_paged(&s.doc()?.doc, "pages.disable")?;
    s.edit("Use Artboards Only", |d, _| {
        d.pages = None;
        Ok(())
    })?;
    ok()
}

fn list(s: &mut Session, _: &Value) -> Result<Value> {
    let d = &s.doc()?.doc;
    let pages: Vec<Value> = d
        .page_order()
        .into_iter()
        .enumerate()
        .filter_map(|(n, i)| {
            let a = d.artboards.get(i)?;
            let r = a.rect.abs();
            Some(json!({ "number": n + 1, "index": i, "id": a.id, "name": a.name, "x": r.x0, "y": r.y0, "width": r.width(), "height": r.height() }))
        })
        .collect();
    let masters: Vec<Value> = d
        .master_layer_ids()
        .into_iter()
        .filter_map(|id| d.layers.iter().find(|l| l.id == id))
        .map(|l| json!({ "id": l.id.0, "name": l.name.clone().unwrap_or_default() }))
        .collect();
    Ok(json!({ "paged": d.is_paged(), "pages": pages, "masterLayers": masters }))
}

/// A page number `p` gives for `key` (1-based) as a position in the page order (0-based).
fn page_number(p: &Value, key: &str, pages: usize, cmd: &str) -> Result<usize> {
    let n = p.get(key).and_then(Value::as_u64).ok_or_else(|| bad(cmd, format!("missing page number `{key}`")))?;
    let n = index(n);
    if n == 0 || n > pages {
        return Err(bad(cmd, format!("no page {n}: the document has {pages}")));
    }
    Ok(n - 1)
}

fn insert(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "pages.insert";
    check_numbers(p, &["width", "height"], C)?;
    let d0 = &s.doc()?.doc;
    let order = d0.page_order();
    let pages = order.len();
    let after = match p.get("after") {
        None | Some(Value::Null) => pages,
        Some(v) => {
            let n = v.as_u64().map(index).ok_or_else(|| bad(C, "`after` must be a page number"))?;
            if n > pages {
                return Err(bad(C, format!("no page {n}: the document has {pages}")));
            }
            n
        }
    };
    let count = match p.get("count") {
        None | Some(Value::Null) => 1,
        Some(v) => v.as_u64().filter(|n| (1..=MAX_INSERT).contains(n)).ok_or_else(|| bad(C, format!("`count` must be 1 to {MAX_INSERT}")))?,
    };
    let count = index(count);
    if pages.saturating_add(count) > MAX_PAGES {
        return Err(bad(C, format!("a document of pages has at most {MAX_PAGES} pages")));
    }
    // The size: given, else that of the page the new ones follow, else the first page's.
    let reference = order.get(after.saturating_sub(1)).or(order.first()).and_then(|i| d0.artboards.get(*i)).map(|a| a.rect.abs());
    let reference = reference.unwrap_or(Rect::new(0.0, 0.0, 612.0, 792.0));
    let w = number_or(p, "width", reference.width(), C)?;
    let h = number_or(p, "height", reference.height(), C)?;
    if !(w.is_finite() && h.is_finite() && w >= 1.0 && h >= 1.0 && w <= vectorcraft_doc::MAX_COORD && h <= vectorcraft_doc::MAX_COORD) {
        return Err(bad(C, "`width` and `height` must be at least 1 pt"));
    }
    let (numbers, indices) = s.edit("Insert Pages", |d, _| {
        let mut table = table(d);
        let right = d.artboards.iter().map(|a| a.rect.abs().x1).fold(f64::NEG_INFINITY, f64::max);
        let top = d.artboards.iter().map(|a| a.rect.abs().y0).fold(f64::INFINITY, f64::min);
        let (mut x, y) = (if right.is_finite() { right + PAGE_GAP } else { 0.0 }, if top.is_finite() { top } else { 0.0 });
        let at = after.min(table.order.len());
        let mut indices = Vec::with_capacity(count);
        let mut ids = Vec::with_capacity(count);
        for k in 0..count {
            if x + w > vectorcraft_doc::MAX_COORD {
                return Err(EngineError::Other("no room on the canvas for another page".into()));
            }
            let id = d.next_artboard_id();
            // Named by the number it gets (the pages after it are numbered on).
            let number = at + k + 1;
            d.artboards.push(Artboard {
                id,
                name: format!("Page {number}"),
                rect: Rect::new(x, y, x + w, y + h),
                locked: false,
                show_center_mark: false,
                show_cross_hairs: false,
                show_video_safe_areas: false,
                video_ruler_pixel_aspect_ratio: 1.0,
                color: None,
            });
            indices.push(d.artboards.len() - 1);
            ids.push(id);
            x += w + PAGE_GAP;
        }
        table.order.splice(at..at, ids);
        d.pages = Some(table);
        let numbers: Vec<usize> = (at + 1..=at + count).collect();
        Ok((numbers, indices))
    })?;
    Ok(json!({ "numbers": numbers, "indices": indices }))
}

fn move_page(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "pages.move";
    let d0 = &s.doc()?.doc;
    let pages = d0.page_order().len();
    let from = page_number(p, "page", pages, C)?;
    let to = p.get("to").and_then(Value::as_u64).map(index).ok_or_else(|| bad(C, "missing page number `to`"))?;
    if to == 0 {
        return Err(bad(C, "`to` is a page number, from 1"));
    }
    s.edit("Move Page", |d, _| {
        let mut table = table(d);
        if from >= table.order.len() {
            return Err(EngineError::Other("no such page".into()));
        }
        let id = table.order.remove(from);
        let to = (to - 1).min(table.order.len());
        table.order.insert(to, id);
        d.pages = Some(table);
        Ok(())
    })?;
    ok()
}

fn set_master(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "pages.setMaster";
    check_switches(p, &["master"], C)?;
    let st = s.doc()?;
    need_paged(&st.doc, C)?;
    let id = match p.get("id") {
        None | Some(Value::Null) => st.current_layer().ok_or_else(|| bad(C, "no current layer: give a layer's `id`"))?,
        Some(v) => v.as_u64().map(vectorcraft_doc::NodeId).ok_or_else(|| bad(C, "`id` must be a layer's id"))?,
    };
    if !st.doc.layers.iter().any(|l| l.id == id && l.is_layer()) {
        return Err(bad(C, format!("{id} is not a top-level layer")));
    }
    let now = st.doc.is_master_layer(id);
    let master = bool_or(p, "master", !now);
    if now == master {
        return ok();
    }
    if master && st.doc.master_layer_ids().len() >= MAX_MASTER_LAYERS {
        return Err(bad(C, format!("a document has at most {MAX_MASTER_LAYERS} master layers")));
    }
    s.edit(if master { "Make Master Layer" } else { "Make Ordinary Layer" }, |d, _| {
        let mut table = table(d);
        table.master_layers.retain(|l| *l != id);
        if master {
            table.master_layers.push(id);
        }
        d.pages = Some(table);
        Ok(())
    })?;
    ok()
}

#[cfg(test)]
mod tests;
