## Your task: `walker-sidestep-decay`, a walk's sidestep never carries across a ride or grab

Branch from current origin/master, then merge origin/s/los-mask-repins-0c-3 and origin/s/los-mask-repins-12-2. Read both reports (docs/reports/los-mask-repins-0c-3.md and docs/reports/los-mask-repins-12-2.md). Both found the same stale sidestep carry in the walker, on 0700000c and 07000012, each patched for its own level. Replace the two level patches with one walker change: the sidestep decays, and resets when a ride or grab scope begins or ends. This is walker-only, no sim changes. Run every walkreal_*_live one level per tool call and repair any walk that shifts, gated to its level. Real input only, no top-ups, never lower damage or health. Run every walkreal binary and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/walker-sidestep-decay`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-11T05:57Z

---
## Report (done, 2026-11-11T06:51Z)

Closed by the driver: the report is on `s/walker-sidestep-decay` (docs/reports/walker-sidestep-decay.md); the worker didn't run finish.sh.
