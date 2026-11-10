## Your task: `level-15-agent-rim-shot`, Equinox can be walked to the Omega sector exit on Agent

Branch from origin/master. walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_agent is ignored: since blasts use the game's straight falloff to Bond's sphere, the #526 rim-shot costs 44.41 at frame 5206 (was 27.73) and the door fight finishes the 100 health. First confirm the 44.41 is the game's number [M the blast falloff]. Then change the route so Bond is not in that blast (take #526 from range or another angle, or kill it before it fires) and the door fight is won with a margin. Real input only, no top-ups, never lower damage or health. Un-ignore it. Run the whole walkreal_07000015_live binary and interactables_07000015_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-15-agent-rim-shot`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-10T10:09Z
