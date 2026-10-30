## Your task: `level-14-door-guards-combat`, 14's released guards must not go to combat unseen

On 07000014, after the fusebox release (ch 0x87 + ch 0x88 -> 0x8C) with Bond held out of sight in the generator hall (-76.8, -1.1, 46.5):
- The game [L PCSX2 fresh boot, 3000 frames]: two guards walk in ENEMY_MISSION (0x1b, sub 0x06) to the west door and STOP at (-51.6, 1.0, 41.2) / (-51.2, 1.0, 38.8), staying 0x1b throughout; their +0x5A6 saved state stays 0; no OPEN_DOOR (0x81) ever.
- The port (master): drone #25 walks 0x1b at alert 0.66, then flips to COMBAT_OUT_OF_RANGE (0x5a) at alert 1.00; drones #23/#24 arrive already 0x5a at alert 1.00. None ever has los / seen / ever_seen. From ~t1230 all three sit at (-51.8, 1.0, 40.0) in 0x81 re-pressing the locked door (door state 2).
Find what raises their alert to 1.0 and puts them in combat without sight in the port, and what the game does instead [M the ENEMY_MISSION DSTATE, the alert writers, the 0x8C release path; D the guards' placement params]. Fix the port side. Add a test that reproduces this setup (load 14, set 0x87/0x88, hold Bond at the hall spot, 3000 ticks) and pins: no drone enters 0x81 or 0x5a, and the two 0x1b guards stop by the door. Run the touched binaries plus every walkreal_07000014 and guard/alarm binary; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-door-guards-combat`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T14:27Z

---
## Report (blocked, 2026-10-30T15:25Z)


---
## Report from the worker (blocked, 2026-10-30T15:25Z)

