## Your task: `level-0c-vent-pulley-class`, port the 0xE2 vent/pulley and walk the display-eye leg

level-0c-real-crossings-2 (merged) leaves the 0c whole walk green with stand-ins; one is the display eye + the plates reach, carried because the 0xE2 vent/pulley class is not ported (the pad riser is capsule-blocked [D probe]). Port the 0xE2 class from the decompile [M its create/update fns; D the 0c placements], then replace that stand-in in `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` with the real-input leg. The walk must stay green. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-vent-pulley-class`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T06:10Z

---
## Report (done, 2026-10-30T08:59Z)

