//! The handful of preferences Sentinel remembers, stored as JSON in the app config dir.

use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::knight::DEFAULT_HEIGHT;

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

#[cfg(test)]
mod tests {
    use super::*;

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
