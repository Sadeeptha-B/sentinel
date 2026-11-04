# Testing Sentinel

## Automated checks

Run all of these before every commit. All of them must pass, and clippy must report no warnings.

CI (`.github/workflows/ci.yml`) runs the same checks on every pull request and on every push to `main`, on Windows and on Linux, with clippy at `-D warnings`. `main` is protected, so both jobs must pass before merging. The Rust version is pinned in `rust-toolchain.toml` so that local builds and CI agree.

```sh
npm test                                     # Vitest: quote picker, wheel steps
npx tsc --noEmit -p .
npm run build                                # production frontend build
cargo test --manifest-path src-tauri/Cargo.toml       # geometry, scaling, hit mask, settings, saving
cargo clippy --all-targets --manifest-path src-tauri/Cargo.toml
```

- **Rust on `PATH`.** The toolchain lives in `%USERPROFILE%\.cargo\bin`, and some shells don't have it on `PATH`. Prepend it: `export PATH="$HOME/.cargo/bin:$PATH"`.
- **Keep Rust tests free of Tauri.** A Rust test that touches Tauri's windowing (for example by constructing `AppState`) crashes on Windows with `STATUS_ENTRYPOINT_NOT_FOUND`, because the test executable lacks the common-controls manifest. Put testable logic in Tauri-free code (`knight.rs`, `settings.rs`) and test it there.

## Live checks on Windows

Unit tests can't see transparency, DPI changes, click-through or the tray, so changes in those areas need a live run with `npm start`. That's a development build with its own identity, so it runs alongside an installed knight without touching its settings (README → *Development build and installed app*). The summon shortcut goes to whichever knight started first, so to check `Ctrl+Alt+K` on the dev build, exit the installed knight before running `npm start`.

**Ask the owner before driving their mouse or keyboard, and put the cursor back afterwards.**

### Passed (Windows 11, two monitors)

| Area | Expected |
|---|---|
| Rendering | Crisp at 3×, 4× and 5×. No visible rectangle. |
| Quote | Shown over his lower half when he's low on screen, below his feet when there's room. Hides by itself. |
| Drag | Moves him, doesn't trigger a quote, saves the position. |
| Resize | Ctrl+wheel and Ctrl+= / Ctrl+- step one `k` at a time. |
| Menu | Size check marks track `k`. Large and Small work. |
| Click-through | His helm, chest and blade catch clicks. The corners and the shadow pass them through. |
| Stand Watch / Stand Down | The messages appear. |
| Hide / Passive | `Ctrl+Alt+K` recovers both. |
| Tray | Left-click toggles visibility. The menu is correct. Reset Position puts him bottom-right. |
| Exit and relaunch | Position and size are restored. |
| Second launch | Summons the running instance, with no duplicate. |
| Idle CPU | 0.00% |
| Stand Watch line (D14) | "I stand ready." stays. A quote replaces it, then it fades back in. A drag or Ctrl+wheel resize hides it, and it returns at his new spot within a second of him settling. Hide hides it, and a second launch brings it back. It's there at launch when Stand Watch was saved on. With Always on Top off it sits directly above him and goes behind other windows with him; turning it back on makes it topmost again. Stand Down shows the farewell line, which fades for good. |
| Stand Watch line after a right-click | At the default spot, with the line over his lower half, right-clicking him and pressing Esc leaves the line's window directly above his, with Always on Top on and off. With it off, neither window is topmost, and an ordinary window opened over him covers both. |
| Dev build alongside the installed knight | Both run. The dev knight has its own settings and its tray tooltip says "Sentinel (dev)". |

### Outstanding

- **Lock:**
  - Locking says "I hold this ground." and unlocking says "I am free to move."
  - The Size submenu is greyed out while locked.
  - A drag, Ctrl+wheel or Ctrl+= while locked says the hold line once and doesn't move or resize him.
- **Feet stay put:** resizing step by step keeps his boots on the same screen row.
- **Click-through after a Windows lock:** lock with Win+L, unlock, and check that his transparent pixels pass clicks through again.
- **Invisible-corner recovery:** drag him so that only an empty corner of his window is on screen, then press `Ctrl+Alt+K`. He should come back.
- **Moving between monitors with different scaling:**
  1. Set one monitor to 125%.
  2. Drag him across and back.
  3. Confirm the window is `58k × 96k` on each side, with `k` switching between 4 (100%) and 5 (125%) at the default size.
  4. Confirm his size is the same after the round trip.
- **Installers:** run `npm run tauri build`, install, and check the first-launch default (bottom-right, 4× at 100%). Also check that the installer runs without a SmartScreen prompt, that `sentinel.exe` runs without installing, that reinstalling keeps settings, and that uninstalling keeps them unless the app-data box is ticked.
- **Stand Watch line, idle CPU:** measure it in a release build with the line showing. The debug build read 0.05–0.55% of one core both with and without the line, which is noise at that level.
- **Stand Watch line, old spot:** after a drag, check frame by frame that the line never flashes where he was. The sampled checks couldn't see a single frame.
- **Linux:** X11 and Wayland (GNOME and KDE). CI only shows that it builds and passes the unit tests there; nobody has run the app. Record the actual behaviour in the README's platform notes.

### How to run live checks

These were done from PowerShell scripts that made the process per-monitor DPI-aware (`SetProcessDpiAwarenessContext(-4)`).

- **Finding his window:** enumerate the windows titled `Sentinel`. The knight is the one whose size divides by 58 × 96.
- **Inspecting state:**
  - `GetWindowRect` and `GetDpiForWindow` give geometry and DPI.
  - The `WS_EX_TRANSPARENT` bit of the extended style shows whether click-through is on.
- **Driving input:** `SetCursorPos`, `mouse_event` and `keybd_event`.
- **Menus:** navigate by keyboard (arrow keys and Enter; separators are skipped).
- **Tray:** UI Automation finds the `Sentinel…` button. It lives in the hidden-icons overflow, which closes after a click.

**Memory baseline:** about 130–210 MB private across the WebView2 processes, depending on GPU-process activity. This is WebView2, not app code.
