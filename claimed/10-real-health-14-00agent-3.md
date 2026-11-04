## Your task: `real-health-14-00agent-3`, get 07000014's Agent and 00 Agent walks past the duct on real health

Continue origin/s/real-health-14-00agent-2: branch from it, merge origin/master in, and read docs/reports/real-health-14-00agent-2.md there plus the report it continues. The Agent walk with its real-damage wrap is still red, so on master it runs without the wrap, and the 00 Agent walk is #[ignore]d. The re-land is one cherry-pick away (`git cherry-pick 3c011896 71d0651c`). With it, the crossing lands and a crouch toggle clears the lid, but Bond jams at the mezzanine's west lip, stuck at (45.13, 3.05, 39.87), and the standing capsule turns five green walks red. That report ties both jams to the live-push versus dead-tail divergence [M Intersect_CylGeom 0x109da0:447, the +0x50 DeltaP lifetime guard]. Settle that first: read what the game does at that guard and port it, since a real difference there affects every walk, not only this one. NF_WALK_CLIP=1 and NF_WALK_GHOST=1 both broke earlier legs, so don't take either as the fix without knowing why. Then finish the legs after the connector. Keep every walk green in walkreal_07000014_live, wrap the Agent walk in `with_real_damage`, and remove the 00 Agent ignore when green; if it must stay, give the measured reason and the next lever. Real input only: no health, ammo or channel writes, and no teleports. Never lower damage or health to make it pass. Fix the port only where it differs from the game, with the address. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in. Run walkreal_07000014_live and nf-game, then every walkreal_*_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-14-00agent-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-04T15:44Z
