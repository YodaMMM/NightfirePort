## Your task: `real-health-00agent-01-hall-7`, 07000001's 00 Agent real-damage walk reaches the exit

Branch from current origin/master; if origin/s/real-health-00agent-01-hall-6 is not on master, merge it in first. Read docs/reports/real-health-00agent-01-hall-6.md there and the ignore message on walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent. Take the g11 levers (i)/(ii) first, then the escort's west route, the slab ambush, the plate AND-touch and the exit. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000001_live binary and interactables_07000001_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-01-hall-7`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-11-14T13:34Z

Main agent 11-14 (master 06db4a156): the last-guard press is MERGED - g11 dies, the hall AND fires at t13027; the walk dies at t13141 to the captain #5 (0x5000038, 0x1c, 24 hp) from (-12.3,17.2). Measured on #5 after the AND: a stand from the press spot (600/900, abort 10/30) dies inside it at t13448; walking back to (7.5,9.5) first then standing (600/1500) - #5 visible, NOT killed, Bond 4.6, death t15108/t16365 near the doorway; from the hold (6.3,7.6) t13307. Next: #5 needs a head line (the hall-6 #7 aim law: torso-centre lane aim with a non-pistol, the x8 head record) or the SMG pre-arm before the stand.

Main 11-14 (06db4a156): a bullet-line scan after the AND finds #5 (at (-12.30,6.35,17.16)) clear from (8,10)/(9,10) (d 21.5) and a band at z 11-13 (x -2..6). From (8.5,10.0) a pistol shoot_bounded (1500, wound abort 40) takes #5 24.0 -> 1.0 and leaves Bond 4.6 (death t17396); a second stand dies inside it (t17976); from (6.0,11.0) t13142. The SMG has no rounds left by then (d3_switch_to_smg keeps 0x02). Next: health into the AND (19.0 after the press) or the SMG kept for #5.

Main 11-14: the (8.5,10.0) captain stand BEFORE the g11 press (1500/2400): #5 takes no damage (not yet exposed; 26.2 kept), the press then kills g11 at 26.2 but the walk dies deeper in the hall (t17517 at (5.46,17.42); t17394). The captain line works only after the AND.

Main 11-14: head-first aim on #5 after the AND (head_aim part-5 with the gun stand-ins) - from (8.5,10.0) the head is never clear (0 ticks, 0 shots; death t18816), from the press spot t13253. Whole walk under with_real_damage_head_hunting: t13545 (head hunting only reaches 6 u). Not shipped.

Main 11-14: the captain shoot_bounded right after the AND with flank_hold_z cleared (900/1800) walks into the hall and dies inside it at t13448. Not shipped.

Main 11-14: one crouch press after the AND - byte-identical death t13141 (the next legs walk the crouch off). Not shipped.
