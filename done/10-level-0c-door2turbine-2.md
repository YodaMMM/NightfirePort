## Your task: `level-0c-door2turbine-2`, Chain Reaction Part 1: walk to the door2turbine E stance for real

Read the level-0c-door2turbine report (on master). The door is decoded and its E stance exists (ring SE corner, use reach 2.62); the walker cannot get from the SE walkway onto the 11.61 shelf / 11.09 corridor - it balances on an unexplained ~13.05 surface at x 40.45..40.6 (owner not in the static census: find it - a drone, a dynamic object, a missed collision mesh [D]). Find how the game's geometry connects the walkway to the corner (step, drop, or a connection the port's world lacks) and fix any port collision gap [M/D]; then replace the stand-in with the real-input leg: corner E opens #281, the slot jump onto (44,-209), the band walk. Then continue to blockers 2-5 of the level-0c-every-objective report in order. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-door2turbine-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-29T05:37Z

---
## Report (done, 2026-10-29T06:32Z)

