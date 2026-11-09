## Your task: `los-mask-01-walk`, the 07000001 walks stay green with guards seeing through the game's line-of-sight masks

Branch from origin/s/los-mask-envelope, merge current origin/master, and read its docs/reports/los-mask-envelope.md. With the masks on, guards see Bond through material-0x10 glass, node-pass cels and 0x40/0x80-material surfaces as in the game, and `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent` in walkreal_07000001_live (d2) dies at tick 8977 (guard #4, 10 hits for 11.1). The first divergence is the t678 guard-#8 fight (100 to 93.7 against 100 to 100 before), and the ambush phase bleeds Bond out from there. Re-tune that walk against the masked vision with the walker's own levers (keep_off, back_off, finish_fights, the fight-site choices), gated to 07000001, never lowering damage or health. Then run every other walkreal_*_live one level per tool call and repair any walk the masks move, gated to its level. Real input only, no top-ups. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the ai binaries, the turret binaries, glass_pass_every_level_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/los-mask-01-walk`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-11-09T01:07Z
