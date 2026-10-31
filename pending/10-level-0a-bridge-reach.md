## Your task: `level-0a-bridge-reach`, 0a: reach the bridge on real input

`walkreal_0700000a_live` still places Bond at `[-11.35, 1.12, -18.6]` ("a REACH teleport onto the bridge's west end — the walk there crosses the wedging mouth legs", around line 884) and at `[-8.0, 1.12, 16.29]` (around line 1005). Read both comments and the related ignore notes. Find the game's route to each spot [D collision/placements/nav; M the classes used], walk it on real input and delete the writes; where the port wedges where the game does not, find the collision difference with evidence (the capsule rules: the head-on cull on the raw denom [M 0x10a1f0-0x10a2e8], Dist2Tri flags 0/99 [M 0x10a330/0x10a33c]). If the data shows no route, pin that with a sister assert and name the live check. Run walkreal_0700000a_live and interactables_0700000a_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-bridge-reach`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

