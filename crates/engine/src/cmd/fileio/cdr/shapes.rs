//! Objects: curves and shapes, groups, clip frames, tables and symbols, bitmaps, and the objects
//! live and bitmap effects draw.

use std::sync::Arc;

use kurbo::{Affine, Rect};
use tracedraw_core as td;
use vectorcraft_doc::appearance::Appearance;
use vectorcraft_doc::geom::{FillRule, PathData};
use vectorcraft_doc::{ImageObject, Node, NodeKind};

use super::paint::scale_of;
use super::{Builder, MAX_DEPTH, has_closed_subpath, in_range};

impl Builder<'_> {
    /// `shape` as a node; `m` places its parent's local space on the page, `to_doc` the page in
    /// the document.
    pub(super) fn node(&mut self, shape: &td::Shape, m: Affine, to_doc: Affine, depth: usize) -> Option<Arc<Node>> {
        if depth > MAX_DEPTH {
            self.warn("objects nested too deeply (left out)");
            return None;
        }
        // Bitmap effects: the picture they make, in the object's place.
        if shape.raster_fx.is_some() {
            if let Some(pic) = shape.drawn_as()
                && pic.raster_fx.is_none()
            {
                return self.node(&pic, m, to_doc, depth + 1);
            }
            self.warn("bitmap effects that could not be drawn (left out)");
        }
        // Live effects: what they draw, below and above the object as they leave it.
        if !shape.effects.is_empty() {
            let ev = td::live::evaluate(shape);
            let plain = |s: &td::Shape| {
                let mut s = s.clone();
                s.effects.clear();
                s.parked_effects.clear();
                s
            };
            let mut kids = Vec::new();
            for s in ev.below.iter().chain(std::iter::once(&ev.main)).chain(&ev.above) {
                if let Some(n) = self.node(&plain(s), m, to_doc, depth + 1) {
                    kids.push(n);
                }
            }
            return match kids.len() {
                0 => None,
                1 => kids.pop(),
                _ => {
                    let mut g = Node::group(self.doc.alloc_id(), kids);
                    g.name = shape.name.clone();
                    Some(Arc::new(g))
                }
            };
        }
        let full = m * shape.transform;
        let mut out = match &shape.kind {
            td::ShapeKind::Group { children } => {
                let kids = self.shapes(children, full, to_doc, depth + 1);
                if kids.is_empty() {
                    return None;
                }
                Node::new(self.doc.alloc_id(), NodeKind::Group { children: kids, clip: false })
            }
            td::ShapeKind::ClipFrame { frame, contents } => {
                // The frame clips the contents; as a clipping path it keeps its fill, drawn
                // behind them, and its outline, drawn over them.
                let mut frame_node = self.path_node(frame, full, to_doc)?;
                if let NodeKind::Path { clipping, .. } = &mut frame_node.kind {
                    *clipping = true;
                }
                let mut kids = vec![Arc::new(frame_node)];
                kids.extend(self.shapes(contents, full, to_doc, depth + 1));
                Node::new(self.doc.alloc_id(), NodeKind::Group { children: kids, clip: true })
            }
            td::ShapeKind::Table(_) | td::ShapeKind::SymbolInstance { .. } => {
                let parts = shape.expand(&self.src.symbols);
                let kids = self.shapes(&parts, full, to_doc, depth + 1);
                if kids.is_empty() {
                    return None;
                }
                let mut g = Node::new(self.doc.alloc_id(), NodeKind::Group { children: kids, clip: false });
                if matches!(shape.kind, td::ShapeKind::Table(_)) {
                    g.name = Some("Table".into());
                }
                g
            }
            td::ShapeKind::Text { .. } => self.text(shape, full, to_doc)?,
            td::ShapeKind::Bitmap { rect, png, .. } => self.image(*rect, png.as_slice(), full, to_doc)?,
            td::ShapeKind::Rect { .. } | td::ShapeKind::Ellipse { .. } | td::ShapeKind::Polygon { .. } | td::ShapeKind::Path { .. } => {
                return self.path_node(shape, m, to_doc).map(Arc::new);
            }
        };
        self.finish(shape, &mut out);
        Some(Arc::new(out))
    }

    /// A curve or shape (rectangle, ellipse, polygon) as a path with its fills and outlines.
    fn path_node(&mut self, shape: &td::Shape, m: Affine, to_doc: Affine) -> Option<Node> {
        let full = m * shape.transform;
        let local = shape.local_path();
        let page = full * local;
        let bez = to_doc * page.clone();
        let bounds = kurbo::Shape::bounding_box(&bez);
        if !in_range(bounds) {
            self.warn("objects beyond the document range VectorCraft supports (left out)");
            return None;
        }
        let filled = has_closed_subpath(&page);
        let appearance = self.appearance(shape, bounds, filled, scale_of(full));
        let data = PathData::from_bezpath(&bez);
        let mut n = Node::new(self.doc.alloc_id(), NodeKind::Path { path: data, rule: FillRule::EvenOdd, live: None, clipping: false, guide: false });
        n.appearance = appearance.into();
        self.finish(shape, &mut n);
        Some(n)
    }

    /// A bitmap placed in `rect` (local space, y up).
    fn image(&mut self, rect: Rect, png: &[u8], full: Affine, to_doc: Affine) -> Option<Node> {
        let r = match crate::cmd::fileio::load::raster_image(png) {
            Ok(r) => r,
            Err(_) => {
                self.warn("bitmaps that could not be decoded (left out)");
                return None;
            }
        };
        let (w, h) = (f64::from(r.width.max(1)), f64::from(r.height.max(1)));
        // Pixel space (y down from the top left) onto the rectangle.
        let xf = to_doc * full * Affine::translate((rect.x0, rect.y1)) * Affine::scale_non_uniform(rect.width() / w, -rect.height() / h);
        if !in_range(xf.transform_rect_bbox(Rect::new(0.0, 0.0, w, h))) {
            self.warn("objects beyond the document range VectorCraft supports (left out)");
            return None;
        }
        self.doc.images.entry(r.key.clone()).or_insert(r.blob);
        let obj = ImageObject { key: r.key, width: r.width, height: r.height, xf, link: None, placement: Default::default(), pixels: None };
        Some(Node::new(self.doc.alloc_id(), NodeKind::Image(Box::new(obj))))
    }

    /// A page background as a locked rectangle covering the page.
    pub(super) fn background(&mut self, fill: &td::Fill, page: Rect, to_doc: Affine) -> Option<Arc<Node>> {
        let bounds = to_doc.transform_rect_bbox(page);
        let paint = self.paint(fill, bounds)?;
        let data = PathData::from_bezpath(&kurbo::Shape::to_path(&bounds, 0.1));
        let mut n = Node::new(self.doc.alloc_id(), NodeKind::Path { path: data, rule: FillRule::NonZero, live: None, clipping: false, guide: false });
        n.appearance = Appearance { items: vec![vectorcraft_doc::appearance::AppearanceItem::Fill(vectorcraft_doc::appearance::FillLayer::new(paint))], ..Appearance::default() }.into();
        n.name = Some("Page Background".into());
        n.locked = true;
        Some(Arc::new(n))
    }

    /// What every object keeps: its name, visibility, lock, opacity and drop shadow.
    pub(super) fn finish(&mut self, shape: &td::Shape, n: &mut Node) {
        n.name = shape.name.clone().filter(|s| !s.trim().is_empty()).or(n.name.take());
        n.visible = shape.visible;
        n.locked = shape.locked;
        n.opacity = if shape.opacity.is_finite() { shape.opacity.clamp(0.0, 1.0) as f32 } else { 1.0 };
        if let Some(s) = &shape.shadow
            && let Some(fx) = self.shadow(s)
        {
            n.appearance.effects.push(fx);
        }
        if shape.opacity_mask.is_some() {
            self.warn("transparency masks of clip frames (imported as ordinary clipping)");
        }
    }
}
