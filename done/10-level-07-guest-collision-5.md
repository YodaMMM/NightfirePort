## Your task: `level-07-guest-collision-5`, guests solid to Bond: the per-cause fixes from round 4

Start from `origin/s/level-07-guest-collision-4` (merge it, then origin/master). Its report (docs/reports/level-07-guest-collision-4.md) root-caused the six reds; work its "What is left" plan in this order:
1. The undrawn raw slide's boundary consumer [M Drone_Control 0x139268:76-83, DroneMove_SetBoundryFlags 0x14f140, DroneAnim_PreTransCallback 0x13b490:120-135] AND the game's route stall swapping the walk danim for the stall anim (so the slide stops with the route) — find the state the game uses so both the 01 shoji crossing and the 07 doorway leg are right.
2. The explosive broadcast (msg 0x21): the port sends it on any grenade Bounce with a bare LOS; the game requires the fear props, the radius and DroneVision_ObjectVisibility >= 0.7 [M 0x1435f0:30-44, 0x1781a8]. Port the gate; this is what scares the 01 contact into CIVILIAN_HIDING.
3. The full push for every capsule-ON bot [M 0x149ec8:11-34, 0x1ec180:229-266] (not team 3 only), the walker dodge mirroring it, and the walks taking the player's way (07 hall NW corner past the two team-1 CivilianDoorGuards; the 09 cab mouth — capture the pusher set with NF_PUSH_DEBUG first).
4. armour_0a and speech_facial per the report.
Each fix is its own commit with its [M] cite. Run the FULL suite on the final merged tree (foreground slices are fine, but cover every binary) and report the total line; nothing red that is green on master.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-guest-collision-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T14:03Z

---
## Report (done, 2026-10-25T15:30Z)

Closed by the driver: the report is on `s/level-07-guest-collision-5` (docs/reports/level-07-guest-collision-5.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-25T15:30Z)

