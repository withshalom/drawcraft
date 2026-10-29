use serde_json::json;

use super::*;
use crate::state::DockTab;

fn app() -> VectorcraftApp {
    VectorcraftApp::new(vectorcraft_engine::Session::new(), crate::Services::default())
}

#[test]
fn illustrate_is_the_default_and_draw_brings_its_workspace() {
    let mut ui = UiState::default();
    assert_eq!(ui.mode, Mode::Illustrate);
    workspaces::switch(&mut ui, "Typography").unwrap();
    set(&mut ui, Mode::Draw).unwrap();
    assert_eq!(ui.mode, Mode::Draw);
    assert_eq!(ui.workspace, DRAW_WORKSPACE);
    assert!(ui.control_bar && ui.view.rulers, "the Control bar and the rulers show");
    assert_eq!(ui.dock_tab, DockTab::Layers);
    // Back to Illustrate: the workspace used before comes back.
    set(&mut ui, Mode::Illustrate).unwrap();
    assert_eq!(ui.mode, Mode::Illustrate);
    assert_eq!(ui.workspace, "Typography");
    assert!(ui.mode_restore.is_none());
}

#[test]
fn the_draw_workspace_keeps_the_changes_made_in_it() {
    let mut ui = UiState::default();
    set(&mut ui, Mode::Draw).unwrap();
    ui.view.rulers = false;
    set(&mut ui, Mode::Illustrate).unwrap();
    assert_eq!(ui.workspace, workspaces::ESSENTIALS);
    set(&mut ui, Mode::Draw).unwrap();
    assert!(!ui.view.rulers, "Draw came back as it was left");
}

#[test]
fn draw_picked_as_a_workspace_is_not_the_one_illustrate_brings_back() {
    let mut ui = UiState::default();
    workspaces::switch(&mut ui, DRAW_WORKSPACE).unwrap();
    set(&mut ui, Mode::Draw).unwrap();
    set(&mut ui, Mode::Illustrate).unwrap();
    assert_eq!(ui.workspace, workspaces::ESSENTIALS);
    // A remembered workspace deleted meanwhile: Essentials.
    workspaces::save_as(&mut ui, "Mine").unwrap();
    set(&mut ui, Mode::Draw).unwrap();
    workspaces::delete(&mut ui, "Mine").unwrap();
    set(&mut ui, Mode::Illustrate).unwrap();
    assert_eq!(ui.workspace, workspaces::ESSENTIALS);
}

#[test]
fn the_mode_is_kept_with_the_preferences() {
    let mut ui = UiState::default();
    workspaces::switch(&mut ui, "Painting").unwrap();
    set(&mut ui, Mode::Draw).unwrap();
    let v = serde_json::to_value(&ui).unwrap();
    assert_eq!(v["mode"], "draw");
    let back: UiState = serde_json::from_value(v).unwrap();
    assert_eq!(back.mode, Mode::Draw);
    assert_eq!(back.mode_restore.as_deref(), Some("Painting"));
    // Preferences written before modes existed open in Illustrate mode.
    let mut old = serde_json::to_value(UiState::default()).unwrap();
    old.as_object_mut().unwrap().remove("mode");
    let old: UiState = serde_json::from_value(old).unwrap();
    assert_eq!(old.mode, Mode::Illustrate);
}

#[test]
fn window_mode_switches_toggles_and_checks_its_menu_item() {
    let mut app = app();
    let r = app.run("window.mode", json!({"mode": "draw"})).unwrap();
    assert_eq!(r, json!({"mode": "draw", "workspace": "Draw"}));
    assert_eq!(crate::menus::checked(&app, "window.mode", &json!({"mode": "draw"})), Some(true));
    assert_eq!(crate::menus::checked(&app, "window.mode", &json!({"mode": "illustrate"})), Some(false));
    // Omitted: the other mode.
    assert_eq!(app.run("window.mode", json!({})).unwrap()["mode"], "illustrate");
    assert_eq!(app.run("window.mode", json!({"mode": "DRAW"})).unwrap()["mode"], "draw");
    assert!(app.run("window.mode", json!({"mode": "paint"})).is_err());
    assert!(app.run("window.mode", json!({"mode": 1})).is_err());
    assert_eq!(app.ui.mode, Mode::Draw, "a bad param changes nothing");
}

#[test]
fn window_mode_is_listed_in_the_window_menu() {
    let items = menu_items();
    assert_eq!(items.len(), 2);
    let ids: Vec<_> = items
        .iter()
        .filter_map(|i| match i {
            crate::menus::Item::Cmd(label, id, p) => Some((*label, *id, p["mode"].as_str().unwrap_or_default().to_string())),
            _ => None,
        })
        .collect();
    assert_eq!(ids, [("Illustrate", "window.mode", "illustrate".into()), ("Draw", "window.mode", "draw".into())]);
    assert!(crate::menus::UI_COMMANDS.iter().any(|c| c.0 == "window.mode"));
}

/// The app bar draws the mode switch with both modes when there is room.
#[test]
fn the_app_bar_shows_the_mode_switch() {
    let mut app = app();
    let ctx = egui::Context::default();
    crate::theme::install_fonts(&ctx);
    let raw = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1400.0, 600.0))), ..Default::default() };
    let mut out = ctx.run_ui(raw, |ui| crate::chrome::app_bar(&mut app, ui));
    out.textures_delta.clear();
    let texts: Vec<String> = out
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Text(t) => Some(t.galley.text().to_string()),
            _ => None,
        })
        .collect();
    assert!(texts.iter().any(|t| t == "Illustrate") && texts.iter().any(|t| t == "Draw"), "{texts:?}");
}
