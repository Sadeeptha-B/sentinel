//! The knight's right-click menu, plus the single handler for every menu action
//! (context menu and tray share ids).

use tauri::{
    AppHandle, Manager,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
};

use crate::{AppState, stage};

const SIZES: [(&str, &str, f64); 3] =
    [("size:300", "Small", 300.0), ("size:400", "Medium", 400.0), ("size:520", "Large", 520.0)];

pub fn popup(app: &AppHandle) -> tauri::Result<()> {
    let s = app.state::<AppState>().settings.lock().unwrap().clone();
    let none = None::<&str>;

    let sizes = SIZES
        .iter()
        .map(|(id, label, h)| {
            let current = (s.height - h).abs() / h < 0.04;
            CheckMenuItem::with_id(app, *id, *label, true, current, none)
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    let size_refs: Vec<&dyn tauri::menu::IsMenuItem<_>> =
        sizes.iter().map(|i| i as &dyn tauri::menu::IsMenuItem<_>).collect();

    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "quote", "Show Quote", true, none)?,
            &MenuItem::with_id(
                app,
                "watch",
                if s.stand_watch { "Stand Down" } else { "Stand Watch" },
                true,
                none,
            )?,
            &PredefinedMenuItem::separator(app)?,
            &Submenu::with_id_and_items(app, "size", "Size", true, &size_refs)?,
            &CheckMenuItem::with_id(app, "lock", "Lock Position", true, s.locked, none)?,
            &CheckMenuItem::with_id(app, "top", "Always on Top", true, s.always_on_top, none)?,
            &CheckMenuItem::with_id(app, "passive", "Passive (click-through)", true, s.passive, none)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "hide", "Hide", true, none)?,
            &MenuItem::with_id(app, "exit", "Exit", true, none)?,
        ],
    )?;
    if let Some(win) = app.get_webview_window(stage::KNIGHT) {
        win.popup_menu(&menu)?;
    }
    Ok(())
}

pub fn handle(app: &AppHandle, id: &str) {
    let s = app.state::<AppState>().settings.lock().unwrap().clone();
    match id {
        "quote" => stage::whisper(app, None),
        "watch" => stage::set_stand_watch(app, !s.stand_watch),
        "lock" => stage::set_locked(app, !s.locked),
        "top" => stage::set_always_on_top(app, !s.always_on_top),
        "passive" => stage::set_passive(app, !s.passive),
        "hide" => stage::set_visible(app, false),
        "toggle" => stage::toggle_visible(app),
        "reset" => {
            stage::reset_position(app);
            stage::set_visible(app, true);
        }
        "exit" => stage::exit(app),
        _ => {
            if let Some((_, _, h)) = SIZES.iter().find(|(sid, _, _)| *sid == id) {
                stage::resize_to(app, *h);
            }
        }
    }
}
