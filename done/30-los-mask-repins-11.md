## Your task: `los-mask-repins-11`, the 07000011 walks green again under the game's guard sight

Branch from origin/master (823ff190b turned on the game's guard sight: per-object masks, the cel walk, the sight rotation, the guard's own cel, the own-cel boundary push, FellOutMap). Read docs/reports/los-mask-repairs.md on origin/s/los-mask-repairs (reports do not go to master). These walks were ignored because the game-correct sight moved their fights: walk_spawn_to_the_lower_floor_load_point_real_damage_00agent. Read each ignore message. Then re-route each walk with real input so it passes, and remove its ignore. Never lower damage or health, no top-ups; helpers gated to 07000011 and the difficulty. Run walkreal_07000011_live (and its sister binaries) and interactables_07000011_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-los-mask-repins-11`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-10T21:15Z

---
## Report (done, 2026-11-10T23:18Z)

# los-mask-repins-11 — the 07000011 walks green again under the game's guard sight

**Result: DONE.** The one ignored 07000011 walk —
`walk_spawn_to_the_lower_floor_load_point_real_damage_00agent` (d3, walkreal_07000011_live.rs)
— is re-routed on real input and **un-ignored**. No damage or health was lowered, no top-ups
were added, no shared helper rule changed: every lever is walk-local and gated to
`difficulty >= 3` inside `walkreal_07000011_live.rs` itself. Commit `49ffa3789` on
`s/los-mask-repins-11` (base `origin/master` `0e1371451`).

## What the ignore was (measured, not guessed)

The ignore message said: "Bond reaches the 18.9 strip at 46 health and dies at tick 19894 to
#19 and #36 (weapon 0x19) at 2.6-5.6 u". `NF_RF_TRACE` runs of the ignored test unpacked it
into three stages [L all runs this task, release build]:

1. **The fights MIX across the stairwell's connector-2 ends.** At the connector bottom the
   return-fire pick opened fight#29 on a guard walking the 23.0 strip ABOVE the flight lip —
   the stand's eye rays never saw him (`aim None` at t0 and t60) — while #19 (2.6 u on the
   18.9 strip, hp 12 the whole stand) and #36 (elevated over the well at (19.7, ~21, 0.4))
   burst health 39 → 0 in 90 ticks. The plain lane pick (this walk had no `hurting_first`)
   stayed on the blind target inside the `fighting=true` window.
2. **The shoot loop drifts Bond off a stand spot.** A first fix used `shoot_bounded` at the
   mouth; its opening repositioning strafe (`t - last_progress < 60` with
   `last_progress = 0`) carried Bond ~2 u west per stand — the run died at d 1.1 of #19 with
   the #19 stand opening on clip 1 (the #18 stand emptied the clip) [L the second run].
3. **The lip blocks the up-look, not the down-look.** From the connector bottom, #29 (the
   connector top, (23.6, 23.8, −1.4), 17.5 in 5 hits) and #36 (39.0 in 11 hits on one run)
   shot Bond while `DamageLog::visible` returned None for both. From the 23.0 strip's east
   end ONE level up, the same guards are a down-look over the same lip: the stands killed
   #27/#26/#18 from there [L the passing run].

## The fix (all real input, all `difficulty >= 3`)

1. **`answer_visible_guards` (test-local helper, this file only):** live, enabled,
   non-WAIT_SWITCH hostiles the eye rays can reach, nearest first, a `stand_and_fire` each
   (Bond HOLDS his spot — no strafe drift), a `reload_full` between guards when the clip is
   under 5 (a player reloads between guards). Called at two spots:
   - **the connector-2 TOP** — after the 23.0 strip's "back along" leg, before the flight
     descent;
   - **the 18.9 strip MOUTH** — after `band(connector 2's bottom)`, before the strip walk west.
2. **`hurting_first = true`** (the 07000008 Agent sisters' existing opt-in flag, set here on
   this walk's own DamageLog instance): the pick answers the shooter actually hurting Bond,
   drops cooled candidates, and the fight retargets every 30 ticks. No shared-code change —
   the flag is checked by the existing `damage_log.rs` arms.
3. The un-ignored test's doc comment documents all three levers with the trace evidence.

The top-up walk, the d1 and d2 sisters never enter the new blocks (they are inside
`if difficulty >= 3`), and `hurting_first` is set only in the same d3 block — their code
paths are untouched.

## Verification

| binary | result |
|---|---|
| walkreal_07000011_live (all 4 tests, incl. the un-ignored d3) | **4 passed / 0 failed / 0 ignored** |
| walkreal_07000011_route_live | 1 / 0 |
| interactables_07000011_live | **17 / 0** |
| nf-game --lib | 782 / 0 / 4 ignored |
| ai_live | 37 / 0 |
| walkreal_07000012_live (shares tower2's route) | 3 passed + 1 ignored (`los-mask-repins-12`, another worker's ledger — pre-existing) |

All runs `--test-threads=1`, release, `NF_ISO` the on-foot image. The d3 walk completed the
whole chain — 07000011 spawn → the stairwell descent → the Load Point exit → the real
07000012 load → tower2's route → the shaft exit → the real 0700004A load → the mission-row
asserts — in ~12 s of sim.

No `crates/*/src/` or `crates/nf-view/tests/common/` file was touched (`git show --stat
49ffa3789` — only `walkreal_07000011_live.rs`), so no other level's walk shares my diff; the
sister walks above are the direct witnesses.

## The full suite: did not fit this sandbox

`tools/test/suite.py` was started twice in the background (it dies between tool calls in this
sandbox) and the build+run was attempted in the foreground six times. The 9.9 GB sandbox disk
cannot hold the ISO (1.0 GB) + the toolchain (~1.6 GB) + the full four-crate release test
binary set (~5 GB) at once: the build repeatedly died with
`couldn't create a temp dir: No space left on device (os error 28)` on a rotating set of
binaries (walkreal_probe15m, gadget_grapple_user_spot, guard_capsule_radii_live,
rigid_body_draw_live, placement_kind_census, walk_0700000a_live, mp_body_anim_live,
walkreal_07000041_wall_live, script_events_3_4, walkreal_0700000c_barrelrow_live,
guards13_start_probe, probe16_writers, hurt_07000046_live, camera_death_crouch_live,
probe_0b_doors, nf-formats lib, nf-view lib). Stale-artifact dedup +
`strip --strip-all` of the built binaries recovered ~2.5 GB twice and the four-crate compile
got through once (`cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio
-p nf-view --no-run` → 0 errors), but each new suite attempt's nf-view relink burst refilled
the disk. **No suite total line from this sandbox.** The targeted binaries above (782 + 37 +
4 + 1 + 17 + 3 = 844 test results, 0 failures) are the verification this report stands on;
the next message can run the suite from this branch on a sandbox with more disk.

## What's left

- The full four-crate suite total on a disk that fits (the runner is ready;
  `tools/test/suite.py` on master).
- Live checks for the main agent: none required by this task — the levers are test-side only.
  If wanted: render `--bond 22.4,23.2,-3.0` facing south-west and down (the connector-2 top
  stand spot: #29/#36/#18 in view below/abreast over the well) and `--bond 22.89,20.81,2.83`
  facing west (the mouth: the 18.9 strip's guards in view, the flight lip above).

## Usage

not visible to me (no token/credit/context-window figures are shown in this environment — no
input/output token counter, no cost meter. The only usage signal I have is the sandbox clock
and the driver message's own estimates, which are not a meter I can read.)
