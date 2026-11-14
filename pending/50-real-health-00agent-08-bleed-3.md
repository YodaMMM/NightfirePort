## Your task: `real-health-00agent-08-bleed-3`, 07000008's 00 Agent whole-mission real-damage walk wins the Hind fight

Branch from current origin/master and read the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent in crates/nf-view/tests/walkreal_07000008_live.rs, plus docs/reports/real-health-00agent-08-bleed-2.md on origin/s/real-health-00agent-08-bleed-2 if it has one. The descent is cleared and the Hind fight is reached; rocket 1 lands, but Bond dies around tick 32622 to the machine. Take the next levers the ignore message names for the Hind fight (dodge timing, cover between volleys, the remaining rockets' pacing), judging each on its own real run. Helpers stay gated to 07000008 and difficulty 3, so the other difficulties' walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if still red, push the ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary and interactables_07000008_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-08-bleed-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


11-13 (master eb89478f5): on the current master the walk wedged BEFORE the Hind - the "to the briefcase" nav leg from the bolt stand (z -117.6) ran Bond into the safe-face corner (162.59, -118.17) for the cap (ch 0x4a never wrote, t26460). Fixed by starting the leg from the plain walk spot (158.93, -116.52); the walk now dies at t34277 at (110.98, 13.52, -179.31). The Hind levers in this task stand.

Measured 11-13 (master 42aff3fcb): the d3 bleed is at the START - the station hall guards take Bond 96 -> 12 on "the walkway head nav 2" (t977-1916: #11 32.0, #3 44.0); the mouth stand sees nobody at t373 (they wake on the plate and stay hidden). A held mouth stand (poll every 30 ticks for a visible guard, 300/900 ticks, radius 25/35) fights nobody; the 900 hold reaches the hall exit at 48 instead of 20 but re-rolls the descent - deaths t25621 (300) / t26960 (900, the lower level east, #19/#20/#23 0x15) vs t34277. Not shipped. Next: fight the hall guards from cover as they come round, or cross the hall on a different line.
