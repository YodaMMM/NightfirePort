# WORKLOG

## movie-skip-keys (started 2026-11-05 08:12 UTC, sandbox5)

Task: on PC only Confirm skips the start movie and the attract trailer; the keys mapped to
action 0x19 work only on state-6 movies. Find the game's skip rule for each movie kind in the
decompile and make every kind accept the same inputs through the real App key path. Test each
at 240 Hz with the 180-tick grace. Cite [M] addresses.

- Branch cut from origin/master (27c191518) and pushed.
- Plan: 1) pin the game's per-kind skip rule from the decompile (frontend/App state machine,
  movie players); 2) read the port's App key path (keys → actions → per-kind movie skip);
  3) failing test through the real App key path; 4) fix; 5) 240 Hz tests with the 180-tick
  grace for every kind; 6) full suite; report.

### movie-skip-keys — the findings and the fix (2026-11-05 08:12–09:06 UTC)

**The game's rule — ONE skip action per kind, only the gates differ [M decompile]:**
- While a front-end movie plays, `MenuManager_Update` 0x21c3d8:37-43 reads
  `Menu_InputAction(-1, 0x19, 4)` (the skip action, R1/D-pad-down, pad mask 0x840 [M
  psiInput_MapInputs 0x10cd80:415-417]) and calls `Menu_StopMovie(0)`.
- `Menu_StopMovie` 0x215a50:6 stops the movie only when the skip byte `DAT_002e1c49` is set
  AND the grace `DAT_002e1c4c` has reached 0 (`Menu_UpdateMovieCounter` 0x215970:6-8 ticks it
  down per manager tick); :11 `psiStopBackgroundMovie`.
- Per kind: boot logos c49 = 0 — unskippable [M P_INTRO_Handler 0x209e88:41,:55]; the START
  movie c49 = 1 with grace c4c = 0xb4 = 180 ticks [M P_ESTHERO_Handler 0x20a060:17-18]; the
  attract trailer c49 = 1, c4c = 0 — no grace [M P_ATTRACT_Handler 0x20a1f0:17-19,:27-29;
  P_TRAILER_Handler 0x214f10 the same]; state 6 reads `Input_Action(-1, 0x19, 1)` directly in
  GameFlow_Main case 6 with no gate [M 0x1c98c8:148-150].

**The port's divergence:** the 0x19 keys (ArrowDown/R, `cutscene_skip_key`) skipped only
ST_MOVIE (state 6) in `App::app_key`; the intro run and the attract accepted only Confirm
(the PC adapter's stand-in) in `fe.action`.

**The fix:** `FrontEnd::movie_skip_armed`/`movie_skip_poll` (menu.rs) — the manager's ONE
poll with the per-kind gates; `App::app_key`'s front-end arm now skips state 6 ungated (case
6) and runs the poll for the front-end kinds; `fe.action`'s attract/intro Confirm arms ride
the same poll (one rule, two triggers). Confirm keeps its 1.0 s state-6 guard (the PC
adapter's double-tap guard [S]).

**Verification:** the failing App-key tests first (the start movie refused the keys past the
grace — reproduced at flow_audit.rs:691), then green after the fix; nf-view lib 137/0; the
touched-surface integration binaries each whole (movie_playback_flow 9/0, menus_flow 43/0,
menu_music_flow 14/0, menu_sfx_flow 19/0, net_e2e_loopback_live 2/0, walkreal_07000004_live
10/0, … — the full list in the report); nf-game+nf-audio 1105/0; nf-formats all green except
the 3 allowed slim-ISO disc_golden failures.
