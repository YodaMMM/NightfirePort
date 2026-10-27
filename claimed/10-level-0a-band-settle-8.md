## Your task: `level-0a-band-settle-8`, the walk's vertical from the game's feet band

Start from origin/master (it has `level-0a-band-settle-6-c`: the radial-tail deltas reverted, D2 behind the off-by-default NF_CAPS_D2 gate) and read the reports of `s/level-0a-band-settle-6-c` and `s/level-0a-band-settle-7`. Their finding: the port's capsule tail (dir = v̂, mag = r−|v|, unconditional) is carrying Bond's vertical settle, which the game's tail almost never does (its operands are bounded by the HITTEST+0x50 accumulator arm [M Intersect_CylGeom 0x109da0:421-447]); the game's walk vertical is the feet band [M Player_FeetOnPoint 0x194188: the ray down +0.6, grounded at hitdist <= +0.1] plus the walked dy (settle-7's remaining candidate: the walk matrix's pitch-y [M Player_SSWalk 0x192310 tail, 0x193600:284-295]). Port the feet band as the y-settle authority, then re-land the [M]-faithful tail deltas (D2 at least) without lifting Bond anywhere: interactables_07000015 `the_dead_ladder_never_grabs_at_point_blank` must still settle y 40.2..40.7, and the 21-test list from settle-6-c stays green. Un-ignore `walkreal_0700000a_live::the_west_hall_glide_height_matches_the_game` (and the other two 0a ignores if they cross) when green. Full suite green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-band-settle-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-27T08:08Z
