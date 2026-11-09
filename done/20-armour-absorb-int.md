## Your task: `armour-absorb-int`, armour absorbs damage with the game's rounding

The level-0d-agent-real-damage report (origin/s/level-0d-agent-real-damage, docs/reports/level-0d-agent-real-damage.md) found that the game truncates the armour absorb to `(int)damage`, and `PlayerHealth::apply` in crates/nf-game/src/bullet.rs does not. That makes no difference for whole-number damage like 28 or 42, but it does for fractional damage. Read the game's player damage and armour absorb in full, with addresses, and every path that reaches it (bullets, blasts, melee, falls, gas, hurt volumes). List which weapons and hazards deal fractional damage on the single-player levels [D], and fix the port's rounding in the game's order for every path that differs. Check the multiplayer `apply_pain` in mp/combat.rs against the same rule. Pin through real play: Bond with armour takes a fractional hit and both his health and armour match the game's math. Repair any walk this shifts, gated to its level, never lowering damage or health. Run the health and bullet binaries (grep tests for `PlayerHealth`), every walkreal_*_live one level per tool call, every mp_*_live binary and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/armour-absorb-int`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-09T05:00Z

---
## Report (done, 2026-11-09T05:56Z)

