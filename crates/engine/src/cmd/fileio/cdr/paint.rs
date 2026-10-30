//! Colours, fills, outlines and drop shadows.

use kurbo::{Affine, Point, Rect, Vec2};
use serde_json::json;
use tracedraw_core as td;
use vectorcraft_color::{Color, Gradient, GradientGeom, GradientKind, GradientPaint, GradientStop, Paint};
use vectorcraft_doc::appearance::{Appearance, AppearanceItem, Arrowhead, Dash, Effect, FillLayer, LineCap, LineJoin, StrokeAlign, StrokeLayer};

use super::{Builder, K};

/// A component in 0..=1 (0 when not finite).
fn unit(v: f32) -> f32 {
    if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 }
}

impl Builder<'_> {
    pub(super) fn color(&mut self, c: td::Color) -> Color {
        match c {
            td::Color::Rgb { r, g, b } => Color::Rgb { r: unit(r), g: unit(g), b: unit(b) },
            td::Color::Cmyk { c, m, y, k } => Color::Cmyk { c: unit(c), m: unit(m), y: unit(y), k: unit(k) },
            td::Color::Gray { v } => Color::Gray { k: 1.0 - unit(v) },
            td::Color::Lab { l, a, b } => {
                let f = |v: f32, lo: f32, hi: f32| if v.is_finite() { v.clamp(lo, hi) } else { 0.0 };
                Color::Lab { l: f(l, 0.0, 100.0), a: f(a, -128.0, 127.0), b: f(b, -128.0, 127.0) }
            }
            td::Color::Registration => Color::Cmyk { c: 1.0, m: 1.0, y: 1.0, k: 1.0 },
            td::Color::Spot { tint, c, m, y, k, .. } => {
                self.warn("spot colours (imported as their process colours)");
                let t = unit(tint);
                Color::Cmyk { c: unit(c) * t, m: unit(m) * t, y: unit(y) * t, k: unit(k) * t }
            }
            // HSB, HSL and YIQ colours: the screen colour they stand for.
            other => {
                let [r, g, b] = other.to_rgb_f32();
                Color::Rgb { r: unit(r), g: unit(g), b: unit(b) }
            }
        }
    }

    /// `fill` as a paint over `bounds` (the object's box in the document); `None` for no fill.
    pub(super) fn paint(&mut self, fill: &td::Fill, bounds: Rect) -> Option<Paint> {
        match fill {
            td::Fill::None => None,
            td::Fill::Solid(c) => Some(Paint::solid(self.color(*c))),
            td::Fill::Fountain(f) => Some(self.fountain(f, bounds)),
            td::Fill::Pattern(p) => {
                self.warn("pattern fills (imported as a flat colour)");
                let c = match p {
                    td::Pattern::TwoColor { front, .. } => self.color(*front),
                    _ => Color::Gray { k: 0.5 },
                };
                Some(Paint::solid(c))
            }
            td::Fill::Texture(t) => {
                self.warn("texture fills (imported as a flat colour)");
                Some(Paint::solid(self.color(t.color_a)))
            }
            td::Fill::Mesh(m) => {
                self.warn("mesh fills (imported as a flat colour)");
                let n = m.nodes.len().max(1) as f32;
                let [mut r, mut g, mut b] = [0.0f32; 3];
                for node in &m.nodes {
                    let [nr, ng, nb] = node.color.to_rgb_f32();
                    (r, g, b) = (r + unit(nr), g + unit(ng), b + unit(nb));
                }
                Some(Paint::solid(if m.nodes.is_empty() { Color::Gray { k: 0.5 } } else { Color::Rgb { r: r / n, g: g / n, b: b / n } }))
            }
        }
    }

    /// A fountain fill as a gradient over `bounds`: its stops with the edge pad, repeats, mirror
    /// and inversion worked in; placed by its angle and centre offset, or by its free nodes.
    fn fountain(&mut self, f: &td::Fountain, bounds: Rect) -> Paint {
        let mut stops = f.resolved_stops();
        // A shaped fountain (0 on the edge) runs from the centre out, as a radial one.
        if f.kind == td::FountainKind::Shaped {
            stops.reverse();
            for s in &mut stops {
                s.pos = 1.0 - s.pos;
            }
        }
        let stops: Vec<GradientStop> = stops
            .iter()
            .map(|s| {
                let pos = if s.pos.is_finite() { s.pos.clamp(0.0, 1.0) as f32 } else { 0.0 };
                let mut stop = GradientStop::new(pos, self.color(s.color));
                stop.opacity = unit(s.opacity);
                stop
            })
            .collect();
        let kind = match f.kind {
            td::FountainKind::Linear => GradientKind::Linear,
            td::FountainKind::Radial | td::FountainKind::Shaped => GradientKind::Radial,
            td::FountainKind::Conical | td::FountainKind::Square => {
                self.warn("conical and square fountain fills (imported as linear gradients)");
                GradientKind::Linear
            }
        };
        let (c, w, h) = (bounds.center(), bounds.width(), bounds.height());
        let mut geom = None;
        // Nodes placed freely: the fill's own frame (start, end and the skew handle), in the
        // object's box with y down.
        if let Some(handles) = f.handles.filter(|_| f.kind != td::FountainKind::Shaped) {
            let at = |p: Point| Point::new(c.x + p.x * w / 2.0, c.y - p.y * h / 2.0);
            let (s, e, k) = (at(handles.start), at(handles.end), at(handles.skew));
            let (u, v): (Vec2, Vec2) = (e - s, k - s);
            if u.hypot() > 1e-9 && [s.x, s.y, e.x, e.y].iter().all(|x| x.is_finite()) {
                let aspect = (v.hypot() / u.hypot()).clamp(0.01, 100.0);
                geom = Some(GradientGeom { start: s, end: e, aspect: if kind == GradientKind::Radial { aspect } else { 1.0 }, focal: None });
            }
        }
        let geom = geom.unwrap_or_else(|| match kind {
            GradientKind::Radial if f.kind == td::FountainKind::Shaped => {
                // A radial gradient reaching the nearer sides stands in for one following the outline.
                let r = w.min(h).max(1e-6) / 2.0;
                GradientGeom { start: c, end: c + Vec2::new(r, 0.0), aspect: 1.0, focal: None }
            }
            GradientKind::Radial => {
                let at = Point::new(c.x + f.offset.x * w / 2.0, c.y - f.offset.y * h / 2.0);
                let r = w.max(h) / 2.0 * std::f64::consts::SQRT_2;
                GradientGeom { start: at, end: at + Vec2::new(r, 0.0), aspect: 1.0, focal: None }
            }
            _ => {
                let a = if f.angle.is_finite() { f.angle.to_radians() } else { 0.0 };
                let start = Point::new(bounds.x0 + w * (0.5 - a.cos() / 2.0), bounds.y0 + h * (0.5 + a.sin() / 2.0));
                let end = Point::new(bounds.x0 + w * (0.5 + a.cos() / 2.0), bounds.y0 + h * (0.5 - a.sin() / 2.0));
                GradientGeom { start, end, aspect: 1.0, focal: None }
            }
        });
        let mut gp = GradientPaint::new(Gradient::new(kind, stops));
        let d = geom.end - geom.start;
        gp.angle = (-d.y).atan2(d.x).to_degrees();
        gp.geom = Some(geom);
        Paint::Gradient(Box::new(gp))
    }

    /// One outline as a stroke; `scale` is how much the object's transform enlarges it.
    fn stroke_layer(&mut self, s: &td::Stroke, bounds: Rect, scale: f64) -> Option<StrokeLayer> {
        let paint = match s.fill.as_deref() {
            Some(fill) if !matches!(fill, td::Fill::None | td::Fill::Solid(_)) => self.paint(fill, bounds)?,
            _ => Paint::solid(self.color(s.color)),
        };
        let width = s.width * K * if s.scale_with_object { scale } else { 1.0 };
        if !(width.is_finite() && width > 0.0) {
            return None;
        }
        let mut l = StrokeLayer::new(paint, width.min(1000.0));
        l.cap = match s.cap {
            td::LineCap::Butt => LineCap::Butt,
            td::LineCap::Round => LineCap::Round,
            td::LineCap::Square => LineCap::Square,
        };
        l.join = match s.join {
            td::LineJoin::Miter => LineJoin::Miter,
            td::LineJoin::Round => LineJoin::Round,
            td::LineJoin::Bevel => LineJoin::Bevel,
        };
        // The file's mitre limit is an angle: corners sharper than it are bevelled. As a ratio of
        // the mitre's length to the width: 1 / sin(angle / 2).
        let half = (s.miter_limit.to_radians() / 2.0).sin();
        l.miter_limit = if half.is_finite() && half > 1e-3 { (1.0 / half).clamp(1.0, 500.0) } else { 10.0 };
        l.align = match s.position {
            td::OutlinePosition::Center => StrokeAlign::Center,
            td::OutlinePosition::Inside => StrokeAlign::Inside,
            td::OutlinePosition::Outside => StrokeAlign::Outside,
        };
        // Dashes are in multiples of the width.
        let dash: Vec<f64> = s.dash.iter().filter(|v| v.is_finite() && **v >= 0.0).map(|v| (v * width).min(10_000.0)).take(6).collect();
        if dash.iter().any(|v| *v > 0.0) {
            l.dash = Some(Dash { pattern: dash, offset: 0.0, align_corners: s.align_dashes });
        }
        l.start_arrow = self.arrowhead(&s.start_arrow);
        l.end_arrow = self.arrowhead(&s.end_arrow);
        if s.stretch < 0.999 || !s.widths.is_empty() || s.painterly.is_some() {
            self.warn("calligraphic, variable-width and brushed outlines (imported as plain outlines)");
        }
        Some(l)
    }

    fn arrowhead(&mut self, a: &td::Arrowhead) -> Option<Arrowhead> {
        Some(match a {
            td::Arrowhead::None => return None,
            td::Arrowhead::Arrow => Arrowhead::Arrow,
            td::Arrowhead::OpenArrow => Arrowhead::ArrowOpen,
            td::Arrowhead::Circle => Arrowhead::Circle,
            td::Arrowhead::Square => Arrowhead::Square,
            td::Arrowhead::Bar => Arrowhead::Bar,
            td::Arrowhead::Diamond => Arrowhead::Diamond,
            td::Arrowhead::Custom { .. } => {
                self.warn("custom arrowheads (imported as triangles)");
                Arrowhead::Triangle
            }
        })
    }

    /// The fills and outlines of `shape` over `bounds` (its box in the document); `filled` is
    /// false for open curves, drawn without their fill; `scale` as for [`Self::stroke_layer`].
    pub(super) fn appearance(&mut self, shape: &td::Shape, bounds: Rect, filled: bool, scale: f64) -> Appearance {
        let mut items = Vec::new();
        if filled {
            for f in std::iter::once(&shape.fill).chain(&shape.extra_fills) {
                if let Some(paint) = self.paint(f, bounds) {
                    items.push(AppearanceItem::Fill(FillLayer::new(paint)));
                }
            }
        }
        let mut behind = Vec::new();
        for s in shape.stroke.iter().chain(&shape.extra_strokes) {
            let Some(l) = self.stroke_layer(s, bounds, scale) else { continue };
            if s.behind_fill {
                behind.push(AppearanceItem::Stroke(l));
            } else {
                items.push(AppearanceItem::Stroke(l));
            }
        }
        behind.extend(items);
        Appearance { items: behind, ..Appearance::default() }
    }

    /// A drop shadow as the drop shadow effect.
    pub(super) fn shadow(&mut self, s: &td::Shadow) -> Option<Effect> {
        let num = |v: f64| if v.is_finite() { (v * 1000.0).round() / 1000.0 } else { 0.0 };
        if s.perspective.is_some() {
            self.warn("perspective shadows (imported as flat drop shadows)");
        }
        let color = self.color(s.color);
        let mode = if matches!(s.merge, td::live::MergeMode::Multiply) { "multiply" } else { "normal" };
        Some(Effect {
            id: "stylize.dropShadow".into(),
            params: json!({
                "mode": mode,
                "opacity": num(s.opacity.clamp(0.0, 1.0) * 100.0),
                // The file's offset is in millimetres with y up.
                "x": num((s.offset.x * K).clamp(-1e4, 1e4)),
                "y": num((-s.offset.y * K).clamp(-1e4, 1e4)),
                "blur": num((s.blur * K).clamp(0.0, 1000.0)),
                "color": color,
            }),
            visible: true,
        })
    }
}

/// How much `m` enlarges lengths (the square root of its area scale).
pub(super) fn scale_of(m: Affine) -> f64 {
    let [a, b, c, d, ..] = m.as_coeffs();
    let s = (a * d - b * c).abs().sqrt();
    if s.is_finite() { s } else { 1.0 }
}
