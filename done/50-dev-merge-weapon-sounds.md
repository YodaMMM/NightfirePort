## Your task: `dev-merge-weapon-sounds`, the dev-merge foreign weapons: fire and the laser loop sound

`crates/nf-view/tests/fix_weapon_sounds_live.rs`: `a_foreign_weapon_fires_with_sounds_on_the_indoor_level_07000001` (the foreign AR draws but held fire spawns no round) and `the_laser_loop_starts_on_press_and_stops_on_release` (the Laser 0x4e has no first-person model in the dev merge) are `#[ignore]`. Port the missing pieces of the dev asset merge so both pass.

Rule: real input only (`step_bond` movement/aim/fire/use; no teleports past the blocker, no channel or state pokes). When it stops, find what the game does there [M]/[D] and port that, never a port-side shortcut. Un-ignore each test when it passes, with a sister that can fail.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/dev-merge-weapon-sounds`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-24T13:32Z

---
## Report (done, 2026-10-24T14:17Z)

Closed by the driver: the report is on `s/dev-merge-weapon-sounds` (docs/reports/dev-merge-weapon-sounds.md); the worker didn't run finish.sh.
