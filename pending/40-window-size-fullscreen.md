## Your task: `window-size-fullscreen`, the PC window can be resized, set to a size, and made fullscreen

Branch from origin/master. The PC build opens winit's default window (lib.rs resumed: Window::default_attributes().with_title) and has no size or fullscreen option (the settings-persist note in lib.rs says so). Add, port-only: (1) borderless fullscreen on the current monitor, toggled by Alt+Enter and by a row on the Options > Screen adjust page (or a port-only Display row beside it - match the existing page frame helpers: page_header, panel, wheel, desc_block, hint_bar, logo_stamp); (2) window size presets (800x600, 1280x960, 1600x1200, 1920x1440 when they fit the monitor) on the same page; (3) the frame keeps the game's 4:3 picture, pillarboxed/letterboxed, when the window is any other shape (check the existing present path first - it may already scale); (4) the choice persists across launches in a port-only file next to the existing settings file (never in the game's option file layout). Unit-test the persistence round trip and the aspect fit maths; headless --shot at two sizes into the report. Do not change any game-page layout. Run nf-view --lib and menus_e2e, menus_mouse_parity.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/40-window-size-fullscreen`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
## Already on master (631871863, main agent)
Alt+Enter borderless fullscreen + display.toml persistence (settings::display_path / display_fullscreen_at / save_display_fullscreen_at; the window is created fullscreen when the file says so), and the 4:3 picture is already pillarboxed at any window shape [render audits: 2560x1440]. What remains of this task: the size presets and a fullscreen row on the Screen adjust page (or a port-only Display row) using the same display.toml - extend that file, do not add a second one.

Note (main agent, 11-12): the stub `Page::Options` is unreachable (nothing calls enter_page(Page::Options) - menus_mouse_parity.rs notes it); the reachable options page is the GAME's AV options (P_CNAVOPTIONS 0x214818). Do not add rows to a game page: if a menu entry is wanted, it must be a port-only page reached without changing a game page's rows, otherwise leave the display choice on Alt+Enter (already on master).
