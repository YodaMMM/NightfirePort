## Your task: `level-11-stairwell-bullet-wall-3`, the 01 contact walk with the new bullet rule

Start from `origin/s/level-11-stairwell-bullet-wall-2` (merge it first, then origin/master). The gondola window and the escort are fixed; one red remains, green on master: `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact`. The fight resolves differently, parking the contact in AllyLeadWait (0x20) at route node 15 (6.9, 6.3, 9.1) with Bond 6.1 away; the release [M 0x15eda8 case 3: pd > -7.0 && (pd != 0.0 || dist2d < 5.0)] reads pd == 0.0 and needs Bond within 5.0, but Bond's walk stalls at (1.38, 6.7, 11.61) in the hall (the nav gives up at its 3000-tick cap).

1. Find why Bond's walk stalls there (the hall geometry, a bot in the way, the nav's route) and walk it the way a player would with real input; check the release rule against [M] while there.
2. Green, plus the stairwell, gondola window, escort and foliage tests; run the FULL suite on the merged tree and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-stairwell-bullet-wall-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox7 at 2026-10-25T15:20Z

---
## Report (done, 2026-10-25T17:08Z)

