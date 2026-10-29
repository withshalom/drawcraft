//! Window › Mode: **Illustrate** (the default, the Illustrator-style layout) and **Draw** (a
//! page-based layout: the Draw workspace, the page bar above the status bar and the swatch strip at
//! the window's right edge). The mode arranges the window only: the document, the commands, the
//! menus' contents and the shortcuts are the same in both (`docs/draw-mode.md` §3, decision D6).
//!
//! Switching to Draw remembers the workspace in use and switches to the built-in Draw workspace;
//! switching back to Illustrate brings the remembered workspace back (Essentials when it is gone).
//! Workspaces keep the changes made in them as always, so the Draw workspace comes back as it was
//! left.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::VectorcraftApp;
use crate::state::UiState;
use crate::workspaces;

/// The window's arrangement (`UiState::mode`, kept with the preferences).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    /// The Illustrator-style layout: artboards, the dock, the status bar's artboard navigation.
    #[default]
    Illustrate,
    /// The page-based layout: the Draw workspace, the page bar and the swatch strip.
    Draw,
}

impl Mode {
    /// Every mode, in the order the app bar and Window › Mode list them.
    pub const ALL: [Mode; 2] = [Mode::Illustrate, Mode::Draw];

    /// The mode's id in `window.mode` params.
    pub fn id(self) -> &'static str {
        match self {
            Mode::Illustrate => "illustrate",
            Mode::Draw => "draw",
        }
    }

    /// The mode's English label (translated where shown).
    pub fn label(self) -> &'static str {
        match self {
            Mode::Illustrate => "Illustrate",
            Mode::Draw => "Draw",
        }
    }

    /// What the mode is, for its tooltip (English; translated where shown).
    pub fn tip(self) -> &'static str {
        match self {
            Mode::Illustrate => "Illustrate mode: artboards on one canvas, panels in the dock",
            Mode::Draw => "Draw mode: pages with a page bar, a swatch strip at the right edge and the Control bar",
        }
    }

    /// The mode a `window.mode` param names, whatever its case.
    pub fn parse(s: &str) -> Option<Mode> {
        Mode::ALL.into_iter().find(|m| m.id().eq_ignore_ascii_case(s.trim()))
    }
}

/// The built-in workspace Draw mode switches to.
pub const DRAW_WORKSPACE: &str = "Draw";

/// Switch the window to `mode`. Draw: the workspace in use is remembered and the Draw workspace
/// comes (as it was left); Illustrate: the remembered workspace comes back, else Essentials. Already
/// in `mode`: nothing changes.
pub fn set(ui: &mut UiState, mode: Mode) -> Result<(), String> {
    if ui.mode == mode {
        return Ok(());
    }
    match mode {
        Mode::Draw => {
            let before = ui.workspace.clone();
            workspaces::switch(ui, DRAW_WORKSPACE)?;
            // Draw picked from Window › Workspace in Illustrate mode is not a workspace to go back to.
            ui.mode_restore = (!before.eq_ignore_ascii_case(DRAW_WORKSPACE)).then_some(before);
        }
        Mode::Illustrate => {
            let back = ui
                .mode_restore
                .take()
                .filter(|n| !n.eq_ignore_ascii_case(DRAW_WORKSPACE) && workspaces::find_saved(ui, n).is_some())
                .unwrap_or_else(|| workspaces::ESSENTIALS.to_string());
            workspaces::switch(ui, &back)?;
        }
    }
    ui.mode = mode;
    Ok(())
}

/// Window › Mode items.
pub fn menu_items() -> Vec<crate::menus::Item> {
    Mode::ALL.into_iter().map(|m| crate::menus::Item::Cmd(m.label(), "window.mode", json!({"mode": m.id()}))).collect()
}

/// `window.mode {mode?}`: switch to `mode` (`illustrate` or `draw`; omitted: the other one) →
/// {mode, workspace}.
pub fn run_command(app: &mut VectorcraftApp, id: &str, p: &Value) -> Option<Result<Value, String>> {
    if id != "window.mode" {
        return None;
    }
    let mode = match p.get("mode") {
        None | Some(Value::Null) => match app.ui.mode {
            Mode::Illustrate => Mode::Draw,
            Mode::Draw => Mode::Illustrate,
        },
        Some(Value::String(s)) => match Mode::parse(s) {
            Some(m) => m,
            None => return Some(Err(format!("unknown mode `{s}` (illustrate or draw)"))),
        },
        Some(_) => return Some(Err("mode must be illustrate or draw".into())),
    };
    Some(set(&mut app.ui, mode).map(|()| json!({"mode": app.ui.mode.id(), "workspace": app.ui.workspace})))
}

/// The app bar's mode switch: one segment per mode, the current one filled. Returns its width.
pub fn width(ui: &egui::Ui) -> f32 {
    Mode::ALL.iter().map(|m| segment_width(ui, *m)).sum()
}

fn segment_width(ui: &egui::Ui, m: Mode) -> f32 {
    ui.painter().layout_no_wrap(tl!(m.label()).to_string(), egui::FontId::proportional(12.0), egui::Color32::WHITE).size().x + 20.0
}

/// Draw the mode switch at the next place of `ui`, `height` tall.
pub fn switch_ui(app: &mut VectorcraftApp, ui: &mut egui::Ui, height: f32) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width(ui), height), egui::Sense::hover());
    ui.painter().rect_filled(rect, egui::CornerRadius::same(4), t.panel);
    let mut x = rect.left();
    let mut chosen = None;
    for m in Mode::ALL {
        let w = segment_width(ui, m);
        let seg = egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(w, height));
        x += w;
        let resp = ui.interact(seg, ui.scope_id().with(("mode-switch", m.id())), egui::Sense::click());
        let on = app.ui.mode == m;
        let fill = if on {
            t.accent
        } else if resp.hovered() {
            t.hover
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter().rect_filled(seg.shrink(1.0), egui::CornerRadius::same(3), fill);
        let text = if on { egui::Color32::WHITE } else { t.text };
        ui.painter().text(seg.center(), egui::Align2::CENTER_CENTER, tl!(m.label()), egui::FontId::proportional(12.0), text);
        resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::RadioButton, true, on, tl!(m.label())));
        if resp.on_hover_text(tl!(m.tip())).clicked() {
            chosen = Some(m);
        }
    }
    if let Some(m) = chosen
        && let Err(e) = app.run("window.mode", json!({"mode": m.id()}))
    {
        app.status(e);
    }
}

#[cfg(test)]
mod tests;
