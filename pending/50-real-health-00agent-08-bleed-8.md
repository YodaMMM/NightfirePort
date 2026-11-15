## Your task: `real-health-00agent-08-bleed-8`, 07000008's 00 Agent walk wins the Hind fight

Branch from current origin/master.

Master c65f36857+: five rockets land (the Hind 489.6 of 598, ONE short), death t33393 with 8.95 left at fight t~6140. Read done/50-real-health-00agent-08-bleed-6.md, -bleed-7.md for every measured lever. The held-health analysis (hold health at the fight loop top) kills the Hind at fight t10131; the next good launch after rocket 5 is at fight t8391, so ~20-30 more health into the fight (or ~2250 ticks of survival) is the gap. Unmeasured ideas: the walker laser_dodge / walker_b2 round_dodge pattern (an opt-in tick_inner override that steps away from incoming explosive rounds only when no hostile is close) applied to the Hind missiles (kind 98, radius 4) outside the launch windows; the east deck head fight (40.0 at t684-1114 to four 0xa guards at 4-10 u) fought from a different stand. Goal: the walk's 00 Agent test passes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.


- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-08-bleed-8`.
