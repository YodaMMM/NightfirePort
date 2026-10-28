## Your task: `cutscene-handback-anim-prev-2`, re-derive Double Cross Part 2's tests from the game's real start

Start from `origin/s/cutscene-handback-anim-prev` (merge origin/master). Its event-6 anim snapshot is CONFIRMED by a live read: on a fresh 07000002 load the game starts Bond at (-27.172, 1.040, 29.796) and, when the intro scene releases him at frame ~1531, places him at (-27.504, 1.131, 32.348), settling to y 1.055 [L 2026-10-27 PCSX2, PLAYERS obj+0x30] - the branch's resolution (-27.50, 1.16, 32.36). On master Bond stays at the old start, and the 02 tests were calibrated on that wrong spot. With the branch these go red: fix_swim_live (4 tests), civilian_no_shoot_live `the_no_shoot_crosshair_keys_on_the_held_weapon`, walkreal_07000002_live `walk_the_castle_all_objectives_to_the_exit`, ladder_underwater_02_live, fix_ladder2_live, fix_ladder_water_live, ladder_02_live, bath_hatch_02_live, bathhouse_voices_live, bathhouse_shout_mixer_live, speech_facial_fallback_live, fix_swim_entry_live. Re-derive each test's route/expectations from the [L] start (cite it), never weakening a decompile-backed assert; if a failure is not explained by the start, find and fix the real cause. Also fold in `10-cutscene-handback-sp-path`'s item (1) if the SP path still drops the object. Full suite green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-handback-anim-prev-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-27T23:21Z

---
## Report (done, 2026-10-28T00:22Z)

Closed by the driver: the report is on `s/cutscene-handback-anim-prev-2` (docs/reports/cutscene-handback-anim-prev-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-28T00:22Z)

# Report: cutscene-handback-anim-prev-2

Branch `s/cutscene-handback-anim-prev-2` (from origin/s/cutscene-handback-anim-prev @ 3130bb95; origin/master was already an ancestor — "Already up to date"). The 02-suite tests re-derived from the game's real 07000002 start, and the sp-path item (1) verdict.

## The re-derivation

The game's fresh 07000002 boot starts Bond at (-27.172, 1.040, 29.796), holds him disabled through the intro cut-scene SP #400 (script 0x06000084, pack 07800002), and releases him at frame ~1531 to (-27.504, 1.131, 32.348), settling to y 1.055 [L 2026-10-27 PCSX2, PLAYERS obj+0x30] — the branch's census resolution (-27.50, 1.16, 32.36). The port now does the same: its event-6 hand-back snapshots the previous stream's ANIM object's matrix [M Script_EventHandler 0x1d94c0:73-84, :125-134]. Master's fallback left Bond wherever he stood — and twelve test binaries were calibrated on that stale spot. Two of their habits break against the real behaviour:

1. **Placing Bond mid-scene.** A placement made while the intro is still live was final on master (the fallback copied the placement back); with the branch the release overwrites it. Worse, the intro SP primes on the FIRST `step_bond` — `disabled` still reads false before it — so a "wait until !disabled" guard that never sees the scene silently does nothing.
2. **Routes assumed the release never happens.** Any leg whose geometry was reached from the old spawn now starts 2.5 m away.

Fix pattern (applied per failing test, with the [L]/[M] citations inline): step once to let the scene grab Bond, then hold the game's own skip action [M Script_FFwd 0x1dac78, SP_Update 0x1dbda8] until it releases him, and only then place/drive Bond. No decompile-backed assert was weakened: every channel write, substate, weapon and sound assert stands; only the test-authored placements/routes moved.

Green after the fix (foreground runs, `--test-threads=1`, slim ISO, this branch):

| bin | result |
|---|---|
| fix_swim_live | 4/4 |
| fix_swim_entry_live | 2/2 |
| fix_ladder2_live | 4/4 |
| fix_ladder_water_live | 4/4 |
| ladder_02_live | 3/3 |
| ladder_underwater_02_live | 6/6 |

That is all four red fix_swim_live tests, fix_swim_entry_live, fix_ladder2_live, fix_ladder_water_live, ladder_02_live and ladder_underwater_02_live resolved — 23 tests across 6 bins, each run twice (red before, green after).

## The sp-path item (1): the SP path does NOT drop the object any more

cutscene-handback-sp-path's item (1) was "`Scripts::update_player` loses the kind-2 stream's object slot before the event fires" (the anim-prev report's NF_SCRIPTLOG divergence: `prev kind Some(2) -> NO entity`). On this branch, on the real runtime, it does not reproduce: an NF_SCRIPTLOG=1 run of the fresh-load intro logs

```
[scriptlog] event6 hash 06000084 stream 3 prev obj 2 (kind Some(2)) snapshot (-27.50,1.16,32.36)
```

— the anim object is alive in its slot at event time, the snapshot matches the game's [L] release to 0.01/0.03/0.01, and the play path no longer falls back. Nothing to fold in. (The skip path fast-forwards the scene and snapshots the anim's END pose: `event6 ... snapshot (-27.30,1.23,28.90)` — the game's own skip behaviour was never live-measured; see Live checks.)

## What's left

1. **walkreal_07000002_live `walk_the_castle_all_objectives_to_the_exit`** — still red, at the Mayhew-computer shot (ch 0x48 [D pl[011] param 0; M 0x12bd48] — the assert is correct and untouched). Diagnosis: with the [L] release as the walk's start, the nav approach to the bunker corridor comes in on the RAISED floor (arrivals (-47.20, 6.46, -22.38) / (-47.82, 6.53, -23.20) across three leg variants — master's route descended into the lower walkway slot at (-48.4, 5.9)); from the raised floor Bond's eye is ~1.7 m above the box centre at 1.6 m distance and the point-blank down-shot clips the desk in front of the Destroy prop's bounds, so ch 0x48 never rises within shoot_point's 2400-tick cap. Two retargets (a wider stand-off east at arrive 1.4/0.6, then the lower slot at 0.9) did not get the descent back. Next round: either find the nav edge that reaches the lower slot from the east approach, or shoot the box's exposed top/east face from the raised floor.
2. **civilian_no_shoot_live `the_no_shoot_crosshair_keys_on_the_held_weapon`** — still red at "the civilian is live after the skip": `drawn` is the per-tick visibility ray from Bond [M drone.rs:17377 `dr.drawn = dr.enabled && visible_to(player, …)`], and after the intro skip Bond stands at the FFwd end-pose hand-back (-27.30, 1.23, 28.90), from where the ray to the civilian (-24.33, 0.95, 31.92) is blocked. The test plays the intro with its own load_and_skip; re-deriving it needs the same resolve-first pattern plus an aim position the game's own release spot reaches — not finished in the window.
3. **bathhouse_voices_live (2 tests) and bathhouse_shout_mixer_live (1 test)** — still red; these DO resolve the intro (they tick until !disabled and then skip the tail), so their failures are downstream of the release spot/pose (the shout key-on windows and the plate #389 touch at :143 `plate #389 raised ch 0x81`), not the mid-scene yank. Not diagnosed in the window.
4. **speech_facial_fallback_live** measured GREEN on this branch (2/2) — the driver's red list was a prediction; no change needed.
5. **The full four-crate suite did not run** — the background runner is killed between tool calls in this sandbox (two attempts: plain nohup and setsid; the second died during the -j4 build phase — this box also OOM-killed -j2 test builds in the previous session, so the suite's -j4 build is the likely killer), and the foreground `cargo test --no-run` build of all binaries took ~25 min of the window. Every binary this task TOUCHED ran green in the foreground; the remaining red bins are listed above. The next session should run `tools/test/suite.py` (pre-warm the build at -j2 first, then start the suite so its build step is a freshness no-op).

## Live checks for the main agent

1. **07000002, the intro's SKIP-path hand-back** — fresh boot, press skip during the intro cut-scene, then read PLAYERS obj+0x30. The port's FFwd hand-back lands at the anim END pose (-27.30, 1.23, 28.90); the game's own skip is unmeasured. If the game lands elsewhere, the port's FFwd needs the same per-tick anim-matrix treatment the play path got.
2. **07000002, the mid-level scene SP #399 (0x0600007f)** — play through the geisha wing until the cut-scene fires and read PLAYERS obj+0x30 when it releases Bond. The census predicts (-45.56, 5.92, 8.15) [D floor 4.75]; a [L] here would pin the second kind-2 hand-back on the real path.
3. **07000002, the Mayhew-computer corridor** — where does the game's Bond stand when the player shoots the computer (the lower walkway slot ~5.9 or the raised floor ~6.46)? That decides which approach walkreal's leg must take, and whether the desk blocks the raised-floor shot line in the game too.

## Usage

not visible to me
