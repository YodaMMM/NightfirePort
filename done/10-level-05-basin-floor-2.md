## Your task: `level-05-basin-floor-2`, the valley's hurt planes are solid without breaking the plunge death or the door-band walk

Branch from origin/s/level-05-basin-floor and merge current origin/master; read docs/reports/level-05-basin-floor.md. That branch makes ungated kind-0xE4 hurt planes solid collision [M Hurt_Create 0x17e870:51-57, Collide_Pick 0x1eb320:337], which is right for the east basin, but its own level's binaries go red with it: interactables_07000005_live::the_valley_walkoff_runs_the_plunge_death_nis_chain and walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked_real_damage (the round ran only its new test). Find what the new solid planes change on those two paths (does Bond now land on a plane before the PlungeTrigger at y ~13.17 fires, or does the plunge chain read a fall the floor now stops?) and fix it the game's way; read the plunge trigger and death NIS code before deciding. If the game's answer needs a live capture, write the exact question in the report and keep going. Also port the gated planes' solidness when their channel rises (the round's open item). Run walkreal_07000005_live and interactables_07000005_live whole, then every walkreal_*_live and interactables_*_live binary one level per tool call (the change touches every level with an ungated hurt plane).

**Live fact (PCSX2, 2026-11-10):** Bond placed just past the east lip at (132.0, 14.4, 126.6) settles at y 14.15 (the PlungeTrigger plane is y 13.17) and the game plays the plunge death and shows Mission Failed. The game does NOT let Bond walk down from the lip into the basin, so `the_valley_plane_hurts_bond_walked`'s route is impossible in the game: delete that test (cite this), and pin the lip as a plunge death instead if the plunge test does not already cover it. Keep the solid hurt planes only if they are game-correct AND keep the plunge test green; otherwise drop the static fold and say why.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-05-basin-floor-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-10T10:42Z

---
## Report (done, 2026-11-10T11:26Z)

