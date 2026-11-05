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
*2026-11-03 · Superseded by D15*

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
*2026-11-03 · Superseded by D17*

- **What:** a branch per change, with small commits that each build and pass. The owner rebase-merges PRs, and merges, pushes and branch deletions need the owner's go-ahead.
- **Why:** commits land on `main` as written, so each one should be reviewable, bisectable and revertable on its own.

### D14. "I stand ready." stays on screen during Stand Watch
*2026-11-04 · Active · Departs from the brief's §4.4 "No permanent text overlay" and §5.5 "optional brief message"*

- **What:**
  - While Stand Watch is on, "I stand ready." stays by him instead of fading.
  - Other lines (a quote, the lock lines) take its place for their few seconds. Once none is showing and he has held still for 0.6 s, it fades back in. Dragging or resizing hides it until he's still, and Hide hides it until Show.
  - If Stand Watch is saved as on, the line appears at launch.
  - It keeps to his layer: topmost only while he's Always on Top. Brief lines are still topmost.
  - Stand Down says "The watch is ended. Rest well." and the line goes.
- **Why:** the owner's choice: the watch should be visible while it lasts. It is still shown only because the user chose the mode.
- **Alternatives rejected:** the brief's one-off message; a line that never yields to quotes or lock lines; showing it only after toggling, not at launch; keeping it topmost when he isn't.

### D15. Quotes are set upright in IM Fell English
*2026-11-04 · Active · Supersedes D6*

- **What:** IM Fell English (regular, upright) in the same muted gold, with the same feathered shadow and no card or bubble. Its size is 5.25% of his height, between 17 and 27 px, about 5% larger than before, because its x-height is small.
- **Why:** the owner's choice from a comparison sheet of seven styles over dark, light and busy backgrounds. The old-press texture suits a weathered knight, and upright type reads steadier than the italic.
- **Alternatives rejected:** EB Garamond italic (D6), upright, upright small caps and upright in soft white; Cinzel capitals; Cormorant Garamond.

### D16. CI builds an unsigned Windows installer for every push to `main`
*2026-11-05 · Active*

- **What:**
  - After the checks pass on a push to `main`, CI builds the NSIS installer and keeps it as a workflow artifact for 14 days. Pull requests don't build one.
  - Windows only, and unsigned: a downloaded build shows the SmartScreen prompt, and the README says how to get past it.
  - Building locally stays the usual way to install: `npm run install-local`, or the hook that runs it on every pull into `main`. The artifact is for when that isn't convenient.
- **Why:** the owner wanted a ready-made installer for each merge without building it by hand. An artifact needs no release process, version bump or write permission, and old builds expire on their own.
- **Alternatives rejected:** a GitHub Release for every merge, which would be noisy at a fixed version; tagged releases, which can come later if versioned builds are needed; code signing, which costs money and isn't needed for one owner; Linux packages, which would ship untested builds (D11); an installer for every PR, which would add minutes per push; an installer for every commit in a push, which would mean several release builds per merge for commits nobody installs.

### D17. Git workflow: a PR per feature or product area
*2026-11-05 · Active · Supersedes D13*

- **What:**
  - Each large feature, separate product area or markedly different piece of work gets its own branch and PR. Smaller related changes can share one, as separate commits.
  - Commits stay small, and each one builds and passes.
  - The owner rebase-merges PRs. Merges, pushes and branch deletions need the owner's go-ahead.
- **Why:** with one owner and no other contributors, a PR for every change costs more than it gives. Separate areas still get their own PR, so each can be reviewed, merged or reverted alone. Commits land on `main` as written, so each one should be reviewable, bisectable and revertable on its own.
- **Alternatives rejected:** a branch per change (D13), which split closely related work, such as the install-on-pull hook and the CI installer, into separate PRs.
