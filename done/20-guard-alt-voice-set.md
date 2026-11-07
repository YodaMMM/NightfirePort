## Your task: `guard-alt-voice-set`, every second guard speaks from the game's alternate line set

The port's `hit_talk_extra` in crates/nf-game/src/drone.rs always uses the base set 0x133 and says `+0x108 & 1` has no port source, but the source is known: `Drone_SM_InitObject` 0x175838 stores a running serial there (`DAT_00299090` + 1, per object init). Six talk sites read its low bit to pick a second line set: `NDrone2_AttackTalk` 0x13eac0:86, `NDrone2_HitTalkExtra` 0x13ed38:49, `NDrone2_SeenPlayerTalk` 0x13f1d0:66/77/88/96, `DroneFunc_InjuredTalk` 0x13fa20:45 and `DroneFunc_RunForCoverTalk` 0x13fd08:45. First confirm each site reads the same record `Drone_SM_InitObject` writes (the base pointer, in the disassembly), find where the counter resets, and list which objects take a serial in the game's init order, because anything that takes one shifts every later guard's parity. Give each guard the game's serial, then fix all six sites in the port, comparing each with the decompile. Remove the "stands in" comment and the out-of-pass/fail note. The arm changes only the line, so the count of Rand draws must stay the same. Read a guard's serial on one level live only if the order cannot be settled from the decompile, and list that as a live check for the main agent. Pin through real play on 07000014: a guard with an odd serial speaks from the alternate set and one with an even serial from the base set. Run the voice binaries (grep tests for `talks` and `hit_talk_extra`), the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-alt-voice-set`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-06T23:37Z

---
## Report (done, 2026-11-07T00:14Z)

