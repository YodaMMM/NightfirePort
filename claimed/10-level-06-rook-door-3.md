## Your task: `level-06-rook-door-3`, the doorway guard fix on 06 without losing the 01 contact

Start from `origin/s/level-06-rook-door-2` (merge it first). It removes the port's ATTACK-ENTER `ChangeToAttackMode` flush (no game caller [M — the five jal sites of 0x14a648]), which fixes the 07000006 guard parked in door #47's doorway, but `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` goes red: the contact (drone #1, skin 0x0500004f, team 2) dies in round 0 of the escort loop once the corridor guards stop parking as stationary snipers. Its report (docs/reports/level-06-rook-door-2.md) lists the candidates.

1. Attribute the kill: record the contact's attacker at the death tick (the shooter's index, dtype/kind/mode/alt_state, the bullet origin) and compare that guard's behaviour with the game's [M — his placement, his DSTATE chain, the mode-2 guard at (-29.73, 10.22, 13.66): does he leave 0x29/0x2a and fire into the hall in the game?]. If the killer deviates from the game, fix the sim [M]; if the kill is game-faithful, change the WALK to play it the way a player does (clear the round-0 guards first) with real input only.
2. Both 01 walkreal tests green with the flush still removed; un-ignore `rook06_realpath_live` and run it green; every rook_*/fix_06_*/escort_*/fix_mission test green.
3. Run the full suite on the final tree merged with origin/master and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-06-rook-door-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T03:08Z
