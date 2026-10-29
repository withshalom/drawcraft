//! Draw mode's page bar, above the status bar: First, Previous, "Page n of m", Next, Last and Add
//! Page, then a tab per page in page order (`docs/draw-mode.md` §3). A click on a tab goes to that
//! page and fits it in the window; double-click renames it; dragging a tab onto another moves the
//! page there; a tab's menu inserts pages before or after it, renames, moves and deletes it.
//!
//! Everything runs a command (`view.goToArtboard`, `pages.insert`, `pages.move`,
//! `artboard.setProps`, `artboard.delete`). A document without pages shows its artboards as its
//! pages; adding or moving a page makes it a pages document in the same undo step.

use egui::{Sense, Stroke, Ui, vec2};
use serde_json::{Value, json};

use super::{Tokens, VectorcraftApp, widgets};

/// The bar's height.
pub const HEIGHT: f32 = 26.0;
/// A tab's least and greatest width.
const TAB_MIN: f32 = 52.0;
const TAB_MAX: f32 = 168.0;

/// A page as the bar shows it: its artboard's index and name.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Page {
    pub index: usize,
    pub name: String,
}

/// The active document's pages in page order (an artboards document: its artboards).
pub(crate) fn pages(app: &VectorcraftApp) -> Vec<Page> {
    let Some(st) = app.session.active() else { return Vec::new() };
    let d = &st.doc;
    d.page_order().into_iter().filter_map(|i| d.artboards.get(i).map(|a| Page { index: i, name: a.name.clone() })).collect()
}

/// The place in page order (0-based) of the active page.
pub(crate) fn current(app: &VectorcraftApp, pages: &[Page]) -> usize {
    let n = app.session.active().map(|d| d.doc.artboards.len()).unwrap_or(0);
    let active = crate::panels::artboards::selected(app, n);
    pages.iter().position(|p| p.index == active).unwrap_or(0)
}

/// What a click in the bar asks for, run after drawing.
#[derive(Clone, Debug, PartialEq)]
enum Action {
    /// Go to the page whose artboard has this index.
    Go(usize),
    /// Insert a page after page number `after` (0: before the first) and go to it.
    Insert(usize),
    /// Rename the artboard of this index.
    Rename(usize, String),
    /// Move page number `page` to number `to`.
    Move { page: usize, to: usize },
    /// Delete the artboard of this index.
    Delete(usize),
}

/// A page tab being renamed: its artboard's index and the text typed so far.
#[derive(Clone, Debug, Default)]
struct Renaming(Option<(usize, String)>);

/// The page being dragged, by its place in page order.
#[derive(Clone, Copy, Debug)]
struct TabDrag(usize);

pub fn page_bar(app: &mut VectorcraftApp, ui: &mut Ui) {
    if app.session.active().is_none() || crate::menus::home_showing(app) {
        return;
    }
    let t = Tokens::get(ui.ctx());
    let pages = pages(app);
    let cur = current(app, &pages);
    let m = pages.len();
    let mut action = None;
    egui::Panel::bottom("page_bar")
        .exact_size(HEIGHT)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(8, 0)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                let at = |k: usize| pages.get(k).map(|p| Action::Go(p.index));
                for (icon, k, tip, on) in
                    [("chevrons-left", Some(0), tl!("First Page"), cur > 0), ("chevron-left", cur.checked_sub(1), tl!("Previous Page"), cur > 0)]
                {
                    if widgets::icon_button_enabled(ui, icon, tip, false, on, 18.0).clicked() {
                        action = k.and_then(at);
                    }
                }
                let shown = if m == 0 {
                    tl!("No pages").to_string()
                } else {
                    crate::i18n::fmt(tl!("Page {n} of {m}"), &[("n", &(cur + 1).to_string()), ("m", &m.to_string())])
                };
                ui.label(egui::RichText::new(shown).size(11.5).color(t.text_dim));
                for (icon, k, tip, on) in [
                    ("chevron-right", Some(cur + 1), tl!("Next Page"), cur + 1 < m),
                    ("chevrons-right", m.checked_sub(1), tl!("Last Page"), cur + 1 < m),
                ] {
                    if widgets::icon_button_enabled(ui, icon, tip, false, on, 18.0).clicked() {
                        action = k.and_then(at);
                    }
                }
                if widgets::icon_button(ui, "plus", tl!("Add Page after this one"), false, 18.0).clicked() {
                    action = Some(Action::Insert(if m == 0 { 0 } else { cur + 1 }));
                }
                ui.separator();
                egui::ScrollArea::horizontal().id_salt("page-bar-tabs").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        for (k, p) in pages.iter().enumerate() {
                            if let Some(a) = tab(app, ui, &t, k, p, k == cur, m) {
                                action = Some(a);
                            }
                        }
                    });
                });
            });
        });
    if let Some(a) = action {
        run(app, a);
    }
}

/// One page tab; returns what it asks for.
fn tab(app: &VectorcraftApp, ui: &mut Ui, t: &Tokens, k: usize, p: &Page, active: bool, m: usize) -> Option<Action> {
    let rename_id = egui::Id::unique("page-bar-rename");
    let renaming: Renaming = ui.data(|d| d.get_temp(rename_id)).unwrap_or_default();
    if let Some((index, mut text)) = renaming.0.clone()
        && index == p.index
    {
        let r = ui.add(egui::TextEdit::singleline(&mut text).desired_width(TAB_MAX - 16.0).font(egui::FontId::proportional(11.5)));
        if !r.has_focus() && !r.lost_focus() {
            r.request_focus();
        }
        let (enter, escape) = ui.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Escape)));
        if r.lost_focus() || enter || escape {
            ui.data_mut(|d| d.insert_temp(rename_id, Renaming(None)));
            let name = text.trim().to_string();
            return (!escape && !name.is_empty() && name != p.name).then_some(Action::Rename(p.index, name));
        }
        ui.data_mut(|d| d.insert_temp(rename_id, Renaming(Some((index, text)))));
        return None;
    }
    let font = egui::FontId::proportional(11.5);
    let galley = ui.painter().layout_no_wrap(p.name.clone(), font.clone(), t.text);
    let w = (galley.size().x + 18.0).clamp(TAB_MIN, TAB_MAX);
    let (rect, resp) = ui.allocate_exact_size(vec2(w, HEIGHT - 6.0), Sense::click_and_drag());
    let fill = if active {
        t.row_selected
    } else if resp.hovered() {
        t.hover
    } else {
        t.input
    };
    ui.painter().rect_filled(rect, egui::CornerRadius::same(3), fill);
    ui.painter().rect_stroke(rect, egui::CornerRadius::same(3), Stroke::new(1.0, if active { t.accent } else { t.border }), egui::StrokeKind::Inside);
    let text = egui::WidgetText::from(egui::RichText::new(&p.name).font(font)).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        rect.width() - 12.0,
        egui::TextStyle::Body,
    );
    let at = egui::pos2(rect.center().x - text.size().x / 2.0, rect.center().y - text.size().y / 2.0);
    ui.painter().with_clip_rect(rect).galley(at, text, if active { t.text_strong } else { t.text });
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, active, &p.name));
    // Dragging a tab onto another moves the page there.
    if resp.drag_started() {
        resp.dnd_set_drag_payload(TabDrag(k));
    }
    if resp.dnd_hover_payload::<TabDrag>().is_some_and(|from| from.0 != k) {
        let x = if resp.dnd_hover_payload::<TabDrag>().is_some_and(|from| from.0 > k) { rect.left() - 1.0 } else { rect.right() + 1.0 };
        ui.painter().vline(x, rect.y_range(), Stroke::new(2.0, t.accent));
    }
    let mut action = None;
    if let Some(from) = resp.dnd_release_payload::<TabDrag>()
        && from.0 != k
    {
        action = Some(Action::Move { page: from.0 + 1, to: k + 1 });
    }
    let page_name = p.name.clone();
    let resp = resp.on_hover_text(&page_name);
    if resp.double_clicked() {
        ui.data_mut(|d| d.insert_temp(rename_id, Renaming(Some((p.index, page_name.clone())))));
    } else if resp.clicked() {
        action = Some(Action::Go(p.index));
    }
    resp.context_menu(|ui| {
        if ui.button(tl!("Insert Page Before")).clicked() {
            action = Some(Action::Insert(k));
            ui.close();
        }
        if ui.button(tl!("Insert Page After")).clicked() {
            action = Some(Action::Insert(k + 1));
            ui.close();
        }
        if ui.button(tl!("Rename Page")).clicked() {
            ui.data_mut(|d| d.insert_temp(rename_id, Renaming(Some((p.index, page_name.clone())))));
            ui.close();
        }
        ui.separator();
        if ui.add_enabled(k > 0, egui::Button::new(tl!("Move Page Left"))).clicked() {
            action = Some(Action::Move { page: k + 1, to: k });
            ui.close();
        }
        if ui.add_enabled(k + 1 < m, egui::Button::new(tl!("Move Page Right"))).clicked() {
            action = Some(Action::Move { page: k + 1, to: k + 2 });
            ui.close();
        }
        ui.separator();
        let locked = app.session.active().and_then(|d| d.doc.artboards.get(p.index)).is_some_and(|a| a.locked);
        if ui.add_enabled(m > 1 && !locked, egui::Button::new(tl!("Delete Page"))).clicked() {
            action = Some(Action::Delete(p.index));
            ui.close();
        }
    });
    action
}

/// Run what the bar asked for; a failure goes to the status bar.
fn run(app: &mut VectorcraftApp, a: Action) {
    let r: Result<Value, String> = match a {
        Action::Go(i) => app.run("view.goToArtboard", json!({ "index": i })),
        Action::Insert(after) => app.run("pages.insert", json!({ "after": after })).and_then(|r| {
            // Go to the new page.
            match r["indices"].get(0).and_then(Value::as_u64) {
                Some(i) => app.run("view.goToArtboard", json!({ "index": i })),
                None => Ok(r),
            }
        }),
        Action::Rename(i, name) => app.run("artboard.setProps", json!({ "index": i, "name": name })),
        Action::Move { page, to } => app.run("pages.move", json!({ "page": page, "to": to })),
        Action::Delete(i) => app.run("artboard.delete", json!({ "index": i })),
    };
    if let Err(e) = r {
        app.status(e);
    }
}

#[cfg(test)]
mod tests;
