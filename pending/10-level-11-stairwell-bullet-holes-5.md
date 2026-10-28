## Your task: `level-11-stairwell-bullet-holes-5`, Phoenix Fire Part 1's stairwell: no bullet holes in mid-air

On 07000011's stairwell the player still sees bullet holes hanging in mid-air (rows along the railing level, a cluster on a plane in the well, more looking up) and invisible walls eating shots. Four earlier rounds narrowed it to collision geometry the bullets hit that the game's bullets pass (0x40-material well panes etc.). The per-caller LOS/bullet masks [M Collide_Intersect 0x1ebaa8:300-319] are being finished on `s/capsule-cull-and-los-masks-11`; read rounds 1-4's reports first. Find, for every mid-air hit the stairwell produces, which collision face the port's bullet ray stops on and what the game's bullet does with that face [M Bullet_init/Bullet_Update's ray mask, Collide_FilterBullets 0x1ece30, the surface byte drops], and fix the bullet path so impacts land only where the game's do. Test: fire across and up/down the well from the measured spots and assert every impact lies on a drawn surface. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-stairwell-bullet-holes-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

