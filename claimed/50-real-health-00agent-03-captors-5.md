## Your task: `real-health-00agent-03-captors-5`, 07000003's 00 Agent full-route real-damage walk goes green

Branch from current origin/master; if origin/s/real-health-00agent-03-captors-4 is not on master, merge it in first. Read docs/reports/real-health-00agent-03-captors-4.md there and the ignore message on walk_the_full_route_and_complete_every_objective_real_damage_00agent. Take the next levers in the ledger: the saddle route without the (22.1,-2.0) anchor, then the #7 aimed stand once armed. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000003_live binary and interactables_07000003_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-03-captors-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-14T14:00Z

Main 11-14 (master 06db4a156, NF_DMG_TRACE): the t74135 death is a DRY STAND at the east lawn saddle (23.18,5.50,7.20): Bond reaches the saddle with the P2K 3+13 and 41.2 health, the saddle wake sweep burns it, then the stand cycles weapons (0x1 clip 1 spare 0, cycling 0..11) while #7 (0x5000039, 0x12, hp 4) at (11.4,10.1,-16.1), 26.5 u, lands 1.8 a hit for ~40. A loot + resupply pass before the saddle finds nothing (3+13 unchanged). Next: leave the saddle line when dry (the sweep breaks on dry, but the stand inside it keeps cycling), or carry rounds from the servants-ground fights.

Main 11-14 more: the saddle sweep has NO dry check (the comment claims one) - Bond arrives at the saddle with the P2K 3+13 and 41.2, the sweep and the stands spend it, and the walker switch_to_armed then stands ~730 ticks cycling an empty wheel (0x1 clip 1) in #7s line. Measured: a dry stop inside the sweep / skipping the sweep when nothing fires - byte-identical t74135 (the cycling comes from the loot pass after it); return fire off on the walk to the saddle (3+13 kept) t73170; saddle sweep radius 15/20 (skip #7 at 26.5) t74135, with return fire off t72825. Not shipped. #7 at (11.4,10.1,-16.1) needs an answer that is not a 26.5 u pistol duel.

Main 11-14: a positioning shoot_bounded on #7 at the saddle (900) dies inside it at t73864 at (29.49,6.38,18.38). Not shipped.

Main 11-14: crouched at the saddle (one press before the sweep) - t74068, no gain (#7 at 26.5 u still lands). Not shipped.
