## Your task: `real-health-00agent-08-bleed`, 07000008 can be walked to the Hind fight on 00 Agent with health to win it

Branch from origin/s/level-08-rocket-self-blast, merge current origin/master, and read its docs/reports/level-08-rocket-self-blast.md plus the 00 Agent ignore message. The guided rocket is proven equal to the game, and the walk now reaches the Hind fight, but it arrives at 15 health and dies to the machine's own blast. The blocker is the bleed on the way there: 23 hits for 100.0, with the walkway pair taking 36.0 from 53-105 u and then the stairwell and descent fights. Re-tune those fights on the current master tree (stands out of the pair's line, fight order, cover) so Bond reaches the Hind with health to win it, then finish the fight with the levers the level-08-hind-fight branches proved. Helpers stay gated to 07000008 and difficulty 3, never lower damage or health, and keep the rocket count the walk really carries. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000008_live binary, the copter and homing binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-08-bleed`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-10T02:57Z
