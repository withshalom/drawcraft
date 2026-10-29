//! Draw mode's swatch strip: a column of the document's colour swatches along the window's right
//! edge, outside the dock (`docs/draw-mode.md` §3). Click a swatch for the fill, right-click it for
//! the stroke; the first swatch is None. It lists the document's own solid swatches (the Swatches
//! panel's, groups included), so editing those edits the strip; it ships no palette of its own.

use egui::{Sense, Stroke, Ui, vec2};
use serde_json::{Value, json};
use vectorcraft_color::Paint;

use super::{Tokens, VectorcraftApp};

/// The strip's width, and a swatch's side.
pub const WIDTH: f32 = 24.0;
const CELL: f32 = 18.0;

/// A swatch as the strip shows it: its name (none for None), its paint and the params that apply it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Cell {
    pub name: String,
    pub paint: Paint,
    pub params: Value,
}

/// None, then the document's solid swatches in the Swatches panel's order (groups after).
pub(crate) fn cells(app: &VectorcraftApp) -> Vec<Cell> {
    let mut out = vec![Cell { name: String::new(), paint: Paint::None, params: json!({"none": true}) }];
    let Some(st) = app.session.active() else { return out };
    let d = &st.doc;
    let solids = d.swatches.iter().chain(d.swatch_groups.iter().flat_map(|g| g.swatches.iter()));
    for s in solids.filter(|s| matches!(s.paint, Paint::Solid { .. })) {
        if out.iter().any(|c| c.name == s.name) {
            continue;
        }
        out.push(Cell { name: s.name.clone(), paint: s.paint.clone(), params: json!({"swatch": s.name}) });
    }
    out
}

pub fn swatch_strip(app: &mut VectorcraftApp, ui: &mut Ui) {
    if app.session.active().is_none() || crate::menus::home_showing(app) {
        return;
    }
    let t = Tokens::get(ui.ctx());
    let cells = cells(app);
    let mut apply: Option<(&'static str, Value)> = None;
    egui::Panel::right("swatch_strip")
        .exact_size(WIDTH)
        .resizable(false)
        .frame(egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().id_salt("swatch-strip").scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(
                ui,
                |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(4.0);
                        ui.spacing_mut().item_spacing.y = 2.0;
                        for c in &cells {
                            let (r, resp) = ui.allocate_exact_size(vec2(CELL, CELL), Sense::click());
                            crate::widgets::swatch_tile(ui, r, &c.paint, false, resp.hovered());
                            let name = if c.name.is_empty() { tl!("None").to_string() } else { c.name.clone() };
                            let resp = resp
                                .on_hover_text(crate::i18n::fmt(tl!("{name}: click for the fill, right-click for the stroke"), &[("name", &name)]));
                            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &name));
                            if resp.clicked() {
                                apply = Some(("paint.setFill", c.params.clone()));
                            } else if resp.secondary_clicked() {
                                apply = Some(("paint.setStroke", c.params.clone()));
                            }
                        }
                        ui.add_space(4.0);
                    });
                },
            );
        });
    if let Some((cmd, mut params)) = apply {
        // The proxy in front stays in front: the strip sets paint, it doesn't switch Fill and Stroke.
        params["focus"] = json!(false);
        if let Err(e) = app.run(cmd, params) {
            app.status(e);
        }
    }
}

#[cfg(test)]
mod tests;
