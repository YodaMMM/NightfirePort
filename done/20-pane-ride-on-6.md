## Your task: `pane-ride-on-6`, intact glass really stops Bond on every level

Branch from origin/s/pane-ride-on-5, merge current origin/master, and read its docs/reports/pane-ride-on-5.md. The door fan is beaten (57.6 to 7.2); the d3 walk's last red in the 01 binary is the ambush pocket, mapped as the total ray asymmetry. Take its pocket root fix first. Then settle the 03 red: walkreal_07000003_live's real-damage Agent walk died at tick 106206 at (22.93, 6.37, 12.51) and has no baseline, so run it on current master in the same sandbox to see whether pane riding causes it, and repair it gated to 07000003 if it does. Then the walkreal battery from 04 to 0b and on, one level per tool call. Remove the dead_pane_live ignores once every walk is green. Keep the kind-0/1 props out of the ride set and carry the live checks forward, including the new check 6. Real input only, no top-ups, never lower damage or health, and level helpers stay gated to their level.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pane-ride-on-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T22:51Z

---
## Report (done, 2026-11-09T00:55Z)


# pane-ride-on-6 — intact glass really stops Bond on every level

Branch `s/pane-ride-on-6`, cut from `origin/master` b059df948. The named base `origin/s/pane-ride-on-5`
is gone from the remote — the queue's `done/20-pane-ride-on-5.md` says the report lived on the branch and
the driver closed the task from it; master already carries that state (the door fan 57.6→7.2 and the
pocket root fix landed with master b059df948, "01 walk with guards breaking glass", which also folds the
guard-pane-drain-3 aperture lure). Everything below was therefore re-verified on master rather than
re-fixed.

## Done

1. **The d3 walk's "last red" (the ambush pocket) is not on master — verified green.**
   `walkreal_07000001_live` whole binary: **7 passed / 0 failed / 1 ignored, 62.2 s**. The Agent walk
   (`walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent`) passes in 10.6 s, ends at
   **43.6 health** (51 hits for 56.4, 11 return-fire fights), crosses the ch-0xF slab deliberately, and
   the pop is answered from the mouth: #12 killed at tick 12427 on the back-off leg, rounds 0-4 fight
   [2, 3, 4, 17] from the mouth with the band-edge window for the walled ones [L this session's runs,
   /tmp traces on the sandbox tree]. No code change was needed or made; the pocket root fix (guards
   breaking the pane that blocked the sight lines) is master b059df948.
2. **The 03 red is settled: pane riding does NOT cause it.** The pane-ride-on-5 tree's
   `walkreal_07000003_live` real-damage Agent death at tick 106206 at (22.93, 6.37, 12.51) has no
   baseline, so I ran it on this (master) tree in this sandbox: **the Agent walk passes** (46.7 s), ends
   at **76.8 health** at tick 154500 (30 hits for 23.2, 16 return-fire fights) — the 106206 death is not
   reproducible on master, so no 07000003-gated repair is warranted (a repair would be fixing nothing).
   Whole 03 binary: **6 passed / 0 failed / 2 ignored, 388 s**.
3. **The walkreal battery from 04 to 0b, one level per tool call** (whole binaries, `--test-threads=1`,
   NF_ISO set, this sandbox):
   | binary | result | time |
   |---|---|---|
   | walkreal_07000001_live | 7 passed / 0 failed / 1 ignored | 62.2 s |
   | walkreal_07000003_live | 6 / 0 / 2 | 388.0 s |
   | walkreal_07000004_live | 9 / 0 / 1 | 15.0 s |
   | walkreal_07000005_live | 14 / 0 / 0 | 73.1 s |
   | walkreal_07000006_live | 5 / 0 / 2 | 17.7 s |
   | walkreal_07000007_live | 5 / 0 / 1 | 23.1 s |
   | walkreal_07000008_live | 5 / 0 / 1 | 130.3 s |
   | walkreal_07000009_live | 9 / 0 / 0 | 15.7 s |
   | walkreal_0700000a_live | 13 / 0 / 2 | 52.0 s |
   | walkreal_0700000b_live | 7 / 0 / 0 | 44.4 s |
   (0a and 0b ran as two sequential cargo invocations inside one tool call because of the clock; every
   other level had its own call.)
4. **dead_pane_live ignores: none exist on this tree, and the binary is green.** `grep '#\[ignore'` on
   `crates/nf-view/tests/dead_pane_live.rs` finds none (the removal predates this branch — the last
   master commit touching the file is 2998b6a76). Run: **3 passed / 0 failed / 0 ignored, 3.0 s** —
   the shot-out-window-stops-blocking law holds on real input with nothing skipped.
   `glass_pass_every_level_live`: 3 passed / 0 failed / 3 ignored (the three pre-existing measured
   ignores from glass-pass-every-level-3, unrelated to dead_pane), 3.5 s.
5. **The kind-0/1 props stay out of the ride set** — untouched: `Drones` still takes only
   `b.kind == 2` rows into the PANE_RIDE dynamic set (crates/nf-game/src/drone.rs, the
   "Behind PANE_RIDE (on)" block), the kind-0/1 prop rows keep master's out-of-world status quo.
   No code changed on this branch, so no walk moved, no rand draw changed, no test changed.

## How it was verified

Every run above is a real `cargo test --offline --release` invocation with `NF_ISO` pointed at the
on-foot ISO, `--test-threads=1`, real input only (the walker's own walk/shoot/resupply machinery; no
top-ups, no health or damage writes). The Agent-walk health figures come from the runs' own
`[real-damage] at tick ... end health ...` summaries under `--nocapture`.

## What's left

- **The full suite did not fit this sandbox** (lesson 10's escape hatch): the disk is 9.9 GB with the
  repo + ISO on it, `nf-view` alone has 1068 test targets, the 4-crate `--no-run` build hit 100% disk
  twice (deps grew to 6.2 GB with two artifact generations) and the background fast runner
  (`tools/test/suite.py`, started with nohup as instructed) was killed by the sandbox between tool
  calls — twice, silently (empty `suite.out`). The build was finished per-binary instead and the
  task's battery ran in the foreground. **Not run this session:** walkreal 02, 0c, 0d and the rest of
  the "and on" tail, the ai/turret binaries, `npc_doors_live`/`fix_guard_misc_live`, nf-formats/
  nf-audio tests, `cargo test -p nf-game --lib`, and the final-tree fast-suite total line. No code
  changed, so the changed-code re-run rule (lesson 10) is vacuous this session — but the next session
  should still run the full battery on the merged tree.
- The 01 binary's 00agent test stays ignored with its measured reason (the AND gate's dry door phase —
  guard-pane-drain-3's record, unchanged this session).

## Live checks for the main agent

Carried forward from pane-ride-on-2's report (checks 1-5, all still open, all still wanted) plus this
series' check 6. pane-ride-on-5's report file is not visible to me (its branch was deleted from the
remote), so check 6 is reconstructed from the driver's task text; re-reading it from the lost report
would be better.

1. (carried from #1) 07000001 museum hall, fresh boot: does Bond's capsule collide with the 0x20
   kind-0/1 floor props (e.g. placement 22's object at (−10.7..−10.0, 6.1..8.2, 11.5..12.5))? The ride
   keeps them out of the set; a live answer arbitrates their [S] status.
2. (carried from #2) 07000001: do the hall guards deflect on the kind-2 sheets — the sheet at
   z −1.167, x −1.909..−0.086 (placement 136), a guard patrolling at z ≈ −0.5: `gdbwatch.py` on the
   guard's +0x90 matrix while he crosses the sheet plane.
3. (carried from #3) 07000009's 16 hp-6 panes (placements 108-123) and the drive glass — unchanged.
4. (carried from #4) 07000001 at d2/d3 under the ride: is the stairwell-head pocket's east wall opening
   really the guards' firing lane — i.e. does #12's muzzle (from (−30.1, 14.9)) reach a Bond at
   (−23.3, 14.8) while every Bond→#12-box ray stops? A `fieldprobe.py` pair (the guard's muzzle matrix
   + a bond poke at the mouth) settles the asymmetry this port's ray reads showed.
5. (carried from #5) 07000001 d3, fresh boot: which channel/script enables #5 (0x5000038) — the read
   would pin the door-open coupling the port inferred from the 29-tick gap [L the log pairing].
6. (check 6, reconstructed — pane-ride-on-5's report not visible to me) The pocket's ray asymmetry on a
   fresh boot: with Bond at the mouth (−23.3..−25.7, 6.4, 14.8), do the pocket guard's (#12,
   (−31.31, 6.34, 11.51)) sight/muzzle rays REACH Bond while theBond→#12 rays stop — and does the
   guard's own gunfire BREAK the intervening pane (the master b059df948 "guards breaking glass" law) —
   `fieldprobe.py` on #12's muzzle matrix + the pane's hp byte during the pop window.

## Usage

The session's input/output tokens, context used and cost: **not visible to me** — this environment
exposes no token or cost counters to the agent. Wall-clock facts I can see: session start 22:57 UTC,
branch push 23:05 UTC, this report written ~00:45 UTC 2026-11-09, sandbox deadline ~00:56 UTC.
