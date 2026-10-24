## Your task: `level-4a-lift-4`, 0700004a: the lift ride's timed satchels, defused with the laser

How the ride works in the game: during the lift descent the guards drop timed satchel charges onto the lift; each one must be deactivated by burning it with the laser watch (0x4E), or its timer fails the mission.

Live [L fresh boot, PCSX2, god mode, logs in hand]: the ride descends at 0.0303 units/frame; guards shoot Bond on the way down (health 100 -> 69.4 by y -53); the lift stops at y -54.1971 for the landing scene's disable (575 ticks); the hand-back lands at the marker's raw x/z (1.7994, -3.4534) and the lift resumes; walking north at the hand-back is blocked at z -1.85 in the game too (so that is NOT the exit); with no defusing, **ch 0x62 rises at y -95.85 with health still 100** and the mission fails — the fail is the satchel timer's channel write, not damage. 

The branch `s/level-4a-lift-3` is NOT on master. Its nis.rs change (skip the script-base compose for hash-0 markers) broke exit_aim_01_live, interact_07000001_live (the armour pickup, the entry door), walk_0700004a_live (the descent plates) and fix_level_chain_live (the 4a cutscene movie code) — do not carry it over without fixing those. Its probes and its [S] kill zone at -95.80 are also off master. Its findings (the 575-tick landing-scene disable, the stop height) are in the facts above.

Seen live [L fresh boot screenshots]: after the re-attach, by frame ~3582 (Bond at (1.799, -73.418, -3.453)) a satchel lies on the lift roof showing a green numeric countdown, with bullet sparks around it; the mission fails at ~-95.86 when nothing is defused. The objective shown on the descent is "Escape through the elevator shaft".

1. Find the satchels in the data and decompile: who drops them (drone weapon/drop on the ride's spawners, or script entities), the object class and its update (the timer, what it writes on expiry — ch 0x62 or the mission-fail path [M Mission_Update 0x182c60]), and the laser defuse (likely the same burn as `Lock_Update` 0x1ae3d8's 0x4E arm the port already runs for hinges and bolts — cite it). Port them; remove the [S] kill zone.
2. Tests, real input only: ride the lift, burn each satchel with the laser watch as it lands, and finish the ride alive into the lobby (whatever the real exit is once the satchels are defused); sister: an undefused satchel fails the mission when its timer ends.
3. The probe tests (walkreal_probe_l3*) are assertion-free dumps; keep them ignored.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-lift-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-24T02:52Z

---
## Report (done, 2026-10-24T04:16Z)

