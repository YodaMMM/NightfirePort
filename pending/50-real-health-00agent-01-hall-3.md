## Your task: `real-health-00agent-01-hall-3`, 07000001's hall door phase can be won on 00 Agent with real health

Branch from origin/s/real-health-00agent-01-hall-2, merge current origin/master, and read its docs/reports/real-health-00agent-01-hall-2.md plus the 00 Agent ignore message. Take its next lever, the door-phase kill path. Either work out from the decompile which weapon and timing make the hearing chain draw the guards across [M DroneFunc_SetNoiseHeard 0x149920, HandleSoundAlerts 0x148a30] and lure with that, or kill the crowd through the aperture with crouched taps along the flight lane, keeping the ammo pool metered. Do not repeat its measured dead ends. Helpers stay gated to 07000001 and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000001_live binary and interactables_07000001_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-01-hall-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
Main agent, fresh deaths on master 729f9694e (each ignored walk of the level, --ignored --test-threads=1; test names not captured):
walkreal_07000001_live: Bond died at tick 6709 (frame 6738) at (6.62,6.47,9.85) armour 0.0
