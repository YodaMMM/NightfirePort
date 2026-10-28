## Your task: `level-0d-c5-door-laser-2`, finish the C5 door: sustained beam fire drains it to the smash

Continue `origin/s/level-0d-c5-door-laser` (merge `origin/master` first). Done there: the 0x20 gate on Explode_Propagate case 0x21 [M 0x17c518:168-181], the bullet mask filter [M Collide_FilterBullets 0x1ece30:26-33], the hit-colour pair [M Destroy_CollisionHandler 0x12bd48:61-95], one 50-damage contact per round. Left, in order:
1. The sustained-fire defect: with the trigger held and no look input, ~5 contacts land and the health then freezes while the beam quad stays up and the heat climbs (`fired 30`, zero new contacts); with look input held every tick it drains -50/tick. The game drains steadily under a plain held trigger [L PCSX2 2026-10-27: 10000 -> 7100 -> 4050 over two 60-frame bursts with the stick at rest]. Find the port's fault in the no-input fire path (aim easing, the round's spawn origin/direction once the aim settles, the fire arm's re-trigger) against GunImp_Update 0x1320d8 [M], fix it, and un-ignore `the_beam_drains_the_c5_door_at_50_a_tick_and_the_smash_fires_at_zero` (its asserts: the death at -50, bigdoor01 dead the same tick at 1000, ch 0x1f, objective 6 -> state 4).
2. The destroyed-glist swap's draw: expose the dead/destroyed state so the static instance stops drawing (the C5 leaves have no LOD variant [D lod_word -1], so the doorway ends open [L]); list the draw hook in the report for the main agent's render if it touches drawing code.
Run the FULL suite and quote its total line - the master tree today carries a one-contact smash that this branch replaces, so every 0d test must be green on the merged tree.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-c5-door-laser-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

