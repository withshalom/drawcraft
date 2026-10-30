//! Artistic text (point type) and paragraph text (area type).

use kurbo::{Affine, Rect};
use tracedraw_core as td;
use vectorcraft_color::Paint;
use vectorcraft_doc::geom::shapes;
use vectorcraft_doc::text::{CharStyle, Justify, TextKind, TextObject, TextRun};
use vectorcraft_doc::{Node, NodeKind};

use super::{Builder, K, in_range};

/// Most characters one text object brings in (a file is untrusted).
const MAX_CHARS: usize = 1 << 20;

fn justify(a: td::TextAlign) -> Justify {
    match a {
        td::TextAlign::Left | td::TextAlign::None => Justify::Left,
        td::TextAlign::Center => Justify::Center,
        td::TextAlign::Right => Justify::Right,
        td::TextAlign::Justify => Justify::JustifyLeft,
        td::TextAlign::ForceJustify => Justify::JustifyAll,
    }
}

/// The style a font's weight and slant name.
fn font_style(bold: bool, italic: bool) -> &'static str {
    match (bold, italic) {
        (false, false) => "Regular",
        (true, false) => "Bold",
        (false, true) => "Italic",
        (true, true) => "Bold Italic",
    }
}

impl Builder<'_> {
    pub(super) fn text(&mut self, shape: &td::Shape, full: Affine, to_doc: Affine) -> Option<Node> {
        let td::ShapeKind::Text { spans, origin, frame, align, para, on_path } = &shape.kind else { return None };
        if on_path.is_some() {
            self.warn("text fitted to a path (imported as ordinary text)");
        }
        let bounds = to_doc.transform_rect_bbox(full.transform_rect_bbox(kurbo::Shape::bounding_box(&shape.local_path())));
        let mut runs = Vec::new();
        let mut chars = 0usize;
        for span in spans {
            let text: String = span
                .text
                .chars()
                .filter_map(|c| match c {
                    '\r' | '\n' | '\u{2029}' | '\u{2028}' => Some('\n'),
                    '\t' => Some('\t'),
                    c if c.is_control() => None,
                    c => Some(c),
                })
                .take(MAX_CHARS.saturating_sub(chars))
                .collect();
            chars += text.chars().count();
            if text.is_empty() {
                continue;
            }
            let style = self.char_style(span, shape, para, bounds);
            runs.push(TextRun::new(text, style));
        }
        // A text that ends a paragraph has no empty paragraph after it here.
        if let Some(last) = runs.last_mut()
            && last.text.ends_with('\n')
        {
            last.text.pop();
            if last.text.is_empty() {
                runs.pop();
            }
        }
        if runs.is_empty() {
            return None;
        }
        let mut obj = TextObject::point(kurbo::Point::ZERO, "", CharStyle::default());
        obj.runs = runs;
        obj.vertical = para.vertical;
        obj.para.justify = justify(*align);
        let indent = |v: f64| if v.is_finite() { (v * K).clamp(-5000.0, 5000.0) } else { 0.0 };
        obj.para.left_indent = indent(para.left_indent);
        obj.para.right_indent = indent(para.right_indent);
        obj.para.first_line_indent = indent(para.first_line_indent);
        // Text space (points, y down, at the first baseline's start or the frame's top left) to
        // the object's local space (mm, y up), then the page and the document.
        let flip = Affine::scale_non_uniform(1.0 / K, -1.0 / K);
        match frame {
            None => {
                obj.xf = to_doc * full * Affine::translate(origin.to_vec2()) * flip;
                obj.scale_text_attributes();
            }
            Some(f) => {
                let (w, h) = (f.width * K, f.height * K);
                if !(w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0) {
                    self.warn("text frames with no size (left out)");
                    return None;
                }
                obj.xf = to_doc * full * Affine::translate((origin.x, origin.y + f.height)) * flip;
                obj.kind = TextKind::Area { frame: shapes::rectangle(Rect::new(0.0, 0.0, w, h)) };
                if para.columns > 1 {
                    self.warn("paragraph text in columns (imported in one column)");
                }
            }
        }
        if !in_range(obj.xf.transform_rect_bbox(Rect::new(0.0, 0.0, 1.0, 1.0))) {
            self.warn("objects beyond the document range VectorCraft supports (left out)");
            return None;
        }
        obj.normalize_paras();
        Some(Node::new(self.doc.alloc_id(), NodeKind::Text(Box::new(obj))))
    }

    /// A span's character style: its own fill and outline, or the text object's.
    fn char_style(&mut self, span: &td::TextSpan, shape: &td::Shape, para: &td::ParagraphStyle, bounds: Rect) -> CharStyle {
        let finite = |v: f64, d: f64| if v.is_finite() { v } else { d };
        let mut style = CharStyle::default();
        if !span.font_family.trim().is_empty() {
            style.font_family = span.font_family.clone();
        }
        style.font_style = font_style(span.bold, span.italic).into();
        let size = finite(span.size_pt, 12.0).clamp(0.1, 1296.0);
        style.size = size;
        // Tracking: the file's percent of an em; here thousandths of an em.
        style.tracking = finite(span.tracking_pct * 10.0, 0.0).clamp(-1000.0, 10_000.0);
        style.baseline_shift = finite(span.baseline_shift_pt, 0.0).clamp(-5000.0, 5000.0);
        style.rotation = finite(span.angle_deg, 0.0);
        style.underline = span.underline;
        style.strikethrough = span.strikethrough;
        // Line spacing: the paragraph's, in its unit; 100 % of the character height as the
        // font's usual 1.2 em.
        let leading = match para.leading_unit {
            td::LeadingUnit::PercentCharHeight => size * 1.2 * para.leading_pct / 100.0,
            td::LeadingUnit::Points => para.leading_pct,
            td::LeadingUnit::PercentPointSize => size * para.leading_pct / 100.0,
        };
        if leading.is_finite() && leading > 0.0 && (leading - size * 1.2).abs() > 1e-6 {
            style.leading = Some(leading.min(5000.0));
        }
        let fill = span.fill.as_ref().unwrap_or(&shape.fill);
        style.fill = self.paint(fill, bounds).unwrap_or(Paint::None);
        match span.outline.as_ref().or(shape.stroke.as_ref()) {
            Some(s) if s.width.is_finite() && s.width > 0.0 => {
                style.stroke = Paint::solid(self.color(s.color));
                style.stroke_width = (s.width * K).min(1000.0);
            }
            _ => style.stroke = Paint::None,
        }
        style
    }
}
