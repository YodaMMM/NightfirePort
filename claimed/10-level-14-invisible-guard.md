## Your task: `level-14-invisible-guard`, Countdown Part 1: a guard that is there but does not draw

In play on 07000014 a guard is present (he acts / can be fought) but is not drawn. The load log points at a candidate: "g-021: skin 050000b1 is carried only by 07000016.bin/07000015.bin - no container the session has loaded; the table never registers it. On 07000014 the 0x050000b1 placements still resolve through the Drone+0xe0 rewrite [M 0x14b300:82-95, :563-565]; a skin a placement DOES request with no session carrier just misses (AnimObjectNew 0x1be910:44-46) and draws no character". (1) Census every drone on 07000014 that is present/enabled at any point of the level but has no drawn character (no skin resolved, a held-back spawner mesh never released, a palette slot left FAR) - list each with placement, skin hash and why. (2) For each, find what the game draws [M/D; where does the game get the skin - a container the port does not load, a pack, the Drone+0xe0 rewrite] and fix the port so the guard draws with the game's model. Pin it with a test (the character for that drone has a resolved skin and a non-FAR palette slot while present). Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-invisible-guard`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-29T07:47Z
