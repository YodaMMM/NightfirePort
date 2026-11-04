## Your task: `trigger-glist-extents`, parse trigger cel extents where the game places them

Read docs/reports/level-05-truck-class5-contact.md (origin/s/level-05-truck-class5-contact if it is not on master) and the player_touchers notes in crates/nf-game/src/drone.rs (around line 745). The port's parsed glist extents for trigger cels sit off the game's real placement, the ref-gate/extent divergence. 0a's #614 band (y 0.25-0.53) sits about 0.9 under the west corridor's floor (about 1.2), 07000011's #235 about 0.85 over pos, and 0c's ch-27 bay volume is also off. So the standing toucher's deep reach is limited to riding/script plates and a calibrated box top (pos+1.26) stands in for the real reach, which leaves floor plates such as 07000002's bunker plate 0x80 unfired. Find where the game reads these extents and how it applies the ref gate [M], and parse them that way. Pin #614, #235 and the ch-27 volume to their real positions [D]. Then make the deep reach unconditional and retire the box top, if the evidence allows. If an earlier attempt's result matters, s/trigger-beams-3 scopes a related law to beam glists: a whole-trigger law went 21 tests red, and restoring the box left 17, because the segment fired floor plates early. Run the trigger, beam and laser tests, fix_truck_live, walkreal_0700000a_live, walkreal_07000002_live, walkreal_07000011_live, walkreal_0700000c_live and nf-game, then every walkreal_*_live. Real input only: no health, ammo or channel writes, and no teleports. Never lower damage or health to make it pass. Fix the port only where it differs from the game, with the address. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/trigger-glist-extents`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

