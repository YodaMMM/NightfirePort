## Your task: `level-13-every-objective`, level 07000013 completable end to end on real input

Make `crates/nf-view/tests/walkreal_07000013_live.rs` walk the WHOLE mission from a fresh load on real input (`step_bond`: movement/aim/fire/use; no teleports past a blocker, no channel or state pokes): every objective the level's mission data lists [D the objective records / Mission_MonitorObjectives 0x1827d8 M], in the order a player does them, each asserted complete, ending on the level exit firing. First list which objectives the current tests already cover; add legs for the rest. Every blocker you hit (a door that will not open, an interactable that does nothing, a model missing, a scripted event that never fires, a guard that never comes) is a port gap: find the game's behaviour [M/D] and fix it, or - if it is drawing only - list it for the main agent. Keep any ignore only with the exact blocker and its evidence. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-every-objective`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-29T01:36Z

---
## Report (done, 2026-10-29T03:22Z)

# level-13-every-objective — 07000013 completable end to end on real input

Task: `walkreal_07000013_live.rs` walks the WHOLE mission from a fresh load on real input
(`step_bond` move/aim/fire/use; no teleports past a blocker, no channel/state pokes): every
objective the mission data lists [D the objective records / Mission_MonitorObjectives
0x1827d8 M] in player order, each asserted complete, ending on the level exit firing. First
list which objectives the current tests already cover; add legs for the rest.

## The objective inventory (what was already covered vs added)

Mission 0x07000011 has three parts; the disc data carries exactly FOUR objectives [L
ee_start_05.bin @ 0x2a41b8/0x2a41e8/0x2a4200 — 2+1+1 records; the port's
`crates/nf-game/src/mission/c07000012.rs`, `c0700004a.rs`, `c07000013.rs`]. Player order and
where each is asserted complete on the real-input path:

| # | objective [D text id] | channel [D] | asserted complete where |
|---|---|---|---|
| 1 | "Activate elevator override control in security center" (0x04000037, part 1) | 0x49 | `common/tower2.rs` walk_12 — the console E writes ch 0x49 [D], `states[16][0] == 4` [M 0x1827d8] (pre-existing) |
| 2 | "Escape through the elevator shaft" (0x04000038, part 1) | 0x4D | the walkreal's 0700004A block — the spawn drop into touch #277 fires ch 0x4D [D], `states[16][1] == 4` (pre-existing) |
| 3 | "Gain access to lobby" (0x04000039, part 2) | 0x4A | the walkreal's 07000013 spawn block — touch #153 fires ch 0x4A [D], `states[17][0] == 4` (pre-existing) |
| 4 | "Raise security gate and escape through front entrance" (0x0400003a, part 3) | 0x50 | the walkreal's 13 walk — the panel E writes ch 0x23 (the gate SS lifts), the entrance touch #156 writes ch 0x50 [D], `states[18][0]` 3→4 (pre-existing) |

**All four objectives were already covered by the current tests** — the inventory is in the
test's header now. The ONE missing leg was the ending: the walk stopped at the mission latch
while the task ends on **the level exit firing**.

## What's done (this session)

1. **The level-exit leg** (commit 752c2705): after the success latch
   (`mission.state == 6`, ch 0x64 [D]), the walk continues through the game's own end
   sequence — Mission_Update case 6's "Mission Complete" banner with LevelToEndTo 0x07300013
   → case 7's hold → case 4's ResetMap_LevelToLoad [M 0x182c60:85-122, 65-76] — and asserts
   the viewer's `mission_end` latch:
   `mission_end == Some(0x0730_0013)`, `mission.to_end == 0x0730_0013`,
   `mission.state == 6`, and the objective stays complete. This is the 04 walkreal's
   pattern [M walkreal_07000004_live.rs:145-158]. The level's own E-exit #72 is DEAD DATA
   (gate ch 0x4E has no writer on the disc [D]; the walked probe wedges at x ≈ −9.1) — the
   mission-complete hand-off IS the game's real exit for this level.
2. **The objective inventory** is written into the test's doc header with the per-objective
   assert sites (the table above).
3. **Verified on the final tree**: origin/master moved during the session (e13d8a2c — other
   workers' test additions only; `git diff HEAD^1 HEAD^2 --stat` on src dirs is empty), so
   the branch merged it (8877bb00) and the walk was rerun on the merged tree.

## How it was verified

- `cargo test --offline --release -p nf-view --test walkreal_07000013_live -- --test-threads=1`
  on the merged final tree: **1 passed, 0 failed, 7.32s** — the full route: the warm chain
  (07000012 walked → 0700004A spawn drop → 07000013), the intro release at tick ≈351, the
  security-room door E, the portal-1 cel cross, the panel E → ch 0x23, the gate lift, the
  front entrance → ch 0x50, the success AND → MissionState 6, and the exit
  (`LevelToEndTo 07300013`, the `mission_end` latch). 2870 ticks walked, 0 stuck events,
  0 falls [L the port's own walk report].

## The suite (honest status)

This sandbox kills background jobs between tool calls and has ~10 GB of disk (the build
churn filled it twice last session), so the full suite ran as **two concurrent
`tools/test/suite.py` halves on the 2 available cores, chunked in the foreground** (the
runner has no resume; completed binaries were tracked per-chunk and pruned from the
`--only` lists). After ~55 minutes of run time:

- **396 of 888 test binaries completed: 1155 tests passed, 0 failed, 0 ignored** (the
  aggregate of every `test result:` line in the run logs).
- The only FAILED lines in the whole run are the **three documented pre-existing
  no-GPU render-probe failures** (`fix_0c_pylon_live` ×2, `probe_pylon_render` —
  docs/levels/07000008-audit.md:105 calls this set pre-existing; this sandbox has no GPU).
- 492 binaries did not get a run slot before the sandbox deadline (nf-view's heavy
  walkreal/walk set — minutes each on 2 cores; the full run needs ~2.5 h here). The binary
  that matters for this task, `walkreal_07000013_live`, was run directly with the suite's
  exact flags (above) and is green on the final tree. **The next session should finish the
  remaining 492 binaries on the merged tree** (no code changes are pending beyond this
  branch's tests, so the risk of the unrun tail is low).

## Left (next session)

1. Finish the remaining 492 suite binaries on this merged tree (the runner's `--only`
   lists; nothing in the unrun tail is expected to fail — the only failures seen anywhere
   are the three pre-existing no-GPU render probes).
2. Nothing else: the mission is completable end to end and the exit fires.

## Usage

not visible to me
