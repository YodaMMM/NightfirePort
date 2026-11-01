## Your task: `level-0a-ceiling-gun-route`, 0a: getting past the west-hall ceiling gun on real health

On 0700000a laser #607 in the west hall feeds OR #667 -> ch 0x0F, ceiling gun #640's enable (at -22.1, 2.3, 17.2); since the GT_Update case-3 fix [M 0x180498] the gun re-arms and two bursts (27 + 15 rounds x 2.4) outlast Bond's health; only explosions damage a turret (GT_ApplyDamage 0x1816d8 <- Explode_Propagate 0x17c518) and 0a's kit has none. `walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch_real_damage` is ignored for this. Find how a player gets through: avoiding #607 (crouch under / step over — the laser row's height vs Player_Collision 0x194df0's 1.86/1.3 body), a route around it, the gun's cone/LOS, and whether its rounds spread (Bullet_init 0x12c5a0's spread term for owner -1 rounds — the port fires them dead-on). Fix any port gap with the address, make the real-damage walk take the player's route and un-ignore it. Run every *0700000a* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-ceiling-gun-route`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-01T10:10Z
