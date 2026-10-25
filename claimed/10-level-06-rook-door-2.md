## Your task: `level-06-rook-door-2`, 07000006: a guard parks in the courtyard doorway — land the attack-mode fix

Start by merging `origin/s/level-06-rook-door` (evidence only, no port code): its `docs/reports/level-06-rook-door.md` and `docs/systems/rook-door-06.md` hold the proven chain. In short: on 07000006 a switch-16 guard beside door #47 (36.50, 40.00, -16.25) takes RUN_FOR_COVER through the doorway, re-enters ATTACK, and the port's ATTACK-ENTER `ChangeToAttackMode` flush flips him to mode 25 (stationary sniper, SniperAim 0x2a) — he parks in the doorway holding the door open. `NDrone2_ChangeToAttackMode` 0x14a648 has five callers [M] (0x14ce68, 0x14d45c, 0x15da54, 0x15e35c, 0x165948); `NDrone2_DSTATE_Attack` 0x164210 and `DroneFunc_FirstAttack` 0x148010 are not among them. `GetDroneTypeAttackTypeFriend`'s tail 0x14cea8 writes the attack-fold result into +0x5a4, which the port never does.

Removing the ATTACK-ENTER arm fixes 06 but turns `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` red (the contact dies at round 10, state 0x47): the 01 walk only held because the wrong flush parked the corridor guards as snipers.

1. Port the +0x5a4 fold write-back [M 0x14cea8] and remove the ATTACK-ENTER arm (flush only at the game's real call sites).
2. On 07000001, identify the guard that kills the contact (dtype/kind/alt_state, the [M] behaviour he should have) and fix what the game does there with real input — never by keeping the non-game flush.
3. Un-ignore `rook06_realpath_live`; it and the 01 walkreal walk both green, with every rook_*/fix_06_*/escort_* test still green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-06-rook-door-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T02:00Z
