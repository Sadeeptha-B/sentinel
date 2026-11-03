//! The knight figure: embedded sprite metadata, hit mask and the pure geometry that
//! sizes, places and hit-tests his window. Everything here is in physical pixels unless
//! a name says `logical`.
//!
//! He is pixel art, crisp only at whole-number scales, so his size is a scale `k`
//! (physical pixels per sprite pixel). The window is exactly the sprite plus its
//! shadow band at `k`.

use std::sync::LazyLock;

use serde::Deserialize;

/// Logical figure height on first launch (snapped to the nearest whole scale).
pub const DEFAULT_HEIGHT: f64 = 400.0;
pub const MIN_SCALE: u32 = 2;
/// Never let the window take more than this share of the work area's height.
const MAX_SCREEN_SHARE: f64 = 0.9;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    /// Sprite size in pixels; `height` is the figure, without the shadow band.
    pub width: u32,
    pub height: u32,
    /// Rows below the figure holding the baked contact shadow.
    pub band_rows: u32,
    pub mask: MaskMeta,
    /// Horizontal extent of his stance (boots and blade), as fractions of the width.
    pub feet_left: f64,
    pub feet_right: f64,
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

fn window_rows() -> u32 {
    META.height + META.band_rows
}

/// Window size at whole-number scale `k`: the sprite plus its shadow band.
pub fn window_px(k: u32) -> (f64, f64) {
    ((META.width * k) as f64, (window_rows() * k) as f64)
}

/// The whole-number scale that best shows a figure `height` logical px tall on a
/// screen with scale factor `dpi`, keeping the window within `work_h` physical px.
pub fn scale_for(height: f64, dpi: f64, work_h: f64) -> u32 {
    let height = if height.is_finite() && height > 0.0 { height } else { DEFAULT_HEIGHT };
    let max = ((work_h * MAX_SCREEN_SHARE) / window_rows() as f64).floor().max(MIN_SCALE as f64);
    (height * dpi / META.height as f64).round().clamp(MIN_SCALE as f64, max) as u32
}

/// The logical figure height scale `k` gives on a screen with scale factor `dpi`.
pub fn height_for(k: u32, dpi: f64) -> f64 {
    (k * META.height) as f64 / dpi
}

/// The whole-number scale of a window `win_h` px tall.
pub fn scale_of(win_h: f64) -> u32 {
    ((win_h / window_rows() as f64).round() as u32).max(1)
}

/// Whether a `w`×`h` window is exactly the sprite at some whole-number scale.
pub fn is_exact(w: f64, h: f64) -> bool {
    window_px(scale_of(h)) == (w, h)
}

/// Height of the figure itself within a window `win_h` px tall (no shadow band).
pub fn figure_height(win_h: f64) -> f64 {
    win_h * META.height as f64 / window_rows() as f64
}

/// The line his boots stand on, as a share of the window's height: the bottom edge of
/// the lowest opaque mask row. The shadow band and any padding lie below it.
static FEET_LINE: LazyLock<f64> = LazyLock::new(|| {
    let m = &META.mask;
    let lowest = (0..m.height).rev().find(|&y| (0..m.width).any(|x| mask_bit(x, y))).unwrap_or(m.height - 1);
    (lowest + 1) as f64 / window_rows() as f64
});

/// Resize `outer` to `w`×`h` keeping his feet (the centre of the line he stands on)
/// where they were.
pub fn anchored_resize(outer: Rect, w: f64, h: f64) -> Rect {
    let feet = *FEET_LINE;
    Rect::new(
        (outer.x + outer.w / 2.0 - w / 2.0).round(),
        (outer.y + outer.h * feet - h * feet).round(),
        w,
        h,
    )
}

/// Area (physical px²) of the figure itself, not its transparent surroundings, that
/// lies on some screen.
fn visible_figure_area(win: Rect, screens: &[Rect]) -> f64 {
    let m = &META.mask;
    let (pw, ph) = (win.w / m.width as f64, figure_height(win.h) / m.height as f64);
    let shown = (0..m.height)
        .flat_map(|y| (0..m.width).map(move |x| (x, y)))
        .filter(|&(x, y)| mask_bit(x, y))
        .filter(|&(x, y)| {
            let (cx, cy) = (win.x + (x as f64 + 0.5) * pw, win.y + (y as f64 + 0.5) * ph);
            screens.iter().any(|s| s.contains(cx, cy))
        })
        .count();
    shown as f64 * pw * ph
}

/// He is reachable when at least `min`×`min` px of the figure itself is on screen
/// (or all of it, if he is smaller than that). Transparent window corners don't count.
pub fn is_reachable(win: Rect, screens: &[Rect], min: f64) -> bool {
    let whole = visible_figure_area(win, &[win]);
    visible_figure_area(win, screens) >= (min * min).min(whole)
}

fn mask_bit(x: usize, y: usize) -> bool {
    let row_bytes = META.mask.width.div_ceil(8);
    MASK.get(y * row_bytes + x / 8).is_some_and(|byte| byte >> (x % 8) & 1 == 1)
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
    let fig_h = figure_height(win_h);
    if !(x >= 0.0 && y >= 0.0 && x < win_w && y < fig_h) {
        return false;
    }
    let m = &META.mask;
    let mx = ((x / win_w) * m.width as f64) as usize;
    let my = ((y / fig_h) * m.height as f64) as usize;
    mask_bit(mx.min(m.width - 1), my.min(m.height - 1))
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Placement {
    /// Just beneath his feet.
    Below,
    /// Over the lower half of the figure, when there is no room beneath him.
    Over,
}

/// Where the quote window goes: centred on his stance, below his feet when the work
/// area has room for it, otherwise over the lower half of the figure (as when he
/// stands on the taskbar).
pub fn whisper_rect(knight: Rect, work: Rect, w: f64, h: f64) -> (Rect, Placement) {
    let fig_h = figure_height(knight.h);
    let centre = knight.x + knight.w * (META.feet_left + META.feet_right) / 2.0;
    let below = knight.y + fig_h * 0.985;
    let (y, placement) = if below + h <= work.bottom() {
        (below, Placement::Below)
    } else {
        (knight.y + fig_h * 0.72 - h / 2.0, Placement::Over)
    };
    let x = (centre - w / 2.0).clamp(work.x, (work.right() - w).max(work.x));
    let y = y.clamp(work.y, (work.bottom() - h).max(work.y));
    (Rect::new(x.round(), y.round(), w, h), placement)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Rect = Rect { x: 0.0, y: 0.0, w: 1920.0, h: 1080.0 };

    /// Screen y of the bottom edge of his lowest opaque pixel, read from the real mask.
    fn boots(r: Rect) -> f64 {
        let k = r.h / window_rows() as f64;
        let lowest = (0..META.height as usize)
            .rev()
            .find(|&y| (0..META.width as usize).any(|x| mask_bit(x, y)))
            .unwrap();
        r.y + (lowest + 1) as f64 * k
    }

    #[test]
    fn anchored_resize_keeps_feet_in_place() {
        let (w4, h4) = window_px(4);
        let r = Rect::new(1000.0, 400.0, w4, h4);
        for k in [MIN_SCALE, 3, 5, 6] {
            let (w, h) = window_px(k);
            let next = anchored_resize(r, w, h);
            assert_eq!(boots(next), boots(r), "k 4 -> {k}");
            assert_eq!(next.x + next.w / 2.0, r.x + r.w / 2.0);
        }
    }

    #[test]
    fn reachability_requires_a_meaningful_chunk_of_him_on_screen() {
        let (w, h) = window_px(4);
        let on = Rect::new(1700.0, 600.0, w, h);
        let sliver = Rect::new(1900.0, 600.0, w, h);
        let gone = Rect::new(4000.0, 600.0, w, h);
        assert!(is_reachable(on, &[SCREEN], 64.0));
        assert!(!is_reachable(sliver, &[SCREEN], 64.0));
        assert!(!is_reachable(gone, &[SCREEN], 64.0));
        let second = Rect::new(1920.0, 0.0, 1920.0, 1080.0);
        assert!(is_reachable(sliver, &[SCREEN, second], 64.0));
    }

    #[test]
    fn transparent_corner_on_screen_is_not_reachable() {
        // Only the window's empty top-left 64×64 corner is on the screen.
        let (w, h) = window_px(5);
        let corner = Rect::new(SCREEN.right() - 64.0, SCREEN.bottom() - 64.0, w, h);
        assert!(!is_reachable(corner, &[SCREEN], 64.0));
    }

    #[test]
    fn default_rect_sits_bottom_right_inside_work_area() {
        let work = Rect::new(0.0, 0.0, 1920.0, 1032.0);
        let r = default_rect(work, 240.0, 412.0, 24.0);
        assert_eq!(r.right(), 1920.0 - 24.0);
        assert_eq!(r.bottom(), 1032.0);
    }

    #[test]
    fn scale_snaps_to_whole_pixels_and_respects_limits() {
        let k = |h, dpi| scale_for(h, dpi, 1032.0 * dpi);
        let expect = |h: f64, dpi: f64| (h * dpi / META.height as f64).round() as u32;
        assert_eq!(k(400.0, 1.0), expect(400.0, 1.0));
        assert_eq!(k(400.0, 1.25), expect(400.0, 1.25));
        assert_eq!(k(400.0, 1.5), expect(400.0, 1.5));
        assert_eq!(k(10.0, 1.0), MIN_SCALE);
        assert_eq!(k(f64::NAN, 1.0), expect(DEFAULT_HEIGHT, 1.0));
        assert_eq!(k(-5.0, 1.0), expect(DEFAULT_HEIGHT, 1.0));
        let huge = k(1e6, 1.0);
        assert!(window_px(huge).1 <= 1032.0 * MAX_SCREEN_SHARE, "fits the work area");
        assert_eq!(scale_for(1e6, 1.0, 50.0), MIN_SCALE, "never below the minimum");
    }

    #[test]
    fn scale_round_trips_and_window_is_an_exact_multiple() {
        for dpi in [1.0, 1.25, 1.5, 1.75, 2.0] {
            for k in MIN_SCALE..=8 {
                assert_eq!(scale_for(height_for(k, dpi), dpi, 1e5), k, "dpi {dpi} k {k}");
                let (w, h) = window_px(k);
                assert_eq!(w, (META.width * k) as f64);
                assert_eq!(h, ((META.height + META.band_rows) * k) as f64);
                assert_eq!(scale_of(h), k);
                assert!(is_exact(w, h));
                assert!(!is_exact(w + 1.0, h) && !is_exact(w, h - 1.0));
                assert_eq!(figure_height(h), (META.height * k) as f64);
            }
        }
    }

    #[test]
    fn a_dpi_rescaled_window_is_off_scale() {
        // Windows shrinks a window by 96/120 when it moves from a 125% to a 100% display.
        let (w, h) = window_px(4);
        assert!(!is_exact((w * 0.8).round(), (h * 0.8).round()));
    }

    #[test]
    fn mask_matches_the_sprite() {
        assert_eq!(META.mask.width, META.width as usize);
        assert_eq!(META.mask.height, META.height as usize);
        assert_eq!(MASK.len(), META.mask.width.div_ceil(8) * META.mask.height);
    }

    #[test]
    fn hit_mask_is_solid_at_centre_and_clear_at_corners() {
        let (w, h) = window_px(4);
        assert!(is_opaque(w / 2.0, h * 0.5, w, h), "middle of the figure");
        assert!(!is_opaque(2.0, 2.0, w, h), "top-left corner");
        assert!(!is_opaque(w - 3.0, 2.0, w, h), "top-right corner");
        assert!(!is_opaque(w / 2.0, h - 1.0, w, h), "shadow band");
        assert!(!is_opaque(-1.0, 10.0, w, h));
    }

    #[test]
    fn whisper_goes_below_when_there_is_room_else_over_his_lower_half() {
        let work = Rect::new(0.0, 0.0, 1920.0, 1032.0);
        let standing_mid = Rect::new(1000.0, 300.0, 240.0, 412.0);
        let (r, placement) = whisper_rect(standing_mid, work, 420.0, 150.0);
        assert_eq!(placement, Placement::Below);
        assert!(r.y >= standing_mid.y + figure_height(412.0) * 0.98);

        let on_taskbar = default_rect(work, 240.0, 412.0, 24.0);
        let (r, placement) = whisper_rect(on_taskbar, work, 420.0, 150.0);
        assert_eq!(placement, Placement::Over);
        assert!(r.y > on_taskbar.y + on_taskbar.h * 0.4, "lower half");
        assert!(r.right() <= work.right(), "kept on screen");
    }
}
