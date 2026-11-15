## Your task: `real-health-00agent-08-bleed-7`, 07000008's 00 Agent walk wins the Hind fight

Branch from current origin/master.

Master c65f36857: five rockets land (the Hind 489.6 of 598, ONE short) and Bond dies at t33393 with 8.95 left (fight t~6140). Measured: a held-health run of the fight (env probe: hold health at the fight loop top) kills the Hind at fight t10131 - the next good launch after rocket 5 comes at fight t8391, so ~2250 more ticks / ~20-30 more health are needed. The fight bill: rocket 2's own blast (-25.05, the overhead pass at 16 u - every rule that drops rocket 2 re-rolls to three rockets), nine cannon chips (-2.0), a radius-4 blast. The gondola ride costs 12.3 armour standing (crouched it is kept, but the fight re-rolls worse); the east deck head fight costs 40.0 (t684-1114, four 0xa guards at 4-10 u). Goal: the walk's 00 Agent test passes.

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

**Branch:** `s/real-health-00agent-08-bleed-7`.

---
claimed-by: sandbox3 at 2026-11-15T07:10Z

---
## Report (done, 2026-11-15T08:38Z)


---
## Report from the worker (done, 2026-11-15T08:38Z)

