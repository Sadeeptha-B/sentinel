//! A slow background loop that keeps the knight well-behaved without any UI work:
//!
//! - **Silhouette hit-testing.** Clicks only land on his opaque pixels. While the cursor
//!   is near his window the alpha mask decides whether the window ignores the mouse.
//! - **Recovery.** If a display change leaves him off every screen, he's moved back.
//! - **Debounced saving** of settings.
//!
//! Polls at ~30 Hz only while the cursor is near him, ~10 Hz otherwise. Where the
//! platform can't report the global cursor (Wayland), the whole window catches clicks.

use std::{
    thread,
    time::{Duration, Instant},
};

use tauri::{AppHandle, Manager};

use crate::{AppState, knight, stage};

const NEAR_TICK: Duration = Duration::from_millis(33);
const FAR_TICK: Duration = Duration::from_millis(100);
const HIDDEN_TICK: Duration = Duration::from_millis(300);
const REACHABILITY_EVERY: Duration = Duration::from_secs(3);
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
    let mut cursor_supported = true;
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
        if passive || !cursor_supported {
            stage::set_ignore(&app, passive);
            thread::sleep(FAR_TICK);
            continue;
        }

        let Some(rect) = stage::current_rect(&app) else {
            thread::sleep(FAR_TICK);
            continue;
        };
        let near = match cursor(&app) {
            Ok((x, y)) if rect.inflate(NEAR_MARGIN).contains(x, y) => {
                let opaque = knight::is_opaque(x - rect.x, y - rect.y, rect.w, rect.h);
                stage::set_ignore(&app, !opaque);
                true
            }
            Ok(_) => false,
            Err(_) => {
                cursor_supported = false;
                false
            }
        };
        thread::sleep(if near { NEAR_TICK } else { FAR_TICK });
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
