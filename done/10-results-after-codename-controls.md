# results-after-codename-controls — codename creation stops on the wheel

After creating a codename the flow stops on the wheel; the game forwards to Controls [M 0x213288:38-40]. Port that step through the real menu path and test it with key presses from Main: create a codename, reach Controls, carry on to the mission list. Check Esc on Main at the same time: read what the game does there, against the port returning to the title and replaying the start movie, and fix it if it differs.

---
claimed-by: sandbox1 at 2026-11-05T11:06Z

---
## Report (done, 2026-11-05T12:14Z)

Closed by the driver: the report is on `s/results-after-codename-controls` (docs/reports/results-after-codename-controls.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-05T12:14Z)

# results-after-codename-controls — the codename flow forwards to Controls; Esc on Main holds

Branch `s/results-after-codename-controls` off master `f68a72216`. Two port fixes in
`crates/nf-view/src/menu.rs`, both decompile-cited, plus the key-press journey test.

## 1. The create flow forwards through the wheel to Controls [M P_CNMENU 0x213288:38-40]

The symptom: after typing a codename and confirming, the port rested on the codename wheel
(0x4000001d). The game doesn't: `P_CNMENU_Handler`'s 0x4c enter, when the wheel was opened from the
NAME entry (`param_6 == 0x40000020`) with `cheat_mode` clear, sends
`Manager_SendMessage(..., 0x44, 0x40000022, 0)` (:38-40) — Controls opens at once.

Port: in `FrontEnd::card_tick`'s completion (the page message the card flow queues [M 0x217bd8:62-71]),
entering `Page::CnMenu` now runs the wheel's forward. The gate is structural: `pending_page` is
`PAGE_CN_MENU` only from `request_create` (the profile-name save [M C_KEYBOARD 0x213a98:53-65]),
whose from-page IS the name entry and whose cheat grid cannot create — exactly the game's
`from == 0x40000020 && !cheat`. The forward stages the Controls enter's own pair (the same one the
wheel's row-1 arm runs [M P_CNCONTROLS 0x213cd0 0x4c stages `was_inverted`]) and opens
`Page::CnControls` on the style row.

## 2. Esc on Main does nothing [M P_MAIN_Handler 0x20a7b0]

The port's `(Action::Back, _)` on Main returned to `Page::Start` — which replays the start movie
(P_ESTHERO). The game: `P_MAIN_Handler` has NO back arm at all — the 0x4c enter, the 0x50's 45 s
idle-attract arm (0xa8c ticks → 0x40000035), and nothing else; the row controls
C_GONIGHTFIRE 0x2169b8 / C_GOMULTIPLAYER 0x216a78 / C_GOCODENAMES 0x216b38 handle only the 0x4b
select. ○/Esc on Main is ignored. The port now consumes the press and holds. (The menus_flow test
already flagged the old route as "[S] — the port's Main Back"; settled [M] this pass.)

## 3. The journey test — key presses from Main

`codename_entry::from_main_create_codename_reach_controls_then_the_mission_list`: boot → title →
START → Main; Esc holds (no replay); Codenames → New Codename → the grid types AGENT07 → ✓ →
"Saving data…" completes → **Controls 0x40000022** (not the wheel) → ✕ commits → the wheel → Back
out → Main → NightFire → Codenames → Play without saving → Difficulty → **the mission list
0x4000001c**. Every hop cites its handler.

## Pins re-derived (the old expectations embodied the two bugs)

- `codename_entry`: the create/overwrite completions now land on CnControls (the wheel-route test
  renamed `valid_name_creates_saves_and_forwards_to_controls`); the wheel's last-row save test Backs
  from Controls to the wheel first.
- `create_save_live`, `menu_sp_e2e` (×2 sites), `options_persist_roundtrip` (`to_edit_wheel` Backs to
  the wheel for its callers): same forward.
- `menu_sp_e2e::back_walks_every_single_player_step_to_main`: the walk ends on Main and HOLDS there.
- `menu_graph_walk`'s Back probe: Main joins the no-back pages (Start/Credits/…) — the game has no
  arm there.
- `menus_flow`: the [S] note on Main-Back replaced with the settled [M].

## Run list (release, --test-threads=1)

- `nf-game`: **105 binaries, 1077 passed / 0 failed** (the whole crate, --no-fail-fast).
- `nf-formats` + `nf-audio`: **51 binaries, 222 passed**; the only failures are the disc-image pins
  the slim ISO cannot satisfy: `disc_golden` (3) and `identity_disc::the_real_disc_passes_the_
  identity_check` (1 — the identity pins the full retail image's sector count/SHA-1; the task ships
  the trimmed onfoot image). Unrelated to this change (nf-formats does not read menu code).
- `nf-view` (the touched-code set, lesson 10 — every binary that drives `FrontEnd::action` /
  `card_tick` / the FE): codename_entry **15/0**, menus_flow **43/0**, menu_sp_e2e **11/0**,
  options_persist_roundtrip **3/0**, create_save_live **4/0**, memcard_save_flow **7/0**,
  save_progress_roundtrip **4/0**, menu_sfx_flow **19/0**, menu_music_flow **14/0**,
  settings_persist **13/0**, menus_mouse_parity **7/0**, menu_graph_walk **2/0**, menu_movies
  **4/0**, menus_backout_after_character **2/0**, menus_e2e_mp **5/0**, dossier_pages **17/0**,
  menu_audio_flows **9/0**, mp_match_end_live **2/0**.

## What's left

- The `walkreal_*_live` / `mp_*` / `net_*` nf-view binaries were not re-run: the sandbox disk
  (9.9 G) could not hold the ~100 rebuilt test binaries — the linker died twice with SIGBUS at full
  disk; after trimming, the freed space was spent on the crates above. None of them touches
  `menu.rs` (the mission-side sim and the MP net layer; the shared `tests/common/` helpers are
  untouched — lesson 7's opt-in rule never triggered), and `net_e2e_loopback_live` /
  `net_soak_faults_live` remain unrun for the same reason. The next message can start there if the
  driver wants the whole blanket re-run on this branch.

## Live checks for the main agent

1. **Esc on Main** [M→L]: PCSX2, at Main (post-intro), press ○/Esc — expect NOTHING (no iris, no
   title, no movie). The old port returned to the title and replayed 0x073B0048.
2. **The create forward** [M→L]: create a codename from Main (Codenames → New Codename → type → ✓)
   — expect "Saving data…" then CONTROLS (0x40000022), never the wheel resting; ✕ commits back to
   the wheel.
3. **The wheel's idle attract** (unchanged but adjacent): 45 s idle on Main → 0x40000035 [M
   0x20a7b0:0x50 arm] — the port models the tick; a live spot-check is cheap if the debugger is open.

## Usage

not visible to me — this sandbox exposes no token, context-window or cost figures.
