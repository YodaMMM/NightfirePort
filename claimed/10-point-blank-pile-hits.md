## Your task: `point-blank-pile-hits`, rounds at 1 unit into a pile of guards land almost nothing

On origin/s/enemy-stats-3's 07000003 full-route run, 48 player rounds fired at about 1 unit into four guards standing on one spot landed almost no hits. A player firing point-blank into a group should drop them. Find what the game does for a player round at that range [M Bullet_init 0x12c5a0 (the start point vs the muzzle/eye, the near clip), Bullet_update 0x12e7b0 -> Collide_Pick 0x1eb320 / Collide_PickObj 0x1eae80 (drone bounding spheres, overlapping drones, the first-hit rule), the drone hit-boxes], reproduce with four guards held on one spot on 07000003 (or a synthetic pile), fix the port gap with the address cited, and pin: N rounds at 1 u into the pile land the game's hits (sister: at 5 u). Run the bullet/impact/drone-damage binaries, walkreal_07000003_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/point-blank-pile-hits`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-01T08:10Z
