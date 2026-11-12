## Your task: `level-0a-civilian-slide-cels-5`, the 0a civilian slide fix lands with every walk green

Branch from origin/s/level-0a-civilian-slide-cels-4 and merge origin/master; read docs/reports/level-0a-civilian-slide-cels-4.md and -3.md there. The 0a civilian slide fix (the player-visible part, from a user report) is done on this line; it cannot reach master while walks that pass on master are red or ignored. Open, in this order: (1) walkreal_07000001_live's two difficulty-1 walks the -4 branch #[ignore]d (green isolated, red in the whole-binary order - the shared RNG draw order shifts the hall fights): make them green in the WHOLE binary run and remove both ignores; (2) 08 walk_the_whole_mission_on_real_input_real_damage_agent - the live-wire hurt volume at (112.98, 13.47, -177.50): why the route changed; (3) 11 walkreal_07000011_live base real-damage - the airborne wedge at (21.51, 48.03, 15.89) near door (20.0, 46.8, 16.6): name the arm; (4) the 05 d3 / 13 / 15 agent / 11 sister re-tunes in the -3 ledger. Real input only, never lower damage or health, no top-ups, no new ignores. Run each touched walkreal binary WHOLE (not --exact) and the full suite.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-0a-civilian-slide-cels-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox7 at 2026-11-12T16:54Z

---
## Main agent evidence (2026-11-12 12:40 MST): the drawn gate on 07000011, live
[L PCSX2 07000011 d2, Bond idle at the spawn] the start-corridor guard 0x1897990 runs GoToGoal (0x63) from (10.17, 11.68) to (2.44, 16.15) with obj+0xFC = 2 (drawn) on every frame 6-102, stepping ~0.1/frame. The port's #5 (same guard, master 70555d76c) is `drawn = false` from t33 at (7.94, 13.08) (`visible_to`'s eye rays are blocked) and takes the undrawn slide at the danim's 0.04 (0xdc), so it reaches the corridor ~85 ticks late; with the no-bunching scan on, its partner then meets the civilian and parks (0x5e) - the three start_corridor_guard_07000011_live reds. The game draws this guard although no straight ray reaches it: room visibility, not the ray, as this task's gate does.

---
claimed-by: main agent 2026-11-12 12:47 MST (sandbox7 never received it - z.ai's verification puzzle blocks sends since 10:52; the main agent does it)
