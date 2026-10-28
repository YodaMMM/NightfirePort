## Your task: `level-14-delta-exit-walk`, Countdown Part 1 walked from the vent drop to the Delta exit

On 07000014 every mechanism is now on master: the vent-drop hand-back into the facility, the fusebox cuts, the panel deaths, the E-use switch oDroneTrigger1b at (-60.30, 0.61, 38.37) (touch out 0xA0, mask 4 [M Trigger_Activate 0x1b2318]) -> SEQ_AND 0x51 -> the box/generator cascade -> 0x85 -> Door_Slide_11A/B unlock and open [M Door_Update 0x1363c8:88-107]. `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` stays ignored because leg 2's nav route crosses the closed door plane. Give legs 2-3 explicit west-side waypoints, cut both west fuseboxes on the real laser path, walk to the switch and press E (the test `the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door` has the exact input), then legs 4-6 east through the opened door to the Delta exit #944, every leg on real input; un-ignore it (or re-ignore on the next exact blocker with its evidence). Note anything the player would need that the port does not draw (the switch wall art, the M.Hatch indicator) in the report. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-delta-exit-walk`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-28T10:48Z

---
## Report (done, 2026-10-28T12:04Z)

Closed by the driver: the report is on `s/level-14-delta-exit-walk` (docs/reports/level-14-delta-exit-walk.md); the worker didn't run finish.sh.
