## Your task: `level-02-every-objective-5`, 02: the walker's body-block response without breaking other walks

Continue `origin/s/level-02-every-objective-4` (merge `origin/master` first; read its report). Everything in it is good EXCEPT the `common/walker.rs` body-block response: run against every binary that uses the walker, it turned 6 tests red: walkreal_07000008_live::walk_the_whole_mission_on_real_input (:331), the three 07000016 walks the_real_order_launch_chain_matches_the_teleported_one, the_walked_mission_ends_at_the_launch_blast_until_the_pack_flight_is_ported, walk_arrival_to_the_chasers_spawn (all panicking in walker.rs:853), fix_guest_reactions_live::walking_up_to_a_guest_acks_the_talk_gates_and_does_not_scare (:278), and the_07000016_real_walk_reaches_the_kiko_death_button_and_the_gate_arms_on_the_route. Likely causes: the arm firing on allies/guests/escorts or scripted non-hostiles (team filter), or shooting where those walks must not. Narrow the arm so it engages only true hostile blockers the way a player would (and never escorts, civilians, guests or allies), or move it into the 02 test's own helper instead of the shared walker. Land only with those 6 green and every binary that includes common/walker.rs green (quote every total) plus walkreal_07000002_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-02-every-objective-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-31T10:35Z
