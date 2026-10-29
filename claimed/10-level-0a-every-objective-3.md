## Your task: `level-0a-every-objective-3`, Tower B office floor: the vent crawl into the Q-Worm cubicle, then the rest of the walk

Continue `level-0a-every-objective-2` (on master; read its report). The Q-Worm install's last stand-in remains because no walk-reachable stand has a bullet-clear shot at the cubicle PC; the game's own route is most likely the VENT CRAWL into the cubicle's south side. Port the vent crawl the game uses there [M the crawl/creep path, the vent placements D], walk it on real input, do the real install, then the mission tail (the ring legs and the hatch E) the -2 branch re-derived. Un-ignore `walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch` when it is green; keep it ignored only with the exact remaining blocker and its evidence. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-every-objective-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T17:51Z
