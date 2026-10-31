## Your task: `capsule-cull-and-los-masks-21`, the last red: the frozen wire ride

Continue `origin/s/wip-capsule-21` (merge `origin/master` first; read `blocked/20-capsule-cull-and-los-masks-20.md` and -19 on the queue branch). The faithful capsule rules (raw-denom head-on cull [M 0x10a1f0-0x10a2e8], Dist2Tri flags 0/99 skip [M 0x10a330/0x10a33c]) are kept; the only red is `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff`: on the #283 wire ride the WHOLE bond tick stops at t~730-780 (pos frozen, auto_drop frozen while mode prints 7, the jump release never runs). A frozen tick with the wire state intact points at the player being disabled/skipped: check `sim.disabled`, PlayerDisable script events (NF_SCRIPTLOG=1), scene scripts or triggers near (36.1, 13.5, -209.5) [D; M Script_EventHandler 0x1d94c0 events 4/5], and Sim::step_bond's gates. Fix with evidence; land only with the 13 binaries named in blocked/20-capsule-cull-and-los-masks-18.md and every walkreal_* binary green. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-21`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

