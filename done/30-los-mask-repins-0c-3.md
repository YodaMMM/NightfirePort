## Your task: `los-mask-repins-0c-3`, Chain Reaction's real-damage handoff walk green under the game's guard sight

Branch from origin/master (since los-mask-repins-0c-2 the plain handoff walk and the east ground route are green: the walk switches to the P2K for the strip-guard shot; the earlier landing shots were Micro-Camera taps). Read the ignore message on walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage and docs/reports/los-mask-repins-0c-2.md on origin/s/los-mask-repins-0c-2. Make the real-damage sister pass with a real margin and un-ignore it. Real input only, never lower damage or health, no top-ups. If you touch tests/common/*, run every binary that uses the changed helper. Run walkreal_0700000c_live, walkreal_0700000c_eastroute_live and interact_0700000c_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-los-mask-repins-0c-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-10T22:53Z

---
## Report (done, 2026-11-10T23:33Z)

