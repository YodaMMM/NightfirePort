## Your task: `level-13-warm-chain-margin-2`, Chain Reaction can be walked from the warm chain out of the front entrance on real damage

Branch from origin/s/level-13-warm-chain-margin, merge current origin/master, and read docs/reports/level-13-warm-chain-margin.md. Round 1 bisected the death to 7876b7feb (the 12 keypad now takes its typed code, which re-times the chain) and proved the new bleed is game-correct: at frame 1647 a shotgun guard (skin 0x5000008, weapon 0x1c) from the spawner wave on the lobby's east lane unloads 7 pellets for 36.5 at d 2.6. Re-route the lobby leg so Bond is not on the east lane when that wave lands (wait out the wave from cover, or take the wave first from range), then finish the walk with a real margin and un-ignore walk_the_warm_chain_to_the_front_entrance_and_out_real_damage. Real input only, no top-ups, never lower damage or health; helpers gated to 07000013. Run walkreal_07000013_live and interactables_07000013_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-13-warm-chain-margin-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-10T11:10Z

---
## Report (done, 2026-11-10T12:30Z)

