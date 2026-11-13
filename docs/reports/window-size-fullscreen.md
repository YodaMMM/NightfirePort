# window-size-fullscreen — report

Branch `s/window-size-fullscreen` (head 9559c1800 = origin/master 2265b403e + my two commits, merged
cleanly). Everything below is port-only surface — the PS2 had no PC window, so no assertion here
cites game behaviour; the port choices are marked [S] and pinned by the new tests as port design.

## What was already on master (kept, not rebuilt)

- Alt+Enter borderless fullscreen + `display.toml` persistence (`settings::display_path`,
  `display_fullscreen_at`, `save_display_fullscreen_at`, the window created fullscreen when the file
  says so), the 4:3 picture pillarboxed at any window shape, and — beyond what the task note listed —
  `settings::Display { fullscreen, size }`, `WINDOW_PRESETS` (640x480 … 1920x1440), the
  `next_window_preset` fit maths with tests, and the Alt+PageUp/Alt+PageDown preset stepping
  (settings.rs, lib.rs). The aspect-fit maths is `cutscene_frame` (lib.rs) with its own live tests
  (`fix_cutscene_letterbox_live.rs`, `cutscene_side_strips_live.rs`). I did not touch any of it.

## What this task added

1. **The port-only Display page** (`Page::Display`, `PAGE_DISPLAY 0x5000_0006`, next to the online
   pages' spare 0x5000_0000 block; menu.rs). Per the main agent's note the reachable options page is
   the game's own P_CNAVOPTIONS 0x214818 and no row may be added to a game page, so the page is
   reached only through the port's own opener — **Alt+D** at the host (lib.rs `window_event`, same
   layer as Alt+Enter; D is swallowed while Alt is held, it is the menu's ▶ key), never from a game
   page's rows. `FrontEnd::open_display(state, monitor, from)` refuses when input is blocked (a
   notice, a movie) or the page is already up. Back returns to the page it opened from
   (`display_from`).
2. **Rows**: a Fullscreen Off/On toggle, the 4:3 size presets THAT FIT the synced monitor size
   (`display_presets()` — on a 1920x1080 monitor only 640x480 / 800x600 / 1280x960 are offered;
   1600x1200 and 1920x1440 fold away; an unknown monitor offers all), and Back. Confirm on a size
   row means windowed at that size (fullscreen drops, [S]); the pick rides
   `FrontEnd::display_request` and the host drains it in `menu_after` (the settings_save_pending
   pattern): apply to the winit window (`set_fullscreen` / `request_inner_size`) and write the SAME
   `display.toml` the Alt+Enter / Alt+PageUp keys write — no second file, no new format keys.
3. **Draw + input** follow the shipped port-page template (the MpMode arm): page_header / panel /
   wheel / desc_block / hint_bar / logo_stamp, so it reads as one of the game's own pages; the mouse
   walks and clicks the rows like the keyboard (`list_rows_y` bands 138+26k, every row an action).
   `Page::id()` / `Page::from_id` know the new id; no game page's rows, constants or match arms were
   renamed or repurposed (PAGE_MP_ONLINE_LOBBY stayed 0x5000_0005; an early accidental bump of it
   was caught and reverted before commit).
4. **Render aids** for the report: `--page display` boots the page headless (NF_SHOT_DISPLAY_MON /
   NF_SHOT_DISPLAY_SIZE / NF_SHOT_DISPLAY_FS override the shown state) and `--size WxH` picks the
   headless shot's render size (`shot_size`, default stays 1280x720; the F12 spot shot keeps it).

## Verification (what ran, on the final merged tree)

- `cargo test --offline --release -j2 -p nf-view --lib` — **141 passed** (includes settings.rs's
  persistence round trips: `the_window_size_round_trips_and_survives_the_fullscreen_toggle`,
  `the_size_presets_step_within_the_monitor`, `the_fullscreen_choice_round_trips_through_display_toml`).
- `--test display_page` (new, 8 tests) — **8 passed**: the fit list per monitor (1920x1080 vs
  2560x1440 vs unknown), row labels and clamping, Confirm→request round trips (toggle keeps size,
  preset drops fullscreen), Back to the opener, mouse walk+click parity, blocked opens, the page id.
- `--test menus_e2e` — **5 passed**; `--test menus_mouse_parity` — **18 passed**; `--test menus_flow`
  — **43 passed**; `--test page_titles` — **3 passed**.
- Headless shots, both re-taken on the final merged tree (lavapipe, `NF_CONFIG_DIR` pointed at a
  throwaway dir so no real settings were touched):
  - `docs/reports/window-size-fullscreen-800x600.png` — 4:3, the page in the game's frame
    (header, panel, wheel on "Fullscreen Off", desc block, hint bar, logo stamp).
  - `docs/reports/window-size-fullscreen-1920x1080.png` — 16:9, the same page keeping the 4:3
    shape with pillarboxed black at the sides (the existing present-path behaviour, confirmed on
    the port's own page too).
- The full suite (`tools/test/suite.py`) did **not** run: the sandbox reset had wiped the Rust
  toolchain and the game files, so setup (rustup, work-slim, the 1.06 GB ISO, the ALSA prefix,
  `cargo fetch`, the lavapipe userspace) consumed the first ~35 of ~120 available minutes on a
  2-core box. The named test binaries above are what fit; the whole-workspace suite run is the
  follow-up.

## What's left

- The full `tools/test/suite.py` pass on this branch (my changes touch nf-view only; every nf-view
  binary above is green on the merged tree, and no shared test helper changed).
- A real-window pass on a desktop (render audit, lesson 6): my sandbox has no GPU/display — the
  window-side apply (`set_fullscreen` / `request_inner_size`) is exercised by winit only on a real
  machine.

## Live checks for the main agent

Nothing here can be measured against the PS2 — all [S]. If you render on a desktop:

1. Alt+D on the main menu opens the Display page; Esc/Back returns to the main menu with the
   Multiplayer row's cursor unchanged.
2. Confirm on "Fullscreen On" goes borderless immediately; after a restart the window boots
   fullscreen (the display.toml row `fullscreen = true`, written by the same drain).
3. Confirm on "1280 x 960" on a 1920x1080 monitor resizes the window and drops fullscreen if it was
   on; the file's `width`/`height` follow; 1600 x 1200 / 1920 x 1440 must NOT be offered on 1080
   lines.
4. Alt+Enter and Alt+PageUp/Alt+PageDown behave exactly as before (the page writes the same file,
   so the two surfaces must agree after mixing them).

## Usage

- Input tokens / output tokens / context used / cost: not visible to me (the sandbox shows no
  per-session token or cost figures).
