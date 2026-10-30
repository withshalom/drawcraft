//! `.cdr` and CMX files: read by TraceDraw's reader (`tracedraw-cdr`, borrowed; see
//! `crates/tracedraw-cdr/README.md`) into TraceDraw's document model, then mapped here to editable
//! VectorCraft art. Each page becomes a page (an artboard of a pages document), side by side; the
//! layers of the pages become the document's layers, one per name; master layers become master
//! layers. Live effects (blends, contours, extrusions, envelopes…) and bitmap effects come in as
//! what they draw. What could not be mapped is listed in the warnings.
//!
//! TraceDraw's pages are in millimetres with y up from the page's bottom-left corner; VectorCraft's
//! document is in points with y down.

use std::sync::Arc;

use kurbo::{Affine, BezPath, PathEl, Point, Rect};
use tracedraw_core as td;
use vectorcraft_doc::pages::Pages;
use vectorcraft_doc::{Artboard, Document, LayerColor, Node, NodeKind};

use super::load::err;
use crate::Result;

mod paint;
mod shapes;
mod text;

/// Points per millimetre.
const K: f64 = 72.0 / 25.4;
/// Gap between pages, in points (as `pages.insert` leaves).
const PAGE_GAP: f64 = 20.0;
/// Deepest object tree mapped.
const MAX_DEPTH: usize = 160;

/// A `.cdr`, CMX (or older `WL`) file as a VectorCraft document, and what didn't come across.
pub(super) fn import(name: &str, bytes: &[u8]) -> Result<(Document, Vec<String>)> {
    let (src, report) = tracedraw_cdr::open_bytes(bytes, name).map_err(|e| err(format!("the file can't be read as a CDR or CMX drawing ({e})")))?;
    let mut b = Builder::new(&src);
    b.document()?;
    b.doc.limit_depth_noting(&mut b.warnings);
    let mut warnings: Vec<String> = report.warnings.iter().map(|w| format!("CDR: {w}.")).collect();
    warnings.extend(b.warnings.iter().map(|w| format!("CDR: {w}.")));
    Ok((b.doc, warnings))
}

/// One document's mapping.
struct Builder<'a> {
    src: &'a td::Document,
    doc: Document,
    warnings: Vec<String>,
    /// Layers made so far (their colours cycle).
    layers_made: u8,
}

/// A layer being filled from the pages that have one of its name.
struct LayerParts {
    name: String,
    visible: bool,
    locked: bool,
    printable: bool,
    children: Vec<Arc<Node>>,
}

impl<'a> Builder<'a> {
    fn new(src: &'a td::Document) -> Self {
        Self { src, doc: Document::new(0.0, 0.0), warnings: Vec::new(), layers_made: 0 }
    }

    fn warn(&mut self, w: &str) {
        if self.warnings.len() < 200 && !self.warnings.iter().any(|x| x == w) {
            self.warnings.push(w.to_string());
        }
    }

    fn document(&mut self) -> Result<()> {
        let src = self.src;
        let mut artboards = Vec::new();
        let mut page_layers: Vec<LayerParts> = Vec::new();
        let mut backgrounds: Vec<Arc<Node>> = Vec::new();
        let mut first_to_doc = None;
        let mut x = 0.0;
        for page in &src.pages {
            let (w, h) = (page.size.width * K, page.size.height * K);
            if !(w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 && x + w <= crate::MAX_COORD && h <= crate::MAX_COORD) {
                self.warn("pages with no size, or beyond the document range VectorCraft supports (left out)");
                continue;
            }
            // Page space (mm, y up from the bottom left) to the document (points, y down).
            let to_doc = Affine::new([K, 0.0, 0.0, -K, x, h]);
            first_to_doc.get_or_insert(to_doc);
            let id = u32::try_from(artboards.len() + 1).unwrap_or(u32::MAX);
            let name = if page.name.trim().is_empty() { format!("Page {}", artboards.len() + 1) } else { page.name.clone() };
            artboards.push(Artboard {
                id,
                name,
                rect: Rect::new(x, 0.0, x + w, h),
                locked: false,
                show_center_mark: false,
                show_cross_hairs: false,
                show_video_safe_areas: false,
                video_ruler_pixel_aspect_ratio: 1.0,
                color: None,
            });
            if let Some(fill) = &page.background
                && let Some(bg) = self.background(fill, page.rect(), to_doc)
            {
                backgrounds.push(bg);
            }
            for layer in &page.layers {
                let children = self.shapes(&layer.shapes, Affine::IDENTITY, to_doc, 0);
                match page_layers.iter_mut().find(|l| l.name == layer.name) {
                    Some(l) => l.children.extend(children),
                    None => page_layers.push(LayerParts {
                        name: layer.name.clone(),
                        visible: layer.visible,
                        locked: layer.locked,
                        printable: layer.printable,
                        children,
                    }),
                }
            }
            x += w + PAGE_GAP;
        }
        let Some(first_to_doc) = first_to_doc else {
            return Err(err("the drawing has no page with a size"));
        };
        if src.pages.iter().any(|p| !p.guides.is_empty()) {
            self.warn("guides (left out)");
        }
        if !src.fonts.is_empty() {
            self.warn("fonts embedded in the file (not used: text shows in the fonts installed here)");
        }
        // Bottom to top: the page backgrounds, the master layers (drawn beneath every page's own
        // layers), then the pages' layers.
        let mut layers = Vec::new();
        if !backgrounds.is_empty() {
            let l = self.layer(LayerParts { name: "Page Background".into(), visible: true, locked: true, printable: true, children: backgrounds });
            layers.push(l);
        }
        let mut master_ids = Vec::new();
        for m in &src.master {
            // The layers every drawing has (guides, desktop, grid) are only worth a layer here
            // with something on them.
            if m.shapes.is_empty() {
                continue;
            }
            if m.scope != td::MasterScope::All {
                self.warn("master layers shown on odd or even pages only (shown on every page)");
            }
            let children = self.shapes(&m.shapes, Affine::IDENTITY, first_to_doc, 0);
            let l = self.layer(LayerParts { name: m.name.clone(), visible: m.visible, locked: m.locked, printable: m.printable, children });
            master_ids.push(l.id);
            layers.push(l);
        }
        for parts in page_layers {
            layers.push(self.layer(parts));
        }
        if layers.is_empty() {
            layers.push(self.layer(LayerParts { name: "Layer 1".into(), visible: true, locked: false, printable: true, children: Vec::new() }));
        }
        self.doc.pages = Some(Pages { order: artboards.iter().map(|a| a.id).collect(), master_layers: master_ids }.bounded());
        self.doc.artboards = artboards;
        self.doc.layers = layers;
        Ok(())
    }

    fn layer(&mut self, parts: LayerParts) -> Arc<Node> {
        self.layers_made = self.layers_made.wrapping_add(1);
        let name = if parts.name.trim().is_empty() { format!("Layer {}", self.layers_made) } else { parts.name };
        let mut n = Node::layer(self.doc.alloc_id(), &name, LayerColor::Preset(self.layers_made % 9));
        n.visible = parts.visible;
        n.locked = parts.locked;
        if let NodeKind::Layer { printable, children, .. } = &mut n.kind {
            *printable = parts.printable;
            *children = parts.children;
        }
        Arc::new(n)
    }

    /// The nodes of `shapes`, bottom to top; `m` places their local space on the page.
    fn shapes(&mut self, shapes: &[td::Shape], m: Affine, to_doc: Affine, depth: usize) -> Vec<Arc<Node>> {
        shapes.iter().filter_map(|s| self.node(s, m, to_doc, depth)).collect()
    }
}

/// Whether `path` has a closed subpath (one ended with a close, or ending where it starts): open
/// curves are drawn without their fill.
fn has_closed_subpath(path: &BezPath) -> bool {
    // A subpath that drew something and came back to where it started.
    let closes = |start: Point, last: Point, drawn: bool| drawn && (start - last).hypot() < 1e-9;
    let (mut start, mut last, mut drawn) = (Point::ZERO, Point::ZERO, false);
    for el in path.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                if closes(start, last, drawn) {
                    return true;
                }
                (start, last, drawn) = (p, p, false);
            }
            PathEl::LineTo(p) | PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => (last, drawn) = (p, true),
            PathEl::ClosePath => return true,
        }
    }
    closes(start, last, drawn)
}

/// Whether every coordinate of `r` is finite and within the document range.
fn in_range(r: Rect) -> bool {
    [r.x0, r.y0, r.x1, r.y1].iter().all(|v| v.is_finite() && v.abs() <= crate::MAX_COORD)
}
