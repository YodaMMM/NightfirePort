## Your task: `level-0c-parapet-hop`, the last hop on 0c's plates route, then leg 4 for real

level-0c-plates-reach-2 (merged; `probe0c_pocket_descent`, read its report) mapped the display ground -> 7.24 platform -> rib -> 6.92 walkway -> ramp -> yard -> plates chain on real input except ONE hop: the rib's west parapet (x -4.84..-4.79, top 8.67, z -112.21..-116.70 [D]), which every sim jump fails (the sim's jump apex is ~1.0; the parapet needs ~1.43). Work its "What's left" item 1 in order: (i) scan both parapet ends for a walk-around at 7.24 [D]; (ii) NOTE: the port's standing jump is already EXACT — rise 0.7513, apex at 22-23 ticks, the same per-tick curve as the live game on 0700000a's flat hall [L 2026-10-30 PCSX2, Triangle]; a plain jump cannot clear 1.43 in the game either, so do NOT change the jump; look for a crouch/ledge/mantle arm or another link instead. Old item (ii) for reference: check the port's jump against the game [M Player_Jump / the jump velocity and gravity constants; the crouch-jump and any ledge/mantle arm] — if the port's apex is lower than the game's, that is a port physics gap: fix it with cites; (iii) otherwise find the real link. Then rewrite leg 4 of `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` with the proven chain and no teleport. The walk must stay green; run every walkreal_* binary if the jump changes. Quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-parapet-hop`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T17:19Z

---
## Report (done, 2026-10-30T18:35Z)

