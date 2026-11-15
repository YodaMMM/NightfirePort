## Your task: `real-health-00agent-14-landing-5`, 07000014's 00 Agent infiltration walk reaches the delta exit

Branch from current origin/master.

Master fcd3ca397: the escape touch is reached at 26.6 health; the walk dies at t41509 at (-15.85, 1.13, -22.71) on the Rook legs. Read the landing-4 ignore ledger on master. Goal: the walk's 00 Agent test passes.

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

**Branch:** `s/real-health-00agent-14-landing-5`.

---
claimed-by: sandbox3 at 2026-11-15T09:20Z

Main 11-15 (on fcd3ca397): the escape-room armour (-27.3, 0.9, 31.0) right after the touch (now reached at 26.6): death at the room door t40134 - worse than t41509.
