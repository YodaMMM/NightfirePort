## Your task: `guard-push-every-tick`, the guard push on Bond every tick (03 flank fall)

On 07000003 Bond falls out of the courtyard: walking into two guards builds an overlap the port only discharges while Bond is airborne, so on the first airborne tick (lifted 0.04/tick by the gatehouse wall's south end-edge) ~0.74 u of push lands at once and carries him across the wall plane at x 10.915 to x 11.46 with no floor (traced on origin/s/floor-seam-2; `flank_fall_repro::the_old_courtyard_flank_stays_on_the_floor` is ignored for it). The game applies the drone-player push every tick (Collide_Jointy 0x1ec180, no ground gate) behind per-guard collision gates [M NDrone2_DoCollision 0x149ec8: +0x4f8 bits 0x20/0x200, +0x570 bit 2]. Port the gates and the every-tick push, then fix whatever walks move only where the game really differs (walkreal_07000003 x2, walkreal_07000001 the_plate_refuses_bond_alone, exit_aim_01 went red with a naive every-tick push — find which gate keeps them as they are). Un-ignore the flank test. Run every walkreal_*, exit_aim_01_live, the mp_* digest tests and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-push-every-tick`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-01T13:49Z
