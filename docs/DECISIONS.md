# Decisions

Choices the owner has made about Sentinel, and why. Read this before proposing a change in any of these areas: each entry is settled until the owner reopens it.

- **This file only grows.** Add new entries at the bottom. When a decision that is already on `main` changes, set the old entry's status to `Superseded by D<n>` and write a new entry, rather than rewriting history.
- **On a branch, iterate in place.** An entry that exists only on an unmerged branch is a draft. While the owner is still deciding, edit it directly instead of stacking superseded entries for choices that never shipped. If an earlier option was tried on the branch and rejected, list it under the entry's *alternatives*. Superseding starts once the entry is on `main`.
- **An entry is a choice that could reasonably have gone another way**: product behaviour, art direction, scope, platform support, workflow. Correctness rules that must hold are *invariants*, not decisions; they live in the code and tests and are indexed in [`AGENTS.md`](../AGENTS.md).
- The product brief, [`sentinel.md`](sentinel.md), is the baseline. Record a decision here when it departs from, narrows or interprets the brief.

Format: **what** was decided, **why**, the **alternatives** rejected, and anything it **supersedes**.

---

### D1. The knight is a hand-authored pixel sprite, "v3 stocky" build
*2026-11-03 · Active · Supersedes the brief's §4.2 painted cutout*

- **What:** `tools/asset-pipeline/pixel_knight.py` generates the knight from shapes, shading rules and hand-placed pixels, designed from the brief and not traced from the painting. The app uses the `stocky` build. `slim` is kept for comparison only.
- **Why:** the extracted painting looked pasted on and vanished on dark themes. The owner chose v3 stocky after comparing the cutout, scripted pixelisation, a heraldic silhouette, stained glass and three hand-authored passes.
- **Alternatives rejected:** the BiRefNet cutout of the painting (`extract.py`, now reference tooling only); the v1 and v2 sprites; slim proportions.

### D2. Whole-number pixel scaling only
*2026-11-03 · Active*

- **What:** his size is a scale `k` (physical px per sprite px). The window is exactly `58k × 96k` physical px. Small, Medium and Large are 3×, 4× and 5× at 100% DPI. On other displays he snaps to the scale with about the same apparent size.
- **Why:** pixel art blurs at fractional scales.
- **Alternatives rejected:** free resizing with smoothing.

### D3. Quotes appear below his feet when there's room, else over his lower half
*2026-11-03 · Active · Supersedes the earlier "beside him" placement*

- **Why:** the owner's explicit choice.

### D4. Per-pixel (silhouette) click-through, plus an explicit Passive mode
*2026-11-03 · Active*

- **What:** only his opaque pixels catch clicks. Passive makes him fully click-through. `Ctrl+Alt+K` always undoes Hide, Passive and an off-screen position in one step, and the tray menu offers each separately.
- **Alternatives rejected:** whole-window click catching, or Passive as the only option.

### D5. Effects: only a soft contact shadow, baked into the sprite
*2026-11-03 · Active*

- **What:** two stepped alpha levels in 3 rows below his feet. No window glows or other effects. The cloak's cool rim light is part of the art.
- **Why:** the brief's visual dignity (§3.4).

### D6. Quote typography
*2026-11-03 · Active*

- **What:** EB Garamond italic in muted gold, with a feathered dark shadow and no card or bubble.
- **Why:** from the brief. It stayed legible over busy windows in testing.
- **Alternatives rejected for now:** a pixel-style serif. Revisit only if the owner asks.

### D7. Lock Position blocks every resize, not just dragging
*2026-11-03 · Active*

- **What:** while locked, Rust refuses user resizes from every path (the Size menu, Ctrl+wheel, Ctrl+= and Ctrl+-), and the Size submenu is disabled. Automatic DPI refits still happen.
- **Why:** matches the documented meaning of Lock. Enforcing it in Rust means a new resize path can't bypass it.
- **Alternatives rejected:** allowing deliberate menu resizes while locked.

### D8. He says when he's locked; there's no lock badge
*2026-11-03 · Active*

- **What:**
  - Locking: "I hold this ground."
  - Unlocking: "I am free to move."
  - A blocked drag or resize repeats the hold line, unless it is still showing.
- **Why:** gives feedback in his own voice, only in response to the user, with nothing visible while idle.
- **Alternatives rejected:** a persistent lock glyph (window chrome, against §4.3); a cursor change only (easy to miss).

### D9. `height` in settings is the user's intended size
*2026-11-03 · Active*

- **What:** the field is logical px. Only a user resize stores it; DPI refits, launch placement and Reset Position leave it alone.
- **Why:** storing each display's rounded result drifted his size by about 20% after a 200% → 100% → 200% round trip.

### D10. Accepted limitation: a quote requested right after a cold start can be lost
*2026-11-03 · Active*

- **What:** if the quote window's page hasn't subscribed yet, that one request is dropped.
- **Why accepted:** it needs a click in the first moments after launch and costs one missed quote. A readiness handshake isn't worth the extra machinery.

### D11. Platforms
*2026-11-03 · Active*

- **What:**
  - Windows 10/11 is the primary target and is tested.
  - Linux (X11 and Wayland) is best effort and untested. Its expected behaviour is documented in the README.
  - macOS is not a target: transparency there would need Tauri's `macos-private-api`.

### D12. Documentation layout
*2026-11-03 · Active*

- **What:**
  - No separate architecture document, because nobody else is expected to contribute and it would drift.
  - Each invariant is enforced by a test, or by a comment where it is enforced, and indexed in `AGENTS.md`.
  - Decisions go here.
  - Verification goes in `docs/TESTING.md`.
  - The user guide and module layout stay in the README.
  - Session handoff notes aren't tracked.
- **Why:** each fact lives in one place, as close as possible to what it describes.

### D13. Git workflow
*2026-11-03 · Active*

- **What:** a branch per change, with small commits that each build and pass. The owner rebase-merges PRs, and merges, pushes and branch deletions need the owner's go-ahead.
- **Why:** commits land on `main` as written, so each one should be reviewable, bisectable and revertable on its own.
