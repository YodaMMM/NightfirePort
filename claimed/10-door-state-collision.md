## Your task: `door-state-collision`, open doors stop blocking the walk

Land ONLY the door-state filter from `origin/s/capsule-cull-and-los-masks-5` onto origin/master, on its own branch: drone.rs `set_dynamic`'s `script_parts` doors extend filtered by `x.is_closed(&drones.doors.group_states)` — the game's `Door_IsClosed` 0x1367b0 rule (kind-1 doors read closed, grouped doors `DoorGroupsStates[g] == 2`, the rest `obj+0xF4 == CLOSED` exactly; OPEN/OPENING/CLOSING leaves do not block) [M door.rs `Door::is_closed`, the zone consumer 0x135738/0x1336f8], plus its tests and the SS parts review (safe door #205, bolt #217). None of that branch's cull/LOS/gt.rs changes. walkreal_07000008_live must be 4/4 green and every other walkreal / door-family / mp test green (master's mp_net_slots_post_move_live digest must NOT move) on the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/door-state-collision`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-26T23:19Z
