## Your task: `level-1b-drake-4`, 0700001b: the scripted carry into the shafts, then Drake

On master the zero-G float works, and the wall at x 4.433 (y -16.56..+16.74) is the game's own plc #128 OBJECT_Boundary_1 (body-solid, bullet-pass) [D probe1b_wall]. The shaft mouths are two booths at (5.23, 9.45, +/-8.82): a glass pane (mat 0x40, bullet-pass) at x 5.91 and iris doors plc #30 OBJECT_IrisDoor1 / plc #32 OBJECT_IrisDoor3 (kind 0x2031). `walk_arrival_to_drakes_defeat` and `drake_defeat_by_real_fire_ends_the_mission` stay ignored; `probe1b_wall_and_mouths` and `scan_the_x5_wall_openings` are assertion-free dumps (ignored by design).

1. Find the mechanism that carries Bond past the boundary into a shaft: the ch-74 pulse is the candidate; the iris doors' opening channels; any script stream with a player-carry event [M Script_EventHandler 0x1d94c0]. Port it.
2. Then the landing, the fight gate, the defeat, real input only, and un-ignore both tests.
3. The interactables kill-channel gap: the walker-driven kill leaves ch 0x0e off while a raw step_bond kill raises it; find why (the same walker-shoot difference may affect 07000002's computer).
4. If a live read is required, name the address and the moment (fresh boot) and stop there.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-1b-drake-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-10-24T06:08Z
