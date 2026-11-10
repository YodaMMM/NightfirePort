## Your task: `los-mask-repins-08-agent`, Phoenix Fire's Agent real-damage walk green under the game's guard sight

Branch from origin/master (the Operative real-damage walk is green there since los-mask-repins-08). Read the ignore message on walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_agent and docs/reports/los-mask-repins-08.md on origin/s/los-mask-repins-08. Take its levers so the Agent walk passes with a real margin, and un-ignore it. Real input only, never lower damage or health, no top-ups; helpers gated to 07000008 and difficulty 2. Run walkreal_07000008_live and interactables_07000008_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-los-mask-repins-08-agent`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
