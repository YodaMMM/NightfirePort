## Your task: `real-health-00agent-01-hall-7`, 07000001's 00 Agent real-damage walk reaches the exit

Branch from current origin/master; if origin/s/real-health-00agent-01-hall-6 is not on master, merge it in first. Read docs/reports/real-health-00agent-01-hall-6.md there and the ignore message on walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent. Take the g11 levers (i)/(ii) first, then the escort's west route, the slab ambush, the plate AND-touch and the exit. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000001_live binary and interactables_07000001_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-01-hall-7`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-15T02:00Z

---
## Report (done, 2026-11-15T02:38Z)


---
## Report from the worker (done, 2026-11-15T03:27Z)

# real-health-00agent-01-hall-7

Branch `s/real-health-00agent-01-hall-7`, cut from origin/master d1491c571 (the hall-6 work
was already merged there, and newer hall-7 attempts 7b-7e were on master as commits
06db4a156/9efda0b2e/ccdd9de13/68d01acbf/d3d5ef639/820819cb3/4e4e4b87a — there was no
`origin/s/real-health-00agent-01-hall-6` branch to merge). The task: take the ignore
message's remaining levers for `walk_from_the_spawn_to_the_exit_with_the_contact_real_
damage_00agent` — real input only, no top-ups, never lower damage or health, helpers gated
to the level and the difficulty.

## What's done

**THE WALK IS GREEN — THE IGNORE IS REMOVED.** `walk_from_the_spawn_to_the_exit_with_the_
contact_real_damage_00agent` passes: the level exit fires to 07000002. On the single-test
run the exit lands at tick 24955 with Bond at 4.8 health (36 hits for 95.2 absorbed, 11
return-fire fights, armour 0); on the whole-binary stream it lands at 14760 with every
assert still holding. Both runs are real input from the level load, no teleports, no
top-ups, no lowered damage or health.

The 7e wall was the west SMG #4 (skin 0x500003a, 12 hp, weapon 0x12): Bond died at t19729
at (-26.0,14.7) at 5.2 u with #4 UNHURT — he had landed zero hits on it all fight. Three
measured causes, each fixed by its own lever in `crates/nf-view/tests/walkreal_07000001_
live.rs` (commits 3f5a52f28 + bb7912e5c, all d3-gated; no shared-helper file touched this
round, so every other level's walk and replay is byte-identical by construction):

1. **The round order used the wrong visibility model.** The ambush order partitioned by
   the point-segment `DamageLog::visible`, while the shooter needs BOX ENTRY before world
   contact [M Collide_Jointy 0x1ec180 / 0x1ea398:216-229 — the game's own answer, which
   `exposed_aim` implements]. [L r7_trace.log NF_KILL_TRACE] rounds 0/2/3 burned their
   one-guard windows on the pocket #2 (point rays clear — [L] a stand-sweep probe with the
   world's own segment cleared almost every stand — but every part's box entry was
   world-shadowed: fire_state false the whole window, no shot fired, no ammo burn, no
   gunshot-hearing draw [M DroneFunc_SetNoiseHeard]) while the exposed #4 waited outside
   the break. The d3 order now partitions by `exposed_aim`, and a no-entry guard's window
   is skipped outright (running it only fed the blocked sidestep ladder, whose strafes
   drifted Bond into #4's lane).
2. **The contact walked into the fire line.** The escort lead does not wait for the ambush
   fight: by round 2 he had crossed the whole corridor and stood 0.4 u from #4 (r7_f2:
   shield drone 1 at (-30.15,13.68) vs #4 (-30.54,13.25)) — every tap at #4 crossed his
   cylinder, and the game's bullets hit ANY drone [M NDrone2_DealWithObjHit 0x140968:85-95,
   the shield gate] kept the trigger down. The d3 fight now re-takes a spot AHEAD of the
   contact between windows (2.2 u past him along his steering bearing, clamped east of the
   slab and into the walkable band) [M 0x15ea28 PlayerInWay]: the lead stalls with the
   contact BEHIND Bond, where the westward cone never reaches him. r7_f4 held him to 0.6 u
   of creep over the round.
3. **The escort's own clears had the same poison target.** r7_f4: the ambush fight boxed
   out (4 no-fire rounds — the new early exit) and the escort round 0's own-radius clear
   killed #17 (18621, for 5.4 hp) but then burned a blocked 10800-tick window on the
   upper-ledge #2 (aim None from every corridor stand), whose sidestep ladder drifted Bond
   to (-25.13,14.22) — in #4's lane — where #4's fire landed free (10.2 had ALREADY been
   dealt to #4 by that same clear: he was one window from dead). The own-radius and
   contact-radius clears now run the same `exposed_aim` skip at d3. r7_g1: the clear's
   first window on #4 (exposed true, 8.2 u) kills him, and the walk goes to the plate.

The d1/d2 sisters pass unchanged (8/8 in the binary): every change is inside
`difficulty >= 3` branches in that one test file.

## How it was verified

- `walkreal_07000001_live` (whole binary, `--test-threads=1`): **8 passed, 0 failed,
  0 ignored** [L 84.8 s on the final merged tree; 82.3 s pre-merge].
- `interactables_07000001_live` (whole binary, `--test-threads=1`): **10 passed, 0 failed**
  [L 30.4 s pre-merge, 30.5 s on the final merged tree].
- The final tree is merged with origin/master (6a9463509 — 07000008/14/1b test-file
  commits only; clean merge, no conflict) and both batteries were re-run on it.
- Instrumented runs on this sandbox ([L] /tmp/r7_base.log, r7_trace.log NF_KILL_TRACE+
  NF_KILL_TRACE_POS, r7_f1-f4.log, r7_g1.log, r7_final.log), every one through the real
  input path from the level load: the baseline reproduced the 7e death exactly (t19729, #4
  unhurt); the trace run isolated the box-shadowing and the cone block; f1/f2/f3/f4
  measured each lever (the order fix, the skip, the in-way hold, the boxed exit); g1 is
  the green single-test run; final is the green whole-binary run on the merged tree.
- **The full 4-crate suite did not run.** This sandbox reaps background processes between
  tool calls (the suite died twice before the first compile finished), and its 9.9 G disk
  hit ENOSPC on the full nf-view test-binary forest in the hall-4 and hall-6 rounds
  (documented there). No `crates/*/src/` file and no file under `crates/nf-view/tests/
  common/` was touched this session — the whole diff is `walkreal_07000001_live.rs` — so
  the other levels' binaries are unchanged by construction. The two whole level batteries
  above are the coverage; re-run the suite on a bigger disk before merging.

## What's left

- Nothing on this task's named levers: the g11 levers were already on master (06db4a156),
  the corridor ambush is won, the escort crosses the slab, the plate AND-touch fires and
  the exit loads 07000002.
- The full suite on a disk that fits it (and a sandbox that does not reap background
  processes), per the note above.

## Live checks for the main agent (PCSX2, fresh boot)

1. The west-corridor contact behaviour: after the hall AND, when the player fights the
   ch-0xF ambush pop from the corridor mouth, confirm the escort contact's lead walks him
   PAST the player down the west corridor toward the stairwell head while the player
   fights (the port measured him reaching (-30.15,13.68) — 0.4 u from the west SMG's post —
   mid-fight), and that stepping ahead of him on his path (in his PlayerInWay band
   [M 0x15ea28]) stalls the lead.
2. The box-entry sight asymmetry at the corridor's west end: from the corridor mouth
   stands (x -22..-24, z 14.5-16.5, crouched) confirm the upper-ledge guard (the port's #2,
   health 12, standing ~(-30.0, y 10.2, z 13.6)) cannot be hit (every part's ray enters a
   box only after a world contact — the port's `exposed_aim` says None) while the
   corridor-floor SMG at (-30.5,6.34,13.25) IS hittable from the escort-follow stands once
   Bond is behind the contact (the port's clear hit him for 10.2 there).

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token,
cost or context-window readout to the worker. What was visible: the task message's clock
(sandbox expiry ~04:01 UTC; this report written ~03:25 UTC), `git` hashes, test outputs
and process exit codes. No input/output token counters, no credit balance, no dollar cost.
