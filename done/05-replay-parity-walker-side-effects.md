## Your task: `replay-parity-walker-side-effects`, recorded walks replay exactly (FIRST PRIORITY)

Branch from origin/master. Background (already measured): the sim is deterministic and a walk's inputs DO replay to the same state, tick for tick, once every stepped tick is captured. The one hole: crates/nf-view/tests/common/damage_log.rs, in the fight loop's `if reload {` branch, calls `self.scene.sim.as_mut().unwrap().step_bond(Input::default(), false, true, false)` directly instead of going through the walker's tick path. That step skips the walker's per-tick work (keep_alive top-ups, the cut-scene skip request, rise_watch, drone_watch). Do:
1. Route that reload step through the walker's own tick (tick_inner with reload = true, or an equivalent helper), so every walker-stepped tick goes through ONE function. Grep tests/common/ for any other direct `step_bond(` inside Walker/DamageLog methods and route them too (free functions that own a bare Sim are fine).
2. Add an optional input recorder at that single function (a Vec<nf_view::replay::TickInput> on the Walker, filled with TickInput::from_parts and stepping with to_parts(), so the sim sees exactly what is recorded).
3. Commit a test `replay_parity_07000004`: run the 07000004 full-mission walk (difficulty 1, top_up_health false, skip_cutscenes false: armour, StormUzi, armour, then shoot the 0x0500001e target), record, then replay the inputs on a fresh nf_view::load and assert per-tick equality of Bond pos/yaw/health and drones.frame, and the same mission_end tick. Expected: the mission ends around tick 3003 (the target dies to return fire at ~2683).
4. Walks that wait for the mission end should stop at the first tick mission_end is Some (the 04 walk kept walking 6300 ticks after it).
Then run the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/05-replay-parity-walker-side-effects`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox7 at 2026-11-12T00:12Z

---
## Report (done, 2026-11-12T02:10Z)

