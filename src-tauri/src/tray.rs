//! System tray: the always-available way to find, restore or dismiss the knight.

use tauri::{
    AppHandle, Manager, Wry,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::{AppState, settings::Settings, stage};

#[derive(Clone)]
pub struct TrayItems {
    toggle: MenuItem<Wry>,
    passive: CheckMenuItem<Wry>,
    top: CheckMenuItem<Wry>,
}

fn toggle_label(visible: bool) -> &'static str {
    if visible { "Hide Sentinel" } else { "Show Sentinel" }
}

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let s = app.state::<AppState>().settings.lock().unwrap().clone();
    let none = None::<&str>;
    let items = TrayItems {
        toggle: MenuItem::with_id(app, "toggle", toggle_label(s.visible), true, none)?,
        passive: CheckMenuItem::with_id(app, "passive", "Passive Mode", true, s.passive, none)?,
        top: CheckMenuItem::with_id(app, "top", "Always on Top", true, s.always_on_top, none)?,
    };
    let menu = Menu::with_items(
        app,
        &[
            &items.toggle,
            &MenuItem::with_id(app, "quote", "Show Quote", true, none)?,
            &PredefinedMenuItem::separator(app)?,
            &items.passive,
            &items.top,
            &MenuItem::with_id(app, "reset", "Reset Position", true, none)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "exit", "Exit", true, none)?,
        ],
    )?;

    let mut tray = TrayIconBuilder::with_id("sentinel")
        // "(dev)" tells a development build's tray icon apart from the installed one.
        .tooltip(format!(
            "Sentinel{} · {} summons the knight",
            if cfg!(debug_assertions) { " (dev)" } else { "" },
            s.shortcut
        ))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                stage::toggle_visible(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;

    *app.state::<AppState>().tray.lock().unwrap() = Some(items);
    Ok(())
}

/// Reflect settings in the tray's labels and check marks.
pub fn sync(app: &AppHandle, s: &Settings) {
    // Clone the handles out so no lock is held across menu calls.
    let Some(items) = app.state::<AppState>().tray.lock().unwrap().clone() else { return };
    let _ = items.toggle.set_text(toggle_label(s.visible));
    let _ = items.passive.set_checked(s.passive);
    let _ = items.top.set_checked(s.always_on_top);
}
