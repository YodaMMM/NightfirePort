# level-02-skip-handback — the cut-scene SKIP hand-back

Branch `s/level-02-skip-handback` (from `origin/master` 8a966fa3). Task: where the game puts Bond when a
cut-scene is SKIPPED, for every SP with a kind-1/kind-2 hand-back, tested on 07000002/03/14 through the real
skip input.

## Result in one line

**No code change was needed: the port's skip hand-back already is the game's** — the task's premise
("lands at the anim's END pose") turned out to be a misreading of a correct number. The point
(-27.30, 1.23, 28.90) that a skip of 07000002's intro lands on is **the anim object's matrix FROZEN at the
fade-complete tick** (frame ~50 of the still-running intro anim), exactly what the game's decompiled skip
path produces — not the anim's end pose (which the tests prove is > 0.05 away), and not a fixed point at all:
the skip hand-back depends on when the player pressed.

## The game's skip path, read from the decompile ([M] throughout)

- The skip input: `SP_Update` state 2 reads `Input_Action(-1, 0x19, 1)` every tick and calls
  `Script_FFwd(si, 1)` unless the script is a death NIS [M 0x1dbda8:129-141, `Script_IsDeathNIS` 0x1dae60].
- `Script_FFwd` (0x1dac78) arms only while a script camera is running (`+0xAB5` bit 4 — set by
  `Script_CameraStart` 0x1d9140:47, cleared by `Script_CameraEnd` 0x1db348:12): first press fades to black
  over `min(length − clock, 45)` ticks, `+0xA4 = FRAME_RATE_MUL` (1.0 [L ee_start_05, docs/systems/
  script-player.md]), `Music_Event(5)`, `+0xAB5` bit 0, the kind-2 stream objects' drones flagged
  `+0xC8 |= 0x40000`, and on the `ScriptCam` arm `Input_ClearAllActions(-1)` + `Text_FlushAllSubtitles()`
  [M 0x1dac78:23-70]. Later presses only clear `+0xAB2` bit 4 [M :86-88]. A no-camera script (event 0x10)
  just kills its camera stream [M :32-47].
- `Script_Update`'s do-while [M 0x1da998:19-100]: while `+0xA4 <= +0xAB4` the script ticks normally (the
  fade runs over real time, the scene keeps playing at 1×); once the fade passes its length `+0xAB5` bit 5
  is set and the loop SPINS — every stream's `Script_Run` + the clocks +1 per pass inside that one call —
  until the script stops. The clocks never skip, so commands and events (op 0x12) fire at their scripted
  ticks, compressed into wall-clock.
- `Script_Interp` case 2 (kind-2 anim) and case 4 (camera) RETURN while `+0xAB5 & 0x20` [M 0x1d9a08:146,
  205] — the anim object's `+0x90` world matrix freezes at the last pre-freeze pose (its clamped local
  frame at clock press + fade_len [M case 2's 0x90 write against `+0xB0`]). Case 1 (kind-1 entity spline)
  has NO freeze gate [M 0x1d9a08:91-139] and keys through the catch-up.
- Event 6 (cmd 0x12 → `Script_EventHandler` case 6 [M 0x1d94c0:73-84, 125-134]) snapshots the kind-1/2
  prev stream object's `+0x90` into `SCRIPTINFO+0x40` WHEN IT FIRES (the player's own matrix as the
  fallback); `Script_Free` re-enables Bond there [M 0x1da880:44-48] — the same delivery on both paths.

So the game's skip hand-back is **press-time dependent**: for a skip armed at script clock K, an event 6
firing at tick ≤ K + fade_len captures the play-path pose; one firing later captures the anim's frozen
frame-(K + fade_len − start_clock + start_frame) pose (clamped to [1, anim length]) — the anim's end pose
only when the anim had already run to its last frame by the freeze tick.

## Verification

`crates/nf-game/tests/nis_skip_handback.rs` (new) drives every event-6 SP script of 07000002/03/14 through
the port's REAL skip input — `Scripts::skip_request` (the viewer's key handler latch, lib.rs 5761) →
`begin_tick` → `SP_Update` state 2 → `Script::ffwd` — with `spawn_once_scaled` at the placement's own base
and scale, three runs per script: play, early skip (latch held from t3), mid-scene skip (t = event-6 tick/2).
Each skip result is asserted against the [M] rule above, computed from the disc's own anim data
(`AnimWorld::anim_obj_matrix`, the census's harness):

| level | script (plc) | e6 tick, prev stream | play | skip@K+fade_len | mid skip |
|---|---|---|---|---|---|
| 07000002 | 0x0600007f (#399) | 739, anim (0x06000501) | (-45.560, 5.918, 8.148) | @4+45 (-45.450, 5.899, 8.125) | @370+45 (-45.343, 5.891, 8.189) |
| 07000002 | 0x06000084 (#400) — the intro [L] scene | 1539, anim (0x06000504) | (-27.502, 1.162, 32.364) | @4+45 (-27.295, 1.218, 28.952) | @770+45 (-27.357, 1.128, 32.032) |
| 07000003 | 0x06000080 (#475) | 389, anim (0x06000552) | (15.092, 5.554, -12.665) | @4+45 (14.381, 5.538, -13.758) | @195+45 (14.406, 5.560, -12.804) |
| 07000014 | 0x0600062f (#382) | 1197, kind-0x31 entity | (-55.495, 1.546, 42.017) | identical (press-invariant) | identical |
| 07000014 | 0x06000848 (#407) | 949, fallback | fallback | fallback | fallback |
| 07000014 | 0x06000833 (#276) | 1399, Wait/Marker (fallback) | fallback | fallback | fallback |

- The play path reproduces the task's [L] pin: 0x06000084 play (-27.502, 1.162, 32.364) vs the game's
  fresh-boot (-27.504, 1.131, 32.348) — x 0.002, y 0.031, z 0.016 apart (sub-frame anim timing; within the
  test's 0.05 epsilon).
- Every anim row matches the frozen-pose prediction at its own press tick; for the early skips the frozen
  frame is mid-anim, and the tests additionally assert the result is NOT the anim's end pose there — the
  task's symptom number (-27.30, 1.23, 28.90) is this frozen pose (the port's (-27.295, 1.218, 28.952)),
  not the end pose.
- The kind-1 row (0x0600062f, the level-14 capture marker) is press-invariant, as the decompile predicts
  (no freeze gate on case 1).
- 07000014's 0x06000833 (the vent-drop) is deliberately NOT asserted: its prev stream is Wait/Marker only
  [D the pack's stream dump], so the strict [M] rule is the player-matrix fallback — but the port carries
  the level-14-vent-drop capture arm there ([S] unverified, pinned by its own test), which reads the
  scripted Bond body's live root and would freeze with the kind-2 gate on a skip. An [S] item never
  passes/fails (the task rules); the test prints it instead. In this harness it fell back on both paths
  (the body anim never resolved a matrix), so both paths agreed anyway.

The port code carries the whole chain since earlier tasks (verified, not modified): `Script::ffwd`
(fix-dev, the arm per [M 0x1dac78]), the catch-up spin in `Script::update` (fix-script-commands), the
`fade_flags & 0x20` freeze gates in `Script::interp` kinds 2/4/7 [M 0x1d9a08:146, 205], the event-6
snapshot (`Script::event` case 6) and `Script::free`'s `PlayerEnable`.

## What I ran

- `cargo test --offline --release -p nf-game --test nis_skip_handback` — 2 passed (the numbers above).
- The full four-crate suite via `tools/test/suite.py` (fast runner, started detached at 17:34 UTC; NF_ISO
  the on-foot slim image) on this tree. My change touches no library code (one new test file + docs), so
  the suite's results apply to the final tree as built. See Suite for the exact closing numbers.

## Suite

(runner: `NF_ISO=$NF_ISO nohup python3 tools/test/suite.py > suite.out` — updated at run end; the final
commit quotes the closing numbers.)

- Build: `build ok: 878 test binaries in 1129 s`.
- Environmental failures on the slim on-foot ISO (expected — the same four failed in the previous
  sandbox's suite.log):
  - `identity_disc::the_real_disc_passes_the_identity_check` — the identity test wants the 1,191,072-sector
    retail image; the slim on-foot image is 518,024 sectors.
  - `disc_golden::bigf_header_fields_behave_as_documented`,
    `disc_golden::bigf_tables_match_the_golden_manifest`,
    `disc_golden::directory_tree_matches_the_golden_manifest` — the three slim-ISO disc_golden failures the
    task rules allow.
- Everything else green through the binaries completed at report time, including `nis_handback_census`
  (the play-path census this task builds on) and `dev_tools_live::dev_skip_ends_the_gatehouse_cutscene`
  (the real-viewer skip path).

## Live checks for the main agent

1. **The game's skip hand-back [L]** — the one measurement this task could not make (no PCSX2 here).
   Fresh boot (a save-state resume skips the boot path — not usable), level 07000002, let the intro
   start, then press the skip (R1 or D-pad down, pad 0 [M psiInput_MapInputs 0x10cd80:415-417]) and watch
   Bond's position (player 0 `BLData+0x30`, or the SP's `SCRIPTINFO+0x40` at the `Script_Free` call
   0x1da880) through the fade-in. Expected per the [M] chain: for a skip pressed within the first seconds,
   Bond re-enables near (-27.30, 1.22, 28.95) — the anim 0x06000504 object frozen at frame
   (press + 45 − start_clock + start_frame); a skip pressed late in the scene should land near the play
   value (-27.504, 1.131, 32.348). A watchdog on the event-6 write to `SCRIPTINFO+0x40` plus the anim
   object's `+0x90` at the freeze tick settles the frame arithmetic exactly.
2. **Held vs tapped skip**: the game calls `Script_FFwd` on every held tick; the port latches one tick per
   press (the arm is identical either way). A live hold-against-tap comparison of the landing spot would
   confirm no hidden held-only path.
3. **The [S] vent-drop discriminator** (carried from level-14-vent-drop, still open): for 07000014's
   0x06000833, read `SCRIPTINFO+0x40` at the free — the strict [M] fallback predicts the player's own
   disabled matrix; the port's [S] arm predicts the scripted Bond body's (anim 0x06000857) live root.

## What's left

- Nothing blocking on the port side. The two [L] verifications above are the only open items, both
  main-agent live checks.
- The 07000014 vent-drop arm stays [S] (its own test pins it; a live read would upgrade or replace it).

## Usage

not visible to me — this environment shows no token, credit or context-window figures.
