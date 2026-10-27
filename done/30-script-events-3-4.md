## Your task: `script-events-3-4`, script Event 3 (spawn transform) and Event 4 (player disable)

The script player's Event 3 (the spawn-transform bake, `Script_Event` 0x1d94c0:98-116) and Event 4 (`Player_Disable`, :118-121) fall to the port's generic unused arm (nis.rs and drone.rs). On 07000013 the intro 060006d7 fires Event 4 at key 153 (Bond should not be controllable then) and the shutter script 06000959 fires Event 3 twice, so its Entity ops spawn the shutters from the cel transform instead of the baked one [S]. Port both events exactly, then sweep every level's scripts for Event 3/4 users [D dump] and list them in the report. Tests: during 07000013's intro after key 153 player input does not move Bond [M]; the 06000959 Entity spawns use the Event-3 transform [M + D].

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/script-events-3-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-27T01:18Z

---
## Report (done, 2026-10-27T02:53Z)

Closed by the driver: the report is on `s/script-events-3-4` (docs/reports/script-events-3-4.md); the worker didn't run finish.sh.
