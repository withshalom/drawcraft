//! Project links: the GitHub button (header, About, Home) and the link list. Where upstream
//! VectorCraft shows its community's Discord, DRAWCraft links its own repository. Icons are
//! Lucide's (ISC); GitHub's logo is a trademark, so the button uses Lucide's branch icon on a
//! dark GitHub-like grey with the word "GitHub".

use egui::{Color32, CornerRadius, Sense, Ui, vec2};

use crate::theme::{self, Tokens};
use crate::{VectorcraftApp, icons};

/// The large button's fill (a colour, not a logo).
const GITHUB: Color32 = Color32::from_rgb(0x24, 0x29, 0x2f);

/// The GitHub button's label, height and width.
fn github_layout(ui: &Ui, large: bool) -> (std::sync::Arc<egui::Galley>, f32, f32) {
    let (h, font, label) =
        if large { (36.0, theme::semibold(14.0), tl!("DRAWCraft on GitHub")) } else { (24.0, egui::FontId::proportional(12.0), "GitHub") };
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, Color32::PLACEHOLDER);
    let w = galley.size().x + h * 0.55 + h * 0.9;
    (galley, h, w)
}

/// The GitHub button's width (the app bar drops it when space runs out).
pub fn github_width(ui: &Ui, large: bool) -> f32 {
    github_layout(ui, large).2
}

/// The GitHub button: compact and quiet for the app bar, large for About / Home. It opens
/// DRAWCraft's repository (`help.github`).
pub fn github_button(app: &mut VectorcraftApp, ui: &mut Ui, large: bool) -> egui::Response {
    let (galley, h, w) = github_layout(ui, large);
    let icon = h * 0.55;
    let (r, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click());
    let at = egui::pos2(0.0, r.center().y - galley.size().y / 2.0);
    let ir = egui::Rect::from_center_size(r.left_center() + vec2(h * 0.35 + icon / 2.0, 0.0), vec2(icon, icon));
    if large {
        let fill = if resp.hovered() { GITHUB.gamma_multiply(0.85) } else { GITHUB };
        ui.painter().rect_filled(r, CornerRadius::same((h / 2.0) as u8), fill);
        icons::paint(ui, "git-branch", ir, Color32::WHITE);
        ui.painter().galley(egui::pos2(ir.right() + h * 0.2, at.y), galley, Color32::WHITE);
    } else {
        // In the app bar it stays quiet: a soft button with the icon in the text colour.
        let t = Tokens::get(ui.ctx());
        ui.painter().rect_filled(r, CornerRadius::same(6), if resp.hovered() { t.hover } else { t.button });
        icons::paint(ui, "git-branch", ir, if resp.hovered() { t.text_strong } else { t.text });
        ui.painter().galley(egui::pos2(ir.right() + h * 0.2, at.y), galley, if resp.hovered() { t.text_strong } else { t.text });
    }
    let resp = resp.on_hover_text(tl!("DRAWCraft's source code, issues and releases on GitHub"));
    if resp.clicked() {
        app.open_link("help.github");
    }
    resp
}

/// One link row: icon, label, opens the command's URL.
fn link(app: &mut VectorcraftApp, ui: &mut Ui, icon: &str, label: &str, cmd: &str, url: &str) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        icons::icon(ui, icon, 16.0, t.icon);
        let r = ui.add(egui::Button::new(egui::RichText::new(label).color(t.accent).size(13.0)).frame(false)).on_hover_text(url);
        if r.clicked() {
            app.open_link(cmd);
        }
    });
}

/// The GitHub button and the website and app page links.
pub fn links(app: &mut VectorcraftApp, ui: &mut Ui) {
    github_button(app, ui, true);
    ui.add_space(8.0);
    let l = app.session.execute("help.links", &serde_json::json!({})).unwrap_or_default();
    let s = |k: &str| l[k].as_str().unwrap_or("").to_string();
    link(app, ui, "globe", tl!("ArtCraft website"), "help.website", &s("website"));
    link(app, ui, "external-link", tl!("VectorCraft on getartcraft.com"), "help.appPage", &s("appPage"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_clicks_open_the_right_urls() {
        use std::sync::{Arc, Mutex};
        let opened = Arc::new(Mutex::new(vec![]));
        let o = opened.clone();
        let services = crate::Services { open_url: Some(Box::new(move |u: &str| o.lock().unwrap().push(u.to_string()))), ..Default::default() };
        let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), services);
        for id in ["help.website", "help.appPage", "help.github"] {
            crate::menus::invoke(&mut app, id, serde_json::json!({}));
        }
        assert_eq!(
            *opened.lock().unwrap(),
            ["https://getartcraft.com", "https://getartcraft.com/apps/vectorcraft", "https://github.com/withshalom/drawcraft"]
        );
        // The control channel / MCP path returns the URL without opening a browser.
        let v = app.run("help.github", serde_json::json!({})).unwrap();
        assert_eq!(v["url"], "https://github.com/withshalom/drawcraft");
        assert_eq!(opened.lock().unwrap().len(), 3);
        let ctx = egui::Context::default();
        theme::install_fonts(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| links(&mut app, ui));
        out.textures_delta.clear();
    }
}
