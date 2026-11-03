# Working on Sentinel

Sentinel is a pixel-art knight in a frameless, transparent, always-on-top window (Tauri 2 + React + TypeScript). It runs offline, with no backend or telemetry. There is one owner and no other contributors. This file is the guidance for coding agents. Follow it, and keep it current.

## Read first

| File | Use it for |
|---|---|
| [`docs/sentinel.md`](docs/sentinel.md) | The product brief: what Sentinel is and isn't. |
| [`docs/DECISIONS.md`](docs/DECISIONS.md) | Choices already made. Don't reopen them without the owner. |
| [`docs/TESTING.md`](docs/TESTING.md) | Automated checks, live checks and how to run them. |
| [`README.md`](README.md) | User guide, build commands, module layout, platform notes. |

## Product guardrails

- **The brief's out-of-scope list (§9) is binding.** No animation, extra characters or skins, customisation, timers, productivity features, chat or AI content, accounts or sync.
- **Stillness is the design.** Nothing moves or appears unless the user acts.
  - Quotes appear only on request, never on a timer.
  - No idle animation frames.
  - No notifications.
- **Only the figure is visible.** No window chrome, badges, cards or permanent UI. Feedback comes in his voice, as a whisper (see D8).
- **Keep idle cost near zero.** Measured idle CPU is 0.00%. Don't add polling, timers or work in the UI.
- **Do less.** When a request could grow the feature set, propose the smallest version, and say where it touches the brief.

## Decisions

- Before changing behaviour, check [`docs/DECISIONS.md`](docs/DECISIONS.md). If a change would contradict an active entry, stop and ask the owner. Don't work around it quietly.
- Ask the owner, rather than choosing yourself, when a change affects any of these:
  - user-visible behaviour or product direction;
  - persisted settings;
  - a new dependency;
  - platform support.
- When the owner decides something, record it as a new entry: **what**, **why**, the **alternatives** rejected, and what it **supersedes**.
  - Entries already on `main` are history. Don't rewrite them; mark them `Superseded by D<n>`.
  - An entry still on your unmerged branch is a draft. Edit it in place as the decision evolves, and list options the owner rejected along the way as alternatives.
- Record only real choices, ones that could reasonably have gone another way. Correctness rules belong in the invariants below.

## Invariants

Breaking one of these causes a regression. Each is enforced by a test, or documented at the code that enforces it. Read that comment before changing the code around it.

| # | Invariant | Enforced or documented at |
|---|---|---|
| 1 | **Never hold an `AppState` mutex across a window, menu or tray call.** Off the main thread these calls block on it, so a held lock can deadlock. | `AppState` doc in `lib.rs`; `stage.rs` module doc; `tray::sync` |
| 2 | **Click-through state changes only on the main thread.** The cache and the native call change together there. | `stage::set_ignore` |
| 3 | **The knight window is always an exact sprite multiple,** `window_px(k)` = `58k × 96k`. Snap through `scale_for`. | `knight.rs` tests (`scale_round_trips…`, `a_dpi_rescaled_window…`) |
| 4 | **Don't refit in the `ScaleFactorChanged` handler.** Windows resizes the window after that event and overwrites a refit. The ticker re-snaps once the size has held for 150 ms. | `stage::on_window_event`; `ticker.rs` (`SNAP_AFTER`) |
| 5 | **On Windows, read DPI and geometry from the OS** (`stage::dpi`, `stage::current_rect`), not Tauri's caches, which went stale in testing. | doc comments on both functions |
| 6 | **Resizes keep his feet in place**: the line under his boots, not the window bottom. Apply them in one `SetWindowPos`. | `knight::anchored_resize` and its test; `stage::apply_rect` |
| 7 | **"On screen" means enough of the figure is visible,** not just enough of the window rect. | `knight::is_reachable`; `transparent_corner_on_screen_is_not_reachable` |
| 8 | **Assets are one contract.** `knight.png`, `knight.hitmask.bin` and `knight.meta.json` are regenerated together by the pipeline, never edited by hand. No test compares the PNG with the mask; `mask_matches_the_sprite` only checks that the mask's size agrees with the meta. | comment at `knight::META` |
| 9 | **Settings stay backward compatible.** Fields use serde defaults, unreadable files fall back to defaults, and fields aren't renamed or repurposed without a migration. `height` is the intended logical size, stored only on a user resize. | `Settings` doc; `round_trips_and_tolerates_garbage` |
| 10 | **Saves are serialised and retried.** One save at a time; a failed save stays dirty. | `settings::Saver`; `a_failed_save_is_retried` |
| 11 | **The recovery path always works.** `Ctrl+Alt+K` (and a second launch) undoes Hide, Passive and an off-screen position in one step. The tray menu offers each separately: Show, Passive and Reset Position. Tray left-click only toggles visibility (showing also brings him back on screen). | `stage::summon` doc; `register_shortcut` |
| 12 | **Lock blocks every user move and resize.** Rust refuses resizes from any path. Dragging can only be blocked in the page, because the page starts the native drag itself (`startDragging`), so keep that guard. | `stage::resize_to`; `onPointerMove` in `Knight.tsx` |
| 13 | **`src/ipc.ts` matches the commands and events in `lib.rs`.** | header comment in `ipc.ts` |

**When you add an invariant:**
1. Enforce it with a test if it can be tested. Otherwise write a comment at the code that enforces it, saying what breaks if it's violated.
2. Add a row here.
3. Never weaken or delete a test just to get a change through.

## Code conventions

- **Responsibilities:**
  - Rust owns native behaviour, state and persistence.
  - React owns presentation and input. It forwards intent to Rust rather than enforcing rules, with one exception: the Lock check on dragging (invariant 12).
  - Pure, testable logic goes in Tauri-free modules: `knight.rs` for geometry, `settings.rs` for persistence. See `docs/TESTING.md` for why Tauri can't be in unit tests.
- **Comments say why,** and what would break. Don't narrate what the code does. The `ScaleFactorChanged` and locking comments are the standard to follow.
- **Keep it small.** Avoid new abstractions, configuration or dependencies unless the change needs them. A new dependency needs the owner's OK.
- **Match the house style:** Rust 2024 edition, clippy clean, TypeScript strict, React function components.
- **Files edited by script must stay UTF-8 with their CRLF line endings.** Several contain en dashes and box-drawing characters. On Windows, Python defaults to cp1252, so always pass `encoding="utf-8"`.

## The art

- `tools/asset-pipeline/pixel_knight.py` is the only source of the knight. Change the art there and regenerate (README → *The knight*). After `npx tauri icon`, delete `src-tauri/icons/android` and `src-tauri/icons/ios`.
- The `.venv/` and `out/` folders are git-ignored. Recreate them from `requirements.txt` if they're missing.
- `extract.py`, `probe.py`, `preview.py`, `styles.py` and `reference/` are reference tooling only. They never write the app's assets.

## Verifying changes

- **Before every commit:** run every check in `docs/TESTING.md` → *Automated checks*.
- **CI:** the same checks run on Windows and Linux for every PR, and must pass before merging. Don't weaken or skip a CI step to get a change through. Code under `cfg(windows)` has a Linux fallback, so check that the Linux job passes too.
- **Rust version:** pinned in `rust-toolchain.toml`. Bump it in its own commit, together with any clippy fixes the new version needs.
- **Live checks:** a change to windows, DPI, click-through, the tray or menus also needs one. Ask the owner before driving their mouse or keyboard.
- **Reporting:** say plainly what was verified and what wasn't. Add anything unverified to *Outstanding* in `docs/TESTING.md`.

## Git

- **Branches:** branch from an up-to-date `main`, one branch per change.
- **Commits:**
  - Small, and each one builds and passes.
  - Messages say what changed and why.
  - Commit messages end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- **Ask first:** don't push, open or merge PRs, or delete branches without the owner's go-ahead. When a force push is approved, use `--force-with-lease`.
- **Merges:** the owner **rebase-merges**, so commit IDs change on merge. To rebase a branch stacked on a merged one: `git rebase --onto origin/main <old-base-commit> <branch>`.

## Keeping docs current

| When you change… | Update |
|---|---|
| User-visible behaviour | `README.md` |
| An owner decision | `docs/DECISIONS.md` (new entry) |
| A correctness rule | the test or enforcing comment, and the table above |
| Checks, or what has been verified live | `docs/TESTING.md` |
| Module layout | `README.md` → *Layout* |

Don't commit session state ("PR #2 is open", "next I'll…"): it goes stale at once. That belongs in PRs and issues.
