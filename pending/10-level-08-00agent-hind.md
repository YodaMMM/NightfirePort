## Your task: `level-08-00agent-hind`, Phoenix Fire can be walked to the end on 00 Agent

Branch from origin/master. walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent reaches the Hind fight but Bond arrives at 15 health: the upstream bleed is 23 hits for 100.0 (the walkway pair 36.0, then the stairwell/descent fights). Read the ignore note. Cut that bleed with real tactics (cover, head shots, restocks, engagement order) so the Hind fight starts with a margin, and finish the mission. Real input only, no top-ups, never lower damage or health; new flags opt-in, default off. Un-ignore it when green. Run the whole walkreal_07000008_live binary and interactables_07000008_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-08-00agent-hind`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
