## Your task: `level-06-castle-fire`, 07000006: held and repeated fire, then the castle interior walk

`walkreal_07000006_live::walk_the_castle_interior_to_the_upper_gallery` and `stopping_before_the_tunnel_never_ends_the_castle` are ignored on a player-weapon fault the walk measured (read the doc comment above the first test): on the first press the round spawns on the press tick but the view turns -1.38 rad from the aimed yaw (an intra-tick body turn at the fire site, coincident with the fire script's start), and every later press — 20-tick taps, held fire, idle-gated presses, 40-tick taps — spawns nothing (12 presses -> 1 bullet).

1. Reproduce both faults with real input in a small test on 07000006 (the walk's carry and spawn), then find the cause in the fire path [M `Player_SetFiringAnim` 0x1a3c70, `Player_WeaponFiring` 0x1a5460, the view-state machine `Player_SetWeaponAnimObj` 0x1a2e38] against the port's `Gun::update_with` and the fire-script/state handling in `crates/nf-view/src/scene.rs`. The game re-fires a held automatic every interval and a semi-automatic on every fresh press; nothing turns the body on the press tick.
2. Fix it, pin both (press-tick yaw unchanged; N presses -> N rounds for a semi-auto, held fire -> the record's rate for an automatic) with tests that fail before the fix, and check the other weapons' fire tests stay green.
3. Un-ignore the two 06 walks as they go green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-06-castle-fire`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-10-24T15:28Z
