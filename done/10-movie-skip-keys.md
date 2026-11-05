# movie-skip-keys — which inputs skip the start movie and trailer

On PC only Confirm skips the start movie and the attract trailer; the keys mapped to action 0x19 work only on state-6 movies. Find the game's skip rule for each movie kind in the decompile and make every kind accept the same inputs through the real App key path. Test each at 240 Hz with the 180-tick grace. Cite [M] addresses.

---
claimed-by: sandbox5 at 2026-11-05T08:06Z

---
## Report (done, 2026-11-05T09:07Z)

# movie-skip-keys — report

Branch `s/movie-skip-keys` (from origin/master 27c191518). WORKLOG: `docs/WORKLOG.md`,
entry `movie-skip-keys`.

**What the task was:** on PC only Confirm skipped the start movie and the attract trailer;
the keys mapped to action 0x19 worked only on state-6 movies. Find the game's skip rule per
movie kind in the decompile, make every kind accept the same inputs through the real App key
path, test each at 240 Hz with the 180-tick grace.

## The game's rule (decompile, [M])

While a front-end movie plays, `MenuManager_Update` 0x21c3d8:37-43 polls the ONE skip action —
`Menu_InputAction(-1, 0x19, 4)` (action 0x19 = R1/D-pad-down, raw pad mask 0x840, pad 0 [M
`psiInput_MapInputs` 0x10cd80:415-417]) — and calls `Menu_StopMovie(0)`. `Menu_StopMovie`
0x215a50:6 stops the movie only when the skip byte `DAT_002e1c49` is set AND the grace counter
`DAT_002e1c4c` has reached 0 (`Menu_UpdateMovieCounter` 0x215970:6-8 decrements it per manager
tick); :11 is `psiStopBackgroundMovie`. The kinds differ ONLY in their gates:

| kind | skip byte / grace | [M] |
|---|---|---|
| boot logos (P_INTRO) | c49 = 0 — unskippable, nothing skips them | 0x209e88:41,:55 |
| the START movie (P_ESTHERO) | c49 = 1, grace c4c = 0xb4 = 180 ticks (3 s) | 0x20a060:17-18 |
| the attract trailer (P_ATTRACT) | c49 = 1, c4c = 0 — skips at once | 0x20a1f0:17-19,:27-29 (P_TRAILER_Handler 0x214f10 the same shape) |
| state-6 movies (briefings/outros) | `Input_Action(-1, 0x19, 1)` read directly in case 6, no gate | GameFlow_Main 0x1c98c8:148-150 |

## The port's divergence and the fix

The port bound the 0x19 keys (ArrowDown = the mask's D-pad-down half, R = R1 [M port
`cutscene_skip_key`, lib.rs]) to the state-6 skip only (`App::app_key`'s `ST_MOVIE` arm); the
intro run (P_ESTHERO) and the attract accepted only `Action::Confirm` in `fe.action` — the
PC adapter's stand-in. Exactly the reported symptom.

The fix (one rule, two triggers):
- `menu.rs`: `FrontEnd::movie_skip_armed()` (is a skip-accepting front-end movie up?) and
  `movie_skip_poll()` (the manager's poll: the armed check + the 180-tick grace + the stop;
  the attract branch funnels into `attract_movie_finished`, the intro branch is the old
  Confirm arm's mutation). The boot logos are never armed.
- `lib.rs` `App::app_key`: the front-end skip arm now fires on `cutscene_skip_key` when the
  flow is in state 6 (skip ungated, case 6's read [M 0x1c98c8:148-150]) OR the poll is armed
  (the poll honours the gates [M 0x215a50:6]); the key is swallowed either way when a movie
  state is up (the game's msg 0x68/1 input block owns these pages [M 0x20a060:25]).
- `menu.rs` `fe.action`: the attract and intro Confirm arms route through the same
  `movie_skip_poll()` — Confirm keeps working, and the grace/skip-byte rule lives in ONE place.
- Confirm's state-6 path keeps its 1.0 s guard (the PC adapter's double-tap guard [S], a port
  convenience — the game needs none because its skip key is disjoint from its pick keys).

## Reproduce → fix (rule 4)

The App-key tests were written FIRST, through the real input path (`HeadlessApp::key` →
`App::app_key` — the exact path the window's event loop drives, no test-only plumbing):

- `flow_audit::the_start_movie_skips_on_the_action_keys_past_the_grace_at_240hz` — FAILED
  before the fix at "the skip" (the ArrowDown press past the grace did nothing), PASSES after:
  inside the grace (96 ticks < 180) both keys refuse; at 240 ticks ArrowDown skips straight to
  the main menu; a fresh boot skips with R at 204 ticks.
- `flow_audit::the_boot_logos_refuse_every_skip_at_240hz` — green before and after (the
  logos' no-skip was already honoured; pinned so it stays): ArrowDown, R and Enter all refuse
  during the first logo [M 0x209e88:41,:55].
- `flow_audit::the_briefing_skips_on_the_action_keys_at_once_at_240hz` — green before and
  after (state 6 already skipped via the keys; pinned): the briefing at 240 Hz pops on
  ArrowDown and the mission loads.
- `flow_audit::the_attract_trailer_skips_on_the_action_keys_at_240hz` — the attract's App-key
  leg (ArrowDown at once, then R on the next cycle). **Runs only where the image carries the
  PSS files**: this sandbox's slim ISO has no movie data (`movie::Player::start` declines, and
  the glue closes a declined attract at once [M BGFMVPlaying == 0]), so it self-skips here
  like the existing movie-length tests do; the poll's attract BRANCH is pinned instead by
  `flow_audit::the_skip_poll_honours_each_kind_gate` (FrontEnd-level, runs everywhere):
  the attract skips with no grace and lands back on the opener, tick 179 refuses, tick 180
  skips, the boot logos are never armed.
- The grace is game ticks, so it is display-rate independent: the tests run the App at 240 Hz
  (`Player::boot(&iso, 240)`, `frame(1/240)`), the menu ticks stay 60 Hz, 180 ticks = 3 s of
  game time at any display rate.

## Run list (each binary whole)

| binary | result |
|---|---|
| nf-view lib (all, incl. the 5 new tests + flow_audit) | **137 passed; 0 failed** |
| nf-game + nf-audio (all binaries) | **110 binaries; 1105 passed; 0 failed** |
| nf-formats lib + tests | all ok, except disc_golden **11 passed; 3 failed** — the 3 allowed slim-ISO failures (bigf_header_fields_behave_as_documented, bigf_tables_match_the_golden_manifest, directory_tree_matches_the_golden_manifest) |
| movie_playback_flow | 9/0 |
| menu_movies | 4/0 |
| sfx_rejected_start_live | 1/0 |
| reward_after_outro | 3/0 |
| menu_audio_flows | 9/0 |
| briefing_plays_live | 0/0 (all its tests gate on movie data) |
| menus_flow | 43/0 |
| menus_e2e | 5/0 |
| menu_sp_e2e | 11/0 |
| menu_graph_walk | 2/0 |
| screen_adjust | 9/0 |
| save_progress_roundtrip | 2/0 |
| options_persist_roundtrip | 3/0 |
| mission_unlock_flow | 4/0 |
| menu_music_flow | 14/0 |
| menu_sfx_flow | 19/0 |
| mp_screens | 5/0 |
| menus_e2e_mp | 5/0 |
| menus_backout_after_character | 2/0 |
| gameflow_stack_sp_run | 4/0 |
| menus_mouse_parity | 17/0 |
| movie_gop_stream_live | 0/0 (movie-data gated) |
| net_e2e_loopback_live | 2/0 |
| net_match_end_clients_live | 4/0 |
| cn_controls_live | 4/0 |
| create_save_live | 4/0 |
| walkreal_07000004_live (the only walkreal that references the touched surface — `fe.action(Extra/Back/Side)`; tests/common/ untouched) | 10/0 |

The nf-view integration set has ~60 binaries and their release links did not all fit the
session's window; the list above is every binary that calls the functions or fields this task
touched (`rg -l "skip_movie|attract_movie_finished|movie_skip|ESTHERO|attract_playing|
intro_playing" crates/nf-view/tests/`), each run whole. The walkreal suite is untouched
(no common/ change).

## What's left

Nothing for the task. Noted for the driver: on a movie-bearing image the attract's App-key
leg (`the_attract_trailer_skips_on_the_action_keys_at_240hz`) should run — it is gated on the
same `iso()` check the existing movie tests use, so a retail/NF_MOVIE_ISO sandbox picks it up
automatically.

## Live checks for the main agent

1. **Render/visual (PC parity)**: boot the port, on the title press Start and, 3.5 s into the
   rings/Earth intro, press ArrowDown then R — the movie should skip to the main menu; a press
   inside the first 3 s must do nothing. On the title, idle 45 s into the trailer and press
   ArrowDown — it should return to the title/main menu at once. The boot logos must ignore
   every key.
2. **The pad** (if a controller is at hand): the pad's R1/D-pad-down should now skip the same
   three kinds if the pad path maps through the same actions — the gamepad's menu mapping was
   NOT changed by this task (the task's surface was the App key path); check before assuming.

## Usage

not visible to me
