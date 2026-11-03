mod knight;
mod menu;
mod settings;
mod stage;
mod ticker;
mod tray;

use std::{
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicIsize, AtomicU64},
    },
    time::{Duration, Instant},
};

use serde::Serialize;
use tauri::{AppHandle, Manager, RunEvent, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use settings::Settings;

const SAVE_DELAY: Duration = Duration::from_millis(500);

pub struct AppState {
    pub settings: Mutex<Settings>,
    path: PathBuf,
    dirty: Mutex<Option<Instant>>,
    /// The knight window's native handle (Windows), for reading its geometry from any
    /// thread without a round trip through the event loop. Zero when unknown.
    pub hwnd: AtomicIsize,
    /// Last click-through state applied to the knight window.
    pub ignoring: Mutex<Option<bool>>,
    /// The page has painted, so the window may be shown.
    pub revealed: AtomicBool,
    pub whisper_seq: AtomicU64,
    pub whisper_shown: AtomicBool,
    pub tray: Mutex<Option<tray::TrayItems>>,
}

impl AppState {
    fn new(settings: Settings, path: PathBuf) -> Self {
        Self {
            settings: Mutex::new(settings),
            path,
            dirty: Mutex::new(None),
            hwnd: AtomicIsize::new(0),
            ignoring: Mutex::new(None),
            revealed: AtomicBool::new(false),
            whisper_seq: AtomicU64::new(0),
            whisper_shown: AtomicBool::new(false),
            tray: Mutex::new(None),
        }
    }

    pub fn mark_dirty(&self) {
        *self.dirty.lock().unwrap() = Some(Instant::now());
    }

    pub fn flush_if_due(&self) {
        let due = matches!(*self.dirty.lock().unwrap(), Some(t) if t.elapsed() >= SAVE_DELAY);
        if due {
            self.flush();
        }
    }

    pub fn flush(&self) {
        if self.dirty.lock().unwrap().take().is_none() {
            return;
        }
        let snapshot = self.settings.lock().unwrap().clone();
        if let Err(e) = settings::save(&self.path, &snapshot) {
            eprintln!("sentinel: could not save settings to {}: {e}", self.path.display());
        }
    }
}

/// The slice of settings the knight's page needs.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiState {
    locked: bool,
    passive: bool,
    stand_watch: bool,
}

impl From<&Settings> for UiState {
    fn from(s: &Settings) -> Self {
        Self { locked: s.locked, passive: s.passive, stand_watch: s.stand_watch }
    }
}

#[tauri::command]
fn knight_ready(app: AppHandle) {
    stage::reveal(&app);
}

#[tauri::command]
fn ui_state(state: State<'_, AppState>) -> UiState {
    UiState::from(&*state.settings.lock().unwrap())
}

#[tauri::command]
fn request_quote(app: AppHandle) {
    stage::whisper(&app, None);
}

#[tauri::command]
fn resize_by(app: AppHandle, factor: f64) {
    if factor.is_finite() && factor > 0.0 {
        stage::resize_by(&app, factor);
    }
}

#[tauri::command]
fn open_context_menu(app: AppHandle) -> Result<(), String> {
    menu::popup(&app).map_err(|e| e.to_string())
}

#[tauri::command]
fn whisper_done(app: AppHandle, seq: u64) {
    stage::whisper_done(&app, seq);
}

fn register_shortcut(app: &AppHandle) {
    let accelerator = app.state::<AppState>().settings.lock().unwrap().shortcut.clone();
    let plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _, event| {
            if event.state() == ShortcutState::Pressed {
                stage::summon(app);
            }
        })
        .build();
    // A missing shortcut is not fatal: the tray remains the recovery path.
    if let Err(e) = app.plugin(plugin) {
        eprintln!("sentinel: global shortcuts unavailable: {e}");
    } else if let Err(e) = app.global_shortcut().register(accelerator.as_str()) {
        eprintln!("sentinel: could not register shortcut {accelerator}: {e}");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must come first: a second launch summons the running knight instead.
        .plugin(tauri_plugin_single_instance::init(|app, _, _| stage::summon(app)))
        .on_menu_event(|app, event| menu::handle(app, event.id().as_ref()))
        .on_window_event(stage::on_window_event)
        .invoke_handler(tauri::generate_handler![
            knight_ready,
            ui_state,
            request_quote,
            resize_by,
            open_context_menu,
            whisper_done
        ])
        .setup(|app| {
            let path = app.path().app_config_dir()?.join("settings.json");
            app.manage(AppState::new(settings::load(&path), path));
            let handle = app.handle();
            stage::build(handle)?;
            tray::build(handle)?;
            register_shortcut(handle);
            ticker::spawn(handle.clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Sentinel")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                app.state::<AppState>().flush();
            }
        });
}
