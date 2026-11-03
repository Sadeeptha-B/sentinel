//! The handful of preferences Sentinel remembers, stored as JSON in the app config dir.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::knight::DEFAULT_HEIGHT;

/// How long settings must stay unchanged before they're written.
const SAVE_DELAY: Duration = Duration::from_millis(500);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// Outer window position in physical pixels; `None` until first placed.
    pub x: Option<i32>,
    pub y: Option<i32>,
    /// Figure height in logical pixels.
    pub height: f64,
    pub visible: bool,
    pub always_on_top: bool,
    pub locked: bool,
    pub passive: bool,
    pub stand_watch: bool,
    /// Global "summon" shortcut: shows him, leaves passive mode, brings him on screen.
    pub shortcut: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            height: DEFAULT_HEIGHT,
            visible: true,
            always_on_top: true,
            locked: false,
            passive: false,
            stand_watch: false,
            shortcut: "Ctrl+Alt+K".into(),
        }
    }
}

/// Missing or unreadable settings fall back to defaults rather than failing launch.
pub fn load(path: &Path) -> Settings {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Write via a temp file and rename so a crash mid-write cannot corrupt the settings.
pub fn save(path: &Path, settings: &Settings) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(settings)?)?;
    fs::rename(tmp, path)
}

/// Debounced saving to one file, in order, retried until it succeeds.
pub struct Saver {
    path: PathBuf,
    dirty: Mutex<Option<Instant>>,
}

impl Saver {
    pub fn new(path: PathBuf) -> Self {
        Self { path, dirty: Mutex::new(None) }
    }

    pub fn mark_dirty(&self) {
        *self.dirty.lock().unwrap() = Some(Instant::now());
    }

    pub fn flush_if_due(&self, settings: &Mutex<Settings>) {
        let due = matches!(*self.dirty.lock().unwrap(), Some(t) if t.elapsed() >= SAVE_DELAY);
        if due {
            self.flush(settings);
        }
    }

    /// Write `settings` now if anything changed since the last save.
    pub fn flush(&self, settings: &Mutex<Settings>) {
        // Hold `dirty` until the file is written, so a ticker save and the exit save
        // can't commit their snapshots out of order (or share the temp file).
        // Lock order is always `dirty`, then `settings`.
        let mut dirty = self.dirty.lock().unwrap();
        if dirty.take().is_none() {
            return;
        }
        let snapshot = settings.lock().unwrap().clone();
        if let Err(e) = save(&self.path, &snapshot) {
            eprintln!("sentinel: could not save settings to {}: {e}", self.path.display());
            // Still unsaved: try again after the usual delay (e.g. once another
            // program lets go of the file).
            *dirty = Some(Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_save_is_retried() {
        let dir = std::env::temp_dir().join(format!("sentinel-flush-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        // A file where the settings folder should be makes the save fail.
        let blocker = dir.join("config");
        fs::write(&blocker, "").unwrap();
        let path = blocker.join("settings.json");
        let settings = Mutex::new(Settings { locked: true, ..Default::default() });
        let saver = Saver::new(path.clone());

        saver.mark_dirty();
        saver.flush(&settings);
        assert!(saver.dirty.lock().unwrap().is_some(), "still dirty after a failed save");

        fs::remove_file(&blocker).unwrap();
        saver.flush(&settings);
        assert!(saver.dirty.lock().unwrap().is_none());
        assert!(load(&path).locked, "the retry saved the latest settings");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn round_trips_and_tolerates_garbage() {
        let dir = std::env::temp_dir().join(format!("sentinel-test-{}", std::process::id()));
        let path = dir.join("settings.json");
        let s = Settings { x: Some(-1200), y: Some(300), height: 512.0, locked: true, ..Default::default() };
        save(&path, &s).unwrap();
        assert_eq!(load(&path), s);

        fs::write(&path, "{ not json").unwrap();
        assert_eq!(load(&path), Settings::default());

        fs::write(&path, r#"{"height": 333}"#).unwrap();
        assert_eq!(load(&path).height, 333.0);
        assert!(load(&path).visible, "missing fields take defaults");
        fs::remove_dir_all(dir).ok();
    }
}
