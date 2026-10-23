## Your task: `civilian-first-sighting-alert`, a civilian who spots an armed Bond raises the guards once

Branch `s/civilian-mission-wait-2` (task 962, merged in batch 1022ab) ports `NDrone2_DSTATE_CivilianMissionWait` 0x15dbb0 but leaves out case 3's first-sighting alert. The decompile [M 0x15dbb0 case 3, lines 39-60]:

- `NDrone2_NearArmedPerson(param_1, 10.0)` == 0 → `NDrone2_BondTalk` and return 1.
- Otherwise: +0x4f8 |= 0x8000000. If `behaviour_util_getProperty(0x33, +0x4d8)` is set AND +0x4f8 & 0x200000 is clear (the once-only latch): +0x4fc |= 1, `NDrone2_DroneAlertToObject(dc, 0x11, NDrone2_Player(), 20.0, 20.0, 0.76)` (floats 0x41a00000 / 0x41a00000 / 0x3f428f5c — check the argument order in the disasm), then +0x4f8 |= 0x200000.
- Follow the rest of the case past line 60 yourself and port what it does.

1. Port the alert in the port's CivilianMissionWait (drone.rs): property 0x33, the latch, the 0x4fc bit, the DroneAlertToObject call through the port's existing alert path (find it; cite how the 0x11 alert type is handled by the receivers).
2. Tests: a civilian with property 0x33 seeing an armed Bond within 10 alerts the nearby guards ONCE (the latch: a second sighting does not re-alert); sister: without property 0x33 there is no alert; sister: an unarmed Bond → BondTalk, no alert.
3. Name a real level/civilian that carries property 0x33 [D placements] and add a real-path test there if one exists.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/civilian-first-sighting-alert`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-23T15:24Z
