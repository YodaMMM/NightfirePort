## Your task: `level-1b-drake-3`, 0700001b: from the zero-G float to Drake

Start from `s/level-1b-drake-2` (merge it into a fresh branch from master; note master already carries the 0x4b zero-G entry, gated to 0700001b). That branch maps the float to a capsule stop at x 4.98 across z -8..0 (a glass face at x 4.43) and leaves both 1b tests ignored at that wall.

1. The wall: list the triangles at x 4.43, z -8..0 with their owning object and flags. Decide from the game's rules whether the player's capsule collides with them [M Collide_Intersect / the pick admission / the glass flags]. A teleport cannot test this live: outside its cel graph the game freezes the player.
2. Map the shaft mouths at z +/-8.82 and how the route enters SP #141/#142 (from the deck or from the float), then the landing, the fight and the defeat, real input only.
3. Un-ignore the walk and the interactables test with sisters. If a live read is required, name the address and the moment (fresh boot, during the slit crossing) and stop there.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-1b-drake-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

