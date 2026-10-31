## Your task: `level-1b-drake-walker-kill`, Drake killed by the player's own fire starts the finale like the game

`crates/nf-view/tests/interactables_0700001b_live.rs` (~line 211) is #[ignore]d: when Bond kills Drake through the Walker (`w.shoot`: exposed_aim box-centre aims plus the reload/strafe inputs), the death channel 0x0e never rises and the NIS chain stays dead. The same kill through raw `step_bond` (`probe1b_blockers::probe_drake_death`) spawns him enabled, dies at t81 into AstronautDeath 0xbf and raises ch 0x0e. Find what differs between the two kill paths (the hit data, the damage route, the death state entered, or a Walker input that interrupts the death) by tracing Drake's state and the ch-0x0e writer on both [M: cite the death handler that writes 0x0e]. Fix the port (not the test) so a real-input kill raises 0x0e and the finale chain runs, then un-ignore the test. Keep the teleport approach to the arena as it is (documented, the west boundary is sealed). Run the crates you touch and the 1b tests.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/1b-drake-walker-kill`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-10-31T08:59Z

---
## Report (done, 2026-10-31T10:32Z)

