//! A slow background loop that keeps the knight well-behaved without any UI work:
//!
//! - **Silhouette hit-testing.** Clicks only land on his opaque pixels. While the cursor
//!   is near his window the alpha mask decides whether the window ignores the mouse.
//! - **Recovery.** If a display change leaves him off every screen, he's moved back.
//! - **Crisp pixels at a steady size.** When his window's DPI changes (he crossed to a
//!   display with different scaling) or something resizes it off a whole-number pixel
//!   scale (Windows may, on that same move), he's re-snapped once things settle,
//!   rather than racing the OS's own resize.
//! - **The Stand Watch line** comes back once he has held still and no other line is
//!   showing (D14).
//! - **Debounced saving** of settings.
//!
//! Polls at ~30 Hz only while the cursor is near him, ~10 Hz otherwise. While the
//! global cursor can't be read (Wayland; Windows' secure desktop), the whole window
//! catches clicks and the ticker retries every half second.

use std::{
    thread,
    time::{Duration, Instant},
};

use tauri::{AppHandle, Manager};

use crate::{AppState, knight, stage};

const NEAR_TICK: Duration = Duration::from_millis(33);
const FAR_TICK: Duration = Duration::from_millis(100);
const HIDDEN_TICK: Duration = Duration::from_millis(300);
/// Retry interval while the global cursor can't be read.
const NO_CURSOR_TICK: Duration = Duration::from_millis(500);
const REACHABILITY_EVERY: Duration = Duration::from_secs(3);
/// How long a changed DPI or off-scale size must hold still before he's re-snapped.
const SNAP_AFTER: Duration = Duration::from_millis(150);
/// How long he must hold still before the Stand Watch line comes back, so it doesn't
/// flicker in and out during a drag or a run of resizes.
const LINE_AFTER: Duration = Duration::from_millis(600);
/// Physical px around his window within which the cursor counts as "near".
const NEAR_MARGIN: f64 = 64.0;

pub fn spawn(app: AppHandle) {
    thread::Builder::new()
        .name("sentinel-ticker".into())
        .spawn(move || run(app))
        .expect("spawn ticker thread");
}

fn run(app: AppHandle) {
    let mut last_check = Instant::now();
    let mut applied_dpi: Option<f64> = None;
    let mut unsettled: Option<((f64, f64, f64), Instant)> = None;
    let mut still: Option<(knight::Rect, Instant)> = None;
    loop {
        let st = app.state::<AppState>();
        st.flush_if_due();

        let (visible, passive) = {
            let s = st.settings.lock().unwrap();
            (s.visible, s.passive)
        };
        if !visible || !st.revealed.load(std::sync::atomic::Ordering::Relaxed) {
            thread::sleep(HIDDEN_TICK);
            continue;
        }
        if last_check.elapsed() >= REACHABILITY_EVERY {
            stage::ensure_reachable(&app);
            last_check = Instant::now();
        }
        if let (Some(r), Some(dpi)) = (stage::current_rect(&app), stage::dpi(&app)) {
            let applied = *applied_dpi.get_or_insert(dpi);
            let now = (r.w, r.h, dpi);
            unsettled = match unsettled {
                _ if knight::is_exact(r.w, r.h) && dpi == applied => None,
                Some((seen, since)) if seen == now && since.elapsed() >= SNAP_AFTER => {
                    stage::refit(&app);
                    applied_dpi = Some(dpi);
                    None
                }
                Some((seen, since)) if seen == now => Some((seen, since)),
                _ => Some((now, Instant::now())),
            };
            still = match still {
                Some((seen, since)) if seen == r => {
                    if since.elapsed() >= LINE_AFTER {
                        // Cheap when there's nothing to do: a few atomic loads and the
                        // settings lock, with no call to the main thread.
                        stage::restore_standing_line(&app, seen);
                    }
                    Some((seen, since))
                }
                _ => Some((r, Instant::now())),
            };
        }
        if passive {
            stage::set_ignore(&app, true);
            thread::sleep(FAR_TICK);
            continue;
        }

        let Some(rect) = stage::current_rect(&app) else {
            thread::sleep(FAR_TICK);
            continue;
        };
        let tick = match cursor(&app) {
            Ok((x, y)) if rect.inflate(NEAR_MARGIN).contains(x, y) => {
                let opaque = knight::is_opaque(x - rect.x, y - rect.y, rect.w, rect.h);
                stage::set_ignore(&app, !opaque);
                NEAR_TICK
            }
            Ok(_) => FAR_TICK,
            // No global cursor (Wayland), or briefly none (Windows' secure desktop while
            // locked): the whole window catches clicks, and we keep trying, more slowly.
            Err(_) => {
                stage::set_ignore(&app, false);
                NO_CURSOR_TICK
            }
        };
        thread::sleep(tick);
    }
}

/// Global cursor position in physical px.
fn cursor(app: &AppHandle) -> tauri::Result<(f64, f64)> {
    #[cfg(windows)]
    {
        use windows::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};
        let mut p = POINT::default();
        // SAFETY: writes only to the local POINT.
        if unsafe { GetCursorPos(&mut p) }.is_ok() {
            return Ok((p.x as f64, p.y as f64));
        }
    }
    app.cursor_position().map(|c| (c.x, c.y))
}
