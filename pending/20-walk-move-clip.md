## Your task: `walk-move-clip`, Bond's wall contact: clip the move like the game instead of pushing out

Read the level-12-every-objective report (on master), "Blockers hit". Two 07000012 stand-ins (the door[19] doorway pincer, the north-wall funnel pinch) exist because the port resolves Bond's capsule with a two-sided depenetration push (`capsule_push` / `capsule_tri_contact`) whose sums pin him where the game walks through: the game resolves penetration along the move [M Intersect_CylGeom 0x109da0] and its walk push tail is dead [M Collide_Intersect 0x1ebaa8:353-373; L the +0x50 accumulator]. Port the game's move-clip behind an env flag `NF_WALK_CLIP=1` (default OFF), run the WHOLE walkreal battery with it on and off and report both tallies, and replace the 12 stand-ins with real input under the flag. Do not change the default - the main agent decides after a feel check. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/walk-move-clip`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

