## Your task: `escort-door-harness`, the escort's interior doors test harness (07000001)

`crates/nf-view/tests/fix_npc_doors_live.rs::the_escort_opens_the_interior_doors_on_his_route` is `#[ignore]`: the walker's resupply deadlocks door #2 against the escort's DSTATE_OpenDoor (a harness problem, not the port). Fix the harness so the test runs, un-ignore it.

Rule: real input only (`step_bond` movement/aim/fire/use; no teleports past the blocker, no channel or state pokes). When it stops, find what the game does there [M]/[D] and port that, never a port-side shortcut. Un-ignore each test when it passes, with a sister that can fail.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/escort-door-harness`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-24T13:32Z

---
## Report (done, 2026-10-24T14:41Z)

