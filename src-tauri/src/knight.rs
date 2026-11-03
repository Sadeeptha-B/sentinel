//! The knight figure: embedded asset metadata, alpha hit mask and the pure geometry
//! that sizes, places and hit-tests his window. Everything here is in physical pixels
//! unless a name says `logical`.

use std::sync::LazyLock;

use serde::Deserialize;

/// Extra window height below the figure, as a fraction of figure height, that holds
/// the soft contact shadow. Must match `SHADOW_BAND` in `src/Knight.tsx`.
pub const SHADOW_BAND: f64 = 0.03;
pub const MIN_HEIGHT: f64 = 200.0;
pub const DEFAULT_HEIGHT: f64 = 400.0;
/// Never draw the figure more than this much larger than the source art.
const MAX_UPSCALE: f64 = 1.15;
/// Never let the figure take more than this share of the work area's height.
const MAX_SCREEN_SHARE: f64 = 0.9;

#[derive(Deserialize)]
pub struct Meta {
    pub height: u32,
    pub aspect: f64,
    pub mask: MaskMeta,
}

#[derive(Deserialize)]
pub struct MaskMeta {
    pub width: usize,
    pub height: usize,
}

pub static META: LazyLock<Meta> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../assets/knight.meta.json")).expect("valid knight meta")
});
static MASK: &[u8] = include_bytes!("../assets/knight.hitmask.bin");

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }
    pub fn right(&self) -> f64 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }
    pub fn inflate(&self, by: f64) -> Rect {
        Rect::new(self.x - by, self.y - by, self.w + 2.0 * by, self.h + 2.0 * by)
    }
    /// Width and height of the overlap with `other` (zero when disjoint).
    pub fn overlap(&self, other: &Rect) -> (f64, f64) {
        let w = self.right().min(other.right()) - self.x.max(other.x);
        let h = self.bottom().min(other.bottom()) - self.y.max(other.y);
        (w.max(0.0), h.max(0.0))
    }
}

/// Logical window size for a figure `height` logical pixels tall.
pub fn window_size(height: f64) -> (f64, f64) {
    (height * META.aspect, height * (1.0 + SHADOW_BAND))
}

/// Clamp a requested figure height (logical) to what looks good on a screen with the
/// given scale factor and logical work-area height.
pub fn clamp_height(height: f64, scale: f64, work_height_logical: f64) -> f64 {
    let art_limit = META.height as f64 * MAX_UPSCALE / scale;
    let screen_limit = work_height_logical * MAX_SCREEN_SHARE / (1.0 + SHADOW_BAND);
    let max = art_limit.min(screen_limit).max(MIN_HEIGHT);
    if height.is_finite() { height.clamp(MIN_HEIGHT, max) } else { DEFAULT_HEIGHT.min(max) }
}

/// Resize `outer` to `w`×`h` keeping the bottom-centre — his feet — where it was.
pub fn anchored_resize(outer: Rect, w: f64, h: f64) -> Rect {
    Rect::new(
        (outer.x + outer.w / 2.0 - w / 2.0).round(),
        (outer.bottom() - h).round(),
        w,
        h,
    )
}

/// A window is reachable when a `min`×`min` part of it lies on some screen.
pub fn is_reachable(win: Rect, screens: &[Rect], min: f64) -> bool {
    screens.iter().any(|s| {
        let (w, h) = win.overlap(s);
        w >= min.min(win.w) && h >= min.min(win.h)
    })
}

/// Default spot: bottom-right of the work area, shadow resting on the taskbar edge.
pub fn default_rect(work: Rect, w: f64, h: f64, margin: f64) -> Rect {
    Rect::new(
        (work.right() - w - margin).round().max(work.x),
        (work.bottom() - h).round().max(work.y),
        w,
        h,
    )
}

/// Whether window-local point (`x`, `y`) lands on the knight himself, for a window of
/// `win_w`×`win_h`. The shadow band and transparent surroundings are not opaque.
pub fn is_opaque(x: f64, y: f64, win_w: f64, win_h: f64) -> bool {
    let fig_h = win_h / (1.0 + SHADOW_BAND);
    if !(x >= 0.0 && y >= 0.0 && x < win_w && y < fig_h) {
        return false;
    }
    let m = &META.mask;
    let mx = ((x / win_w) * m.width as f64) as usize;
    let my = ((y / fig_h) * m.height as f64) as usize;
    let row_bytes = m.width.div_ceil(8);
    MASK.get(my.min(m.height - 1) * row_bytes + mx.min(m.width - 1) / 8)
        .is_some_and(|byte| byte >> (mx.min(m.width - 1) % 8) & 1 == 1)
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Left,
    Right,
}

/// Where the quote window goes: beside his shoulders, on whichever side of the
/// work area has more room, overlapping the empty margin of his own window.
pub fn whisper_rect(knight: Rect, work: Rect, w: f64, h: f64) -> (Rect, Side) {
    let fig_h = knight.h / (1.0 + SHADOW_BAND);
    let overlap = knight.w * 0.16;
    let left_room = knight.x + overlap - work.x;
    let right_room = work.right() - (knight.right() - overlap);
    let side = if left_room >= w || left_room >= right_room { Side::Left } else { Side::Right };
    let x = match side {
        Side::Left => knight.x + overlap - w,
        Side::Right => knight.right() - overlap,
    };
    let y = knight.y + fig_h * 0.17 - h / 2.0;
    let x = x.clamp(work.x, (work.right() - w).max(work.x));
    let y = y.clamp(work.y, (work.bottom() - h).max(work.y));
    (Rect::new(x.round(), y.round(), w, h), side)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Rect = Rect { x: 0.0, y: 0.0, w: 1920.0, h: 1080.0 };

    #[test]
    fn anchored_resize_keeps_feet_in_place() {
        let r = Rect::new(1000.0, 500.0, 240.0, 412.0);
        let grown = anchored_resize(r, 300.0, 515.0);
        assert_eq!(grown.bottom(), r.bottom());
        assert_eq!(grown.x + grown.w / 2.0, r.x + r.w / 2.0);
    }

    #[test]
    fn reachability_requires_a_meaningful_chunk_on_screen() {
        let on = Rect::new(1700.0, 600.0, 240.0, 412.0);
        let sliver = Rect::new(1900.0, 600.0, 240.0, 412.0);
        let gone = Rect::new(4000.0, 600.0, 240.0, 412.0);
        assert!(is_reachable(on, &[SCREEN], 64.0));
        assert!(!is_reachable(sliver, &[SCREEN], 64.0));
        assert!(!is_reachable(gone, &[SCREEN], 64.0));
        let second = Rect::new(1920.0, 0.0, 1920.0, 1080.0);
        assert!(is_reachable(sliver, &[SCREEN, second], 64.0));
    }

    #[test]
    fn default_rect_sits_bottom_right_inside_work_area() {
        let work = Rect::new(0.0, 0.0, 1920.0, 1032.0);
        let r = default_rect(work, 240.0, 412.0, 24.0);
        assert_eq!(r.right(), 1920.0 - 24.0);
        assert_eq!(r.bottom(), 1032.0);
    }

    #[test]
    fn clamp_height_respects_art_and_screen_limits() {
        assert_eq!(clamp_height(50.0, 1.0, 1000.0), MIN_HEIGHT);
        assert!(clamp_height(5000.0, 1.0, 1000.0) <= 1000.0 * MAX_SCREEN_SHARE);
        assert!(clamp_height(5000.0, 2.0, 4000.0) <= META.height as f64 * MAX_UPSCALE / 2.0);
        assert_eq!(clamp_height(f64::NAN, 1.0, 1000.0), DEFAULT_HEIGHT);
    }

    #[test]
    fn hit_mask_is_solid_at_centre_and_clear_at_corners() {
        let (w, h) = (META.aspect * 400.0, 400.0 * (1.0 + SHADOW_BAND));
        assert!(is_opaque(w / 2.0, h * 0.5, w, h), "chest");
        assert!(!is_opaque(2.0, 2.0, w, h), "top-left corner");
        assert!(!is_opaque(w - 3.0, 2.0, w, h), "top-right corner");
        assert!(!is_opaque(w / 2.0, h - 1.0, w, h), "shadow band");
        assert!(!is_opaque(-1.0, 10.0, w, h));
    }

    #[test]
    fn whisper_prefers_the_roomier_side() {
        let work = Rect::new(0.0, 0.0, 1920.0, 1032.0);
        let near_right = Rect::new(1650.0, 600.0, 240.0, 412.0);
        let (r, side) = whisper_rect(near_right, work, 360.0, 160.0);
        assert_eq!(side, Side::Left);
        assert!(r.right() <= near_right.x + near_right.w * 0.5);
        let near_left = Rect::new(10.0, 600.0, 240.0, 412.0);
        let (r, side) = whisper_rect(near_left, work, 360.0, 160.0);
        assert_eq!(side, Side::Right);
        assert!(r.x >= near_left.x + near_left.w * 0.5);
    }
}
