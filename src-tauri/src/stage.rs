//! Desktop window management: the knight's window and the quote ("whisper") window,
//! placement and recovery, feet-anchored resizing, visibility and mode toggles.
//!
//! Locking discipline: never hold an `AppState` mutex across a window call. Window
//! calls made off the main thread block on it, so holding a lock there can deadlock.

use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, Window, WindowEvent,
};

use crate::{
    AppState, UiState,
    knight::{self, Placement, Rect},
    settings::Settings,
};

pub const KNIGHT: &str = "knight";
pub const WHISPER: &str = "whisper";
/// Logical size of the quote window.
const WHISPER_SIZE: (f64, f64) = (420.0, 150.0);
/// Logical gap between the default spot and the screen's right edge.
const MARGIN: f64 = 24.0;
/// Logical size of the part of him that must stay on some screen.
const REACHABLE_MIN: f64 = 64.0;

#[derive(Clone, Copy)]
struct Screen {
    bounds: Rect,
    work: Rect,
    scale: f64,
}

impl From<&tauri::Monitor> for Screen {
    fn from(m: &tauri::Monitor) -> Self {
        let (p, s, wa) = (m.position(), m.size(), m.work_area());
        Screen {
            bounds: Rect::new(p.x as f64, p.y as f64, s.width as f64, s.height as f64),
            work: Rect::new(
                wa.position.x as f64,
                wa.position.y as f64,
                wa.size.width as f64,
                wa.size.height as f64,
            ),
            scale: m.scale_factor(),
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct WhisperPayload {
    seq: u64,
    /// `None` asks the whisper window to choose a quote itself.
    text: Option<String>,
    placement: Placement,
    font_px: f64,
}

fn state(app: &AppHandle) -> tauri::State<'_, AppState> {
    app.state::<AppState>()
}

fn settings(app: &AppHandle) -> Settings {
    state(app).settings.lock().unwrap().clone()
}

fn knight_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(KNIGHT)
}

fn screens(win: &WebviewWindow) -> Vec<Screen> {
    win.available_monitors().unwrap_or_default().iter().map(Screen::from).collect()
}

fn primary(win: &WebviewWindow) -> Option<Screen> {
    match win.primary_monitor() {
        Ok(Some(m)) => Some(Screen::from(&m)),
        _ => screens(win).into_iter().next(),
    }
}

/// The screen showing most of `r`, falling back to the primary screen.
fn screen_for(win: &WebviewWindow, r: Rect) -> Option<Screen> {
    let best = screens(win)
        .into_iter()
        .map(|s| {
            let (w, h) = r.overlap(&s.bounds);
            (w * h, s)
        })
        .filter(|(area, _)| *area > 0.0)
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, s)| s);
    best.or_else(|| primary(win))
}

/// Physical window rect at `pos` for a figure about `height` logical px tall on
/// `screen`, snapped to a whole-number pixel scale that fits; plus the height that
/// scale actually gives.
fn fitted(screen: &Screen, x: f64, y: f64, height: f64) -> (Rect, f64) {
    let k = knight::scale_for(height, screen.scale, screen.work.h);
    let (w, h) = knight::window_px(k);
    (Rect::new(x, y, w, h), knight::height_for(k, screen.scale))
}

fn default_placement(win: &WebviewWindow, height: f64) -> Option<(Rect, f64)> {
    let screen = primary(win)?;
    let (r, height) = fitted(&screen, 0.0, 0.0, height);
    Some((knight::default_rect(screen.work, r.w, r.h, MARGIN * screen.scale), height))
}

/// The knight window's actual outer rect in physical px.
pub fn current_rect(app: &AppHandle) -> Option<Rect> {
    #[cfg(windows)]
    {
        use windows::Win32::{
            Foundation::{HWND, RECT},
            UI::WindowsAndMessaging::GetWindowRect,
        };
        let raw = state(app).hwnd.load(Ordering::Relaxed);
        let mut r = RECT::default();
        // SAFETY: GetWindowRect only reads; a stale handle simply fails.
        if raw != 0 && unsafe { GetWindowRect(HWND(raw as *mut _), &mut r) }.is_ok() {
            let (w, h) = ((r.right - r.left) as f64, (r.bottom - r.top) as f64);
            return Some(Rect::new(r.left as f64, r.top as f64, w, h));
        }
    }
    outer_rect(&knight_window(app)?)
}

/// The knight window's DPI scale factor. On Windows it's read from the OS, because the
/// windowing layer's cached value can lag when the window moves between displays.
pub fn dpi(app: &AppHandle) -> Option<f64> {
    #[cfg(windows)]
    {
        use windows::Win32::{Foundation::HWND, UI::HiDpi::GetDpiForWindow};
        let raw = state(app).hwnd.load(Ordering::Relaxed);
        // SAFETY: GetDpiForWindow only reads; a stale handle returns 0.
        let dpi = if raw != 0 { unsafe { GetDpiForWindow(HWND(raw as *mut _)) } } else { 0 };
        if dpi > 0 {
            return Some(dpi as f64 / 96.0);
        }
    }
    knight_window(app)?.scale_factor().ok()
}

/// Move and resize in one step so his feet never visibly jump.
fn apply_rect(win: &WebviewWindow, r: Rect) {
    #[cfg(windows)]
    if let Ok(hwnd) = win.hwnd() {
        use windows::Win32::UI::WindowsAndMessaging::{
            SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos,
        };
        // SAFETY: hwnd belongs to a live window owned by this process.
        let moved = unsafe {
            SetWindowPos(
                hwnd,
                None,
                r.x as i32,
                r.y as i32,
                r.w as i32,
                r.h as i32,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
        if moved.is_ok() {
            return;
        }
    }
    let _ = win.set_position(PhysicalPosition::new(r.x as i32, r.y as i32));
    let _ = win.set_size(PhysicalSize::new(r.w as u32, r.h as u32));
}

/// Mutate settings, schedule a save and tell the knight window and tray.
fn update(app: &AppHandle, change: impl FnOnce(&mut Settings)) {
    let st = state(app);
    let snapshot = {
        let mut s = st.settings.lock().unwrap();
        change(&mut s);
        s.clone()
    };
    st.mark_dirty();
    let _ = app.emit_to(KNIGHT, "sentinel://state", UiState::from(&snapshot));
    crate::tray::sync(app, &snapshot);
}

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let s = settings(app);
    // Provisional size; `place` sets the exact pixel-scaled rect before he is shown.
    let (w, h) = knight::window_px(knight::scale_for(s.height, 1.0, 1080.0));
    let knight = WebviewWindowBuilder::new(app, KNIGHT, WebviewUrl::App("index.html".into()))
        .title("Sentinel")
        .inner_size(w, h)
        .transparent(true)
        .decorations(false)
        .shadow(false)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .always_on_top(s.always_on_top)
        .skip_taskbar(true)
        .focused(false)
        .visible(false)
        .build()?;

    let (ww, wh) = WHISPER_SIZE;
    let whisper = WebviewWindowBuilder::new(app, WHISPER, WebviewUrl::App("index.html".into()))
        .title("Sentinel")
        .inner_size(ww, wh)
        .transparent(true)
        .decorations(false)
        .shadow(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focusable(false)
        .focused(false)
        .visible(false)
        .build()?;
    whisper.set_ignore_cursor_events(true)?;
    #[cfg(windows)]
    state(app).hwnd.store(knight.hwnd()?.0 as isize, Ordering::Relaxed);

    place(app, &knight, s.x.zip(s.y), s.height);
    if s.passive {
        set_ignore(app, true);
    }
    Ok(())
}

/// Restore the saved position if he would still be reachable there, else the default.
fn place(app: &AppHandle, win: &WebviewWindow, saved: Option<(i32, i32)>, height: f64) {
    let all = screens(win);
    let bounds: Vec<Rect> = all.iter().map(|s| s.bounds).collect();
    let restored = saved.and_then(|(x, y)| {
        let (x, y) = (x as f64, y as f64);
        let screen = all
            .iter()
            .find(|s| s.bounds.contains(x, y))
            .or_else(|| all.iter().find(|s| s.bounds.inflate(400.0).contains(x, y)))?;
        let (r, h) = fitted(screen, x, y, height);
        knight::is_reachable(r, &bounds, REACHABLE_MIN * screen.scale).then_some((r, h))
    });
    // `height` stays the user's intended size; only the position is saved here.
    if let Some((r, _)) = restored.or_else(|| default_placement(win, height)) {
        apply_rect(win, r);
        update(app, |s| (s.x, s.y) = (Some(r.x as i32), Some(r.y as i32)));
    }
}

/// The page has drawn the knight; show the window if he should be visible.
pub fn reveal(app: &AppHandle) {
    state(app).revealed.store(true, Ordering::Relaxed);
    if settings(app).visible
        && let Some(win) = knight_window(app)
    {
        let _ = win.show();
    }
}

pub fn reset_position(app: &AppHandle) {
    let Some(win) = knight_window(app) else { return };
    let height = settings(app).height;
    if let Some((r, _)) = default_placement(&win, height) {
        apply_rect(&win, r);
        update(app, |s| (s.x, s.y) = (Some(r.x as i32), Some(r.y as i32)));
    }
}

/// Snap him back if a display change or bad coordinates left him out of reach.
pub fn ensure_reachable(app: &AppHandle) {
    let (Some(win), Some(r)) = (knight_window(app), current_rect(app)) else { return };
    let all = screens(&win);
    if all.is_empty() {
        return; // Monitor enumeration failed; don't guess.
    }
    let scale = dpi(app).unwrap_or(1.0);
    let bounds: Vec<Rect> = all.iter().map(|s| s.bounds).collect();
    if !knight::is_reachable(r, &bounds, REACHABLE_MIN * scale) {
        reset_position(app);
    }
}

fn outer_rect(win: &WebviewWindow) -> Option<Rect> {
    let (p, s) = (win.outer_position().ok()?, win.outer_size().ok()?);
    Some(Rect::new(p.x as f64, p.y as f64, s.width as f64, s.height as f64))
}

/// The knight's current whole-number scale, the window's DPI scale factor, and the
/// height of the work area he stands in (physical px).
pub fn scale_info(app: &AppHandle) -> Option<(u32, f64, f64)> {
    let win = knight_window(app)?;
    let current = current_rect(app)?;
    let dpi = dpi(app).unwrap_or(1.0);
    let work_h = screen_for(&win, current).map_or(1080.0 * dpi, |s| s.work.h);
    Some((knight::scale_of(current.h), dpi, work_h))
}

/// Size the window to the whole-number scale nearest `height` (logical), feet kept in
/// place. Returns the scale applied and the DPI it was applied at.
fn fit(app: &AppHandle, height: f64) -> Option<(u32, f64)> {
    let win = knight_window(app)?;
    let current = current_rect(app)?;
    let (_, dpi, work_h) = scale_info(app)?;
    let k = knight::scale_for(height, dpi, work_h);
    let (w, h) = knight::window_px(k);
    let next = knight::anchored_resize(current, w, h);
    if next != current {
        apply_rect(&win, next);
    }
    Some((k, dpi))
}

/// The user chose a size: apply it and remember it as his intended size. Refused, with
/// a word from him, while his position is locked.
pub fn resize_to(app: &AppHandle, height: f64) {
    if settings(app).locked {
        hold_ground(app);
        return;
    }
    if let Some((k, dpi)) = fit(app, height) {
        update(app, |s| s.height = knight::height_for(k, dpi));
    }
}

/// Re-snap to the whole-number scale for his intended size on the current display,
/// leaving that intended size untouched (so crossing displays never drifts it).
pub fn refit(app: &AppHandle) {
    let height = settings(app).height;
    fit(app, height);
}

/// Grow (`steps` > 0) or shrink by whole pixel-scale steps.
pub fn resize_step(app: &AppHandle, steps: i32) {
    let Some((k, dpi, _)) = scale_info(app) else { return };
    let target = (k as i32 + steps).max(knight::MIN_SCALE as i32) as u32;
    resize_to(app, knight::height_for(target, dpi));
}

const HOLD_LINE: &str = "I hold this ground.";

/// Someone tried to move or resize him while locked: say so, once per showing.
pub fn hold_ground(app: &AppHandle) {
    let st = state(app);
    let showing = st.whisper_shown.load(Ordering::Relaxed)
        && st.hold_seq.load(Ordering::Relaxed) == st.whisper_seq.load(Ordering::Relaxed);
    if !showing {
        say_hold(app);
    }
}

fn say_hold(app: &AppHandle) {
    if let Some(seq) = whisper(app, Some(HOLD_LINE)) {
        state(app).hold_seq.store(seq, Ordering::Relaxed);
    }
}

pub fn set_visible(app: &AppHandle, visible: bool) {
    let Some(win) = knight_window(app) else { return };
    if visible {
        ensure_reachable(app);
        if state(app).revealed.load(Ordering::Relaxed) {
            let _ = win.show();
        }
    } else {
        let _ = win.hide();
        hide_whisper(app);
    }
    update(app, |s| s.visible = visible);
}

pub fn toggle_visible(app: &AppHandle) {
    let visible = settings(app).visible;
    set_visible(app, !visible);
}

pub fn set_always_on_top(app: &AppHandle, on: bool) {
    if let Some(win) = knight_window(app) {
        let _ = win.set_always_on_top(on);
    }
    update(app, |s| s.always_on_top = on);
}

pub fn set_locked(app: &AppHandle, on: bool) {
    update(app, |s| s.locked = on);
    if on {
        say_hold(app);
    } else {
        whisper(app, Some("I am free to move."));
    }
}

pub fn set_passive(app: &AppHandle, on: bool) {
    // Turning passive off makes the whole window clickable at once; the ticker then
    // refines it back to the silhouette as the cursor moves.
    set_ignore(app, on);
    update(app, |s| s.passive = on);
}

pub fn set_stand_watch(app: &AppHandle, on: bool) {
    update(app, |s| s.stand_watch = on);
    let line = if on { "I stand ready." } else { "The watch is ended. Rest well." };
    whisper(app, Some(line));
}

/// Global shortcut / second launch: make him visible, clickable and on screen.
pub fn summon(app: &AppHandle) {
    let s = settings(app);
    if !s.visible {
        set_visible(app, true);
    }
    if s.passive {
        set_passive(app, false);
    }
    ensure_reachable(app);
}

/// Let clicks pass through the knight's window (`true`) or catch them (`false`).
///
/// The cached state and the native call change together on the main thread only, so
/// requests from the ticker and the menu can't land out of order and leave the cache
/// claiming a state the window doesn't have. Off the main thread the cache is only a
/// hint to skip posting a no-op.
pub fn set_ignore(app: &AppHandle, ignore: bool) {
    if *state(app).ignoring.lock().unwrap() == Some(ignore) {
        return;
    }
    let handle = app.clone();
    // Runs inline when already on the main thread.
    let _ = app.run_on_main_thread(move || {
        let st = state(&handle);
        if *st.ignoring.lock().unwrap() == Some(ignore) {
            return;
        }
        let applied =
            knight_window(&handle).is_some_and(|w| w.set_ignore_cursor_events(ignore).is_ok());
        *st.ignoring.lock().unwrap() = applied.then_some(ignore);
    });
}

/// Show a line by him: `Some(text)` verbatim, or `None` for a quote of the
/// whisper window's choosing. Returns the line's sequence number if it was shown.
pub fn whisper(app: &AppHandle, text: Option<&str>) -> Option<u64> {
    let knight = knight_window(app)?;
    let whisper = app.get_webview_window(WHISPER)?;
    let s = settings(app);
    if !s.visible {
        return None;
    }
    let st = state(app);
    let rect = current_rect(app)?;
    let scale = dpi(app).unwrap_or(1.0);
    let work = screen_for(&knight, rect).map_or(rect, |s| s.work);
    let (r, placement) =
        knight::whisper_rect(rect, work, WHISPER_SIZE.0 * scale, WHISPER_SIZE.1 * scale);
    let _ = whisper.set_position(PhysicalPosition::new(r.x as i32, r.y as i32));
    let seq = st.whisper_seq.fetch_add(1, Ordering::Relaxed) + 1;
    st.whisper_shown.store(true, Ordering::Relaxed);
    let _ = whisper.show();
    raise_above_knight(&whisper);
    let _ = app.emit_to(
        WHISPER,
        "sentinel://whisper",
        WhisperPayload {
            seq,
            text: text.map(str::to_owned),
            placement,
            font_px: (s.height * 0.05).clamp(16.0, 26.0),
        },
    );
    Some(seq)
}

/// Clicking the knight lifts him to the top of the always-on-top band; put the quote
/// back above him so his armour never hides the words.
fn raise_above_knight(whisper: &WebviewWindow) {
    #[cfg(windows)]
    if let Ok(hwnd) = whisper.hwnd() {
        use windows::Win32::UI::WindowsAndMessaging::{
            HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos,
        };
        // SAFETY: hwnd belongs to a live window owned by this process.
        let _ = unsafe {
            SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE)
        };
        return;
    }
    let _ = whisper.set_always_on_top(true);
}

/// The whisper window finished fading line `seq`; hide it unless a newer one began.
pub fn whisper_done(app: &AppHandle, seq: u64) {
    if state(app).whisper_seq.load(Ordering::Relaxed) == seq {
        hide_whisper(app);
    }
}

fn hide_whisper(app: &AppHandle) {
    if state(app).whisper_shown.swap(false, Ordering::Relaxed)
        && let Some(w) = app.get_webview_window(WHISPER)
    {
        let _ = w.hide();
    }
}

pub fn exit(app: &AppHandle) {
    state(app).flush();
    app.exit(0);
}

pub fn on_window_event(win: &Window, event: &WindowEvent) {
    if win.label() != KNIGHT {
        return;
    }
    let app = win.app_handle();
    let st = state(app);
    match event {
        WindowEvent::Moved(p) => {
            {
                let mut s = st.settings.lock().unwrap();
                (s.x, s.y) = (Some(p.x), Some(p.y));
            }
            st.mark_dirty();
            // A quote left hanging at his old spot looks detached.
            hide_whisper(app);
        }
        // A scale-factor change is handled by the ticker: Windows resizes the window
        // itself after this event, so a refit here would be overwritten.
        WindowEvent::CloseRequested { .. } => exit(app),
        _ => {}
    }
}
