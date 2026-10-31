# level-05-interactables-2 — every player-touchable kind behaves like the game

Branch `s/level-05-interactables-2` (from origin/master e4550f2e, which did not move during the
session). Task window 2026-10-31 ~18:02-19:40 UTC.

## What's done

**The census re-run.** Re-dumped all 47 trigger placements of 07000005 (`nf-extract triggers
Nightfire-onfoot.iso 07000005.bin`) over the §7 kind table (docs/levels/07000005.md) and walked the
kind × coverage matrix against `interact_07000005_live.rs` (11 tests, the prior task),
`interactables_07000005_live.rs` (3 tests), the walk/walkreal suites and the fix_* files. Doors,
plain/lock-gated plates, the pair→mux→OR alarm chain, pickups (kinds 1/3/6), the ladder, the wire
ride, the grapple (aim + `gadget_grapple_live`'s reel), the wall-hug crawl (`walkreal`'s SS_CREEP
legs + `fix_wallhug_prompt_live`/`fix_wallhug_camera_live`), the copter, the searchlights, the
hurt volume, the spawner, the sound triggers, the exits and the objective reveal were already
covered. Three player-touchable chains were not pinned by any live test:

1. **The plunge walk-off death chain.** The 13 PlungeTrigger TouchOnces (#285-#297, out 0xBE=190,
   mask 8 [D audit rows; the resolved boxes — the basin walk crosses #287's x −20.28..59.71,
   y −2.13..10.18, z 50.22..144.68]) fire ch 190, ScriptPlayer #284 (script 0x06000952, watch 190
   [D SP row]) starts the death NIS, and its stream-8 event 0xD [98,1] writes ch 0x62 = mission
   fail [D the script dump; M 0x1827d8's ch-0x62 arm]. The valley test only asserted "health ≤ 0 OR
   ch 0x62" — the chain itself was never pinned on 05.
2. **The wall-hug window ledge run.** The TouchOnces #311/#312/#313 (out 127/128/129, lock 164)
   and #314 (out 133, lock 111 = 0x6F) [D audit rows; the boxes strung z −28.0/−31.45/−35.0/−37.4
   at y 36.73..37.75] over AND mux #80 (out 105 ← {127,128,129,133} [D row]) whose consumers are
   SoundTrigger #81 (ch 105 → sample 332 [D row]) and the alarm sound #83's STOP channel (p4 = 0x69
   [D row]). Nothing pinned the four-plate run, the AND, or the lock.
3. **The blocked ch-0x7D cut-scene Touch.** Touch #70 (105.51, 26.16, 108.37, out 125 = 0x7D, mask
   8 [D row]) is dead on this level — `Trigger_Update` 0x1b1a78:80-82 gates the ch-0x7D write on
   GameState+0x12 == 0x7001500 [M; the port models it None, triggers.rs:382] — and SP #71's script
   0x06000074 is ABSENT from the archive [D the kind-7 list]. Nothing asserted the nothing.

**Three new live tests** in `crates/nf-view/tests/interactables_07000005_live.rs` (now 6), each
driving the real input path (the walk, the drop-in crossing) with a fresh-load sister:

- `the_valley_walkoff_runs_the_plunge_death_nis_chain` — the proven valley-floor stand (63.5, 6.5,
  57.9), the real walk west down the basin: ch 190 fires, control is taken (the NIS's event-4
  disable), SP #284 plays 0x06000952, ch 0x62 lands, and health stays 100 (the kill is the NIS, not
  damage — the who-1 hurt planes are inert [M 0x17e5d0:51-104]). Sister: fresh load keeps 190/0x62
  dark, Bond alive and in control.
- `the_quiet_window_ledge_run_fires_all_four_plates_and_the_dead_and_stays_dead` — the positive:
  with no alarm raised (no approach walk — the west-wall guards wait on ch 0x22 [D p3 = 0x22]), the
  east-side crossings (x −34.7) fire #311→#314 in order; the TouchOnce outs latch [M 0x1b1d78's
  case 1 — no glist 0x10], AND mux #80 completes ch 105, and sound 332 plays. The negative: the
  pair route raises the alarm (the pair test's proven walk), `Trigger_Calc` 0x1b1900:21-23 latches
  switch_channels[0x6F] = 1 while ch 0xA4 is on [M — the decompile's own level latch, verified in
  work/decomp/ACTION.ELF/001b1900_Trigger_Calc__Fv.c:21-23], and a fresh #314 crossing is refused
  by `Trigger_Locked` 0x1b2fc8:24-29 [M] — the ledge run is the game's one-shot. Sisters: a fresh
  load keeps 127/128/129/133/105/0x6F and sound 332 dark.
- `the_blocked_ch7d_cutscene_touch_writes_nothing` — the real truck-park walk across #70's spot
  writes ch 125 never, per tick, while the toucher pass demonstrably runs (#36's ch 0x45 reveal is
  on [D row; L ee_start_05 0x26fc90]); the built-but-scriptless SP #71 never plays [D the kind-7
  list]. Sister: fresh load reads 125 off.

**The census's window-ledge open item, re-measured and resolved** (docs/levels/07000005.md updated):
the port's ledge floors along the plate line are the disc's own 36.5-37.3 surfaces (the west-lane
merges raised them; the census's "walked surface y ≈ 35.3" predates them). The death-NIS catch the
census reported survives only on the WEST approach: a walk that slides off the icon ledge's west
face dips Bond to y 35.4-35.5, inside plunge shell #291's top quad (35.47) — the shells' job. The
east-side crossings settle grounded at 36.98-37.29 and fire the plates with no shell [L probe05i
drops]. The plates' own boxes (y 36.73..37.75 [D]) are the route's evidence. Also corrected there:
**ch 111 has a writer after all** — `Trigger_Calc`'s engine latch, not a placement (the audit's
"no writer of 111" was a placements-only read).

**No port code changed.** All three chains already behave like the game; the tests pin them. The
one port-adjacent finding (the west-slide shell catch) is the shells working as designed over a
walk route the game itself walls off with the hug — documented, with the east-side route as the
pinned one.

## How it was verified

- Every game claim in the new tests carries its tag: [M] the decompile function + address
  (0x1b2b88, 0x1b2fc8:24-29, 0x1b1900:21-23 — read in the decompile file itself, 0x1b1a78:80-82,
  0x1dbda8, 0x1827d8, 0x17e5d0:51-104), [D] the disc parse (the audit rows, the nf-extract trigger
  boxes, the script dump), [L] the EE-snapshot reads the prior tasks recorded. No assertion rests
  on a guess; the two [S] items (the game's exact non-hug slide behaviour on the west face; the
  audibility of the sounds) are kept out of pass/fail and listed below.
- The routes were measured first with a scratch probe (crates/nf-view/examples/probe05i.rs,
  uncommitted): the per-station drops, the west-slide shell catch, the east-side crossings, the
  full quiet run (127→128→129→133→105→332 with 164/0x6F dark), the alarm pair end-state (84/85/
  161/164/0x6F all on), and the valley chain's tick-by-tick (190 at t≈800 → disabled → 0x62 at
  t≈889, hp 100).

## What ran (totals)

- `cargo test --offline --release -p nf-view --test interactables_07000005_live -- --test-threads=1`
  — **6 passed; 0 failed** (3 prior + 3 new).
- `cargo test --offline --release -p nf-view --test walkreal_07000005_live -- --test-threads=1`
  — **11 passed; 0 failed**.
- `cargo test --offline --release -p nf-game` — **953 passed; 0 failed** (7 ignored).
- The fast suite (`tools/test/suite.py`) on the final tree (= origin/master + this branch; master
  did not move): the sandbox kills background processes between tool calls, so the run went through
  a resume-chunk driver (per-binary logs, `--only` batches, sequential). **155 of 995 test binaries
  completed: 1,377 passed, 0 failed, 71 ignored, no FAILED/TIMEOUT/CRASH lines.** The completed set
  includes every 05-family binary (census_07000005, interact_07000005_live,
  interactables_07000005_live, walk_07000005_live, walkreal_07000005_live, copter_05_flight_live,
  sp_disabled_byte_05_live, stuck_guard_combat_05_live, g052_security_room_live) and the nf-game /
  nf-formats / nf-audio / nf-view library suites — the full regression surface of this diff. The
  remaining ~840 binaries (other levels' walkreal/fix_* suites) did not fit the session window
  after the killed background runs; they are master's own surface, untouched by this diff (test
  file + docs only).

## What's left

- The full-suite sweep of the remaining ~840 binaries (above).
- [S] unverified: whether the GAME's non-hug walk off the ledge's west face slides exactly like
  the port's walker (the port's slide dips into shell #291 and dies; the game's route hugs).
  A live PCSX2 walk-off would settle it; the port's shells themselves are pinned.
- [S] unverified: the audibility/visuals of sound 332 and the plunge NIS (the render/audio layer;
  the port's boards are pinned).
- `walk_07000005_live.rs` still carries the stale comment "0x45's setter is the level's script
  stream [S]" and a belt-and-braces manual ch-0x45 poke — §2 of the level doc already corrects
  this (#36 IS the writer [D]; the port fires it on frame 1), and the poke is harmless; the file
  is another task's, left untouched to avoid merge churn. No assertion contradicts the decompile.

## Live checks for the main agent

1. The window-ledge route: fresh boot, teleport/walk to the keep's west window ledge (−34.7, ~37,
   −28.0) and walk south along x −34.7 to z −38.7. Expect: no death NIS, and the four plates'
   boxes (y 36.73..37.75, z −28.5..−37.9 [D]) crossed; a screenshot at (−34.7, 37.1, −31.45) and
   (−34.7, 37.2, −37.4) pins the walked surface the port settles on (36.98-37.29 [L probe]).
2. The plunge death NIS: walk west off the valley floor east stand (63.5, 6.5, 57.9) into the
   basin; expect the NIS camera (0x06000952 s0, 18 keys) and the mission-fail banner on ch 0x62.
3. The quiet-run reward: after the four-plate run, sound 332 (SoundTrigger #81) should be audible
   (2D global), and the alarm siren (329) silent (its stop channel never trips).

## Usage

not visible to me — this environment exposes no token, cost or context-window figures for the
session (no usage panel or CLI counter is available to read; nothing to copy).
