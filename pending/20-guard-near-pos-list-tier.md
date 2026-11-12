## Your task: `guard-near-pos-list-tier`, guards count a route node reached when another guard stands on it, with every walk green

Branch from origin/s/guard-stuck-14-corner and merge origin/master; read docs/reports/guard-stuck-14-corner.md there. The change (Drone::near_pos_fn, NDrone2_DroneNearPos 0x143e68's list tier inside LinkCreep_AtSpecialNode 0x159dd0's 0.4-1.0 band, radius 0.4 from $f12) is reviewed and stays - it is the game. On master + this branch the full suite has exactly 3 reds, all green with the change reverted: fix_mission_live::the_07000001_exit_works_end_to_end, walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage and ::..._real_damage_agent. For each: find what the rule changes there (which guard now counts which special node - door/action/kick - as reached, and what that does to the mission script or the fights; NF_AILOG/NF_DMG_TRACE), then (a) if it exposes a port bug elsewhere, fix that with its address, or (b) if it is the game's behaviour shifting the walk, re-derive the walk leg on real input. The 01 exit test is a mission-chain test - find whether the exit itself breaks (a player-facing break) before touching it. Never lower damage or health, no top-ups, never weaken a decompile-backed assert. Run fix_mission_live, walkreal_07000001_live, walkreal_07000003_live, guard_stuck_sweep_live, nf-game --lib.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-guard-near-pos-list-tier`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: main agent at 2026-11-12T08:59:10Z

---
## Main agent's findings (2026-11-12, released back to pending)
- The 01 exit is NOT broken for a player: walkreal_07000001_live (the real-input walk onto the LoadPointActivator plate) is 7/0/1 with the rule.
- On 07000001 the list tier fires exactly once in fix_mission_live::the_07000001_exit_works_end_to_end: drone 11 counts the node (-13.50, 5.74, 14.09) reached because peer 14 stands at (-13.65, 6.34, 13.73).
- After that, the scripted escort diverges: when the ambush clears (ch 0x99) Mayhew is at (6.55, 6.35, 12.75) in state 0x47 instead of holding at (-32.94, 4.74, 11.61) in state 0x20 (LeadWait) as on master; the harness never gets him onto the plate. Find why Mayhew leaves his hold (which state chain takes him to 0x47 - NF_AILOG on him), and whether the game's escort would do the same with drone 11's changed timing; fix the port if it is a port bug, else re-derive the test's stand-ins (never by weakening the exit assert).
- An env-gated eprintln in near_pos_fn (print the drone, node and peer when the list tier fires) found the event above in one run.
