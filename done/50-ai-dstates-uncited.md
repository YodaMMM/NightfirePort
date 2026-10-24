## Your task: `ai-dstates-uncited`, the guard AI states the port never names

These `ACTION.ELF` functions are never mentioned (by name or address) anywhere in `crates/*/src`: `NDrone2_DSTATE_Prone` 0x166940, `NDrone2_DSTATE_ProneFire` 0x166a28, `NDrone2_DSTATE_Interogator` 0x1729f0, `NDrone2_DSTATE_Alerted1stEncounter` 0x173178, `NDrone2_DSTATE_CombatTooClose` 0x173338, `NDrone2_DSTATE_HoldItRightThere` 0x173570, `NDrone2_DSTATE_SeenOpponent` 0x173b38, `NDrone2_DSTATE_SeenExplosive` 0x173bb8, `NDrone2_DSTATE_FailMission` 0x173f68, `NDrone2_DSTATE_HangUp` 0x1743d8, `NDrone2_DSTATE_WaitForever` 0x174430, `NDrone2_DSTATE_Tester1` 0x16db90, `NDrone2_DSTATE_Tester2` 0x174150, `NDrone2_DSTATE_Tester3` 0x1741c8, `NDrone2_DSTATE_Tester4` 0x1742d8, `NDrone2_initDTYPE_SniperAlert` 0x14e580.

For each one: find its state id(s) in `NDrone2_StateFuncs` 0x29b300 and who enters it (`Drone_SM_SetState` callers, the mode inits, placement params) [M + D]. Then exactly one of: (a) the port already implements it under another name — add the function name and address at that spot, and a test of its behaviour if none exists; (b) it is reachable on an on-foot level and missing — port it [M] with a test that fails before; (c) it is never entered on the shipped on-foot path — prove it (no caller, no table entry a level reaches) and cite that in `crates/nf-game/src/drone.rs`. `tools/calltree/calltree.py` from each ported state must report no never-mentioned callees.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/ai-dstates-uncited`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-24T21:14Z

---
## Report (done, 2026-10-24T22:39Z)

