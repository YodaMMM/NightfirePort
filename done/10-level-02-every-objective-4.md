## Your task: `level-02-every-objective-4`, 02: the real-health walk, without losing the green one

Continue `origin/s/level-02-every-objective-3` (merge `origin/master` first; read its report). Its findings stand (the walker's body-block response: shoot / step back / flank / resupply, all real input; the real pickup takes work). It was not merged because it put `#[ignore]` on `walkreal_07000002_live::walk_the_castle_all_objectives_to_the_exit`, which is GREEN on master, and changed `common/walker.rs` without a run of every walkreal_* binary. Do: keep master's green walk as it is (do not ignore it); put the real-health walk in its OWN test (ignored until honest); keep the walker change only if every walkreal_* binary stays green with it (quote every total). Then chase the two blockers: (1) the west-wing lip fight (drone 4 with no exposed part at 1.4-2.7 u — what the game's auto-aim/body parts allow there [M Check_AutoAim 0x1a73f8]); (2) the enabled geisha leaving Hostage 0x0A while combat runs nearby — find whether the port's hostage/captor state machine moves her where the game keeps her [M the hostage DSTATE handlers; L if needed, name the live check], and fix it with evidence (this one a player would see). Run walkreal_07000002_live, interactables_07000002_live, every walkreal_* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-02-every-objective-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-31T08:23Z

---
## Report (done, 2026-10-31T09:44Z)

