
## real-health-agent-difficulty-08-09-0a-0b
Started 2026-11-03 14:50 UTC. Branch s/real-health-agent-difficulty-08-09-0a-0b cut from origin/master
f9c1d69c (real-health-agent-difficulty-01-07 is already on master — no extra merge). Task: the Agent
(2) sisters of the real-health walks for 08, 09, 0a, 0b — the same route and end, loaded the menu
way, Drones::difficulty == 2 asserted at load; fix the port where it differs [M address], else the
tactics; real input only; shared helpers opt-in per level. Every sister green, the Operative walks
green, each touched walkreal_*_live + nf-game.

Work log:
- (14:50) branch start, WORKLOG entry. Next: the fast runner, then the 01-07 report.
- (16:12) sandbox reset, setup redone from scratch (clone, work-slim, the ISO, cargo fetch, the
  queue worktree; no rustup/7z preinstalled — user-prefix installs). The 01-07 code is ON master
  (load_with_carry_difficulty, the tuning rows, Drone_Create's gate); its 01-04 sister TESTS were
  deliberately reverted on master (2879088f "01/06 red on the batch") and are re-landing via the
  separate real-health-00agent-02-03-04 branch — so NO 01-07 merge: the only pieces it adds that
  my levels need are already here, and merging would resurrect the reverted 01-04 sisters. The
  01-07 report read; the pattern copied from the 05/07 sisters on master.
- (16:13) the four sisters written: 08 `walk_the_whole_mission_on_real_input_at(d)` +
  `..._real_damage_agent` (d2), 09 `walk_tower_a_start_to_finish_at(d)` + sister, 0a
  `load_carry_at`/`walk_to_security_keyboard_at`/`walk_office_floor_qworm_keyboard_hatch_at` +
  sister (the d2 branch on the WARM load only — the cold capture harvests boot values), 0b
  `load_fresh_at` + `walk_fresh_spawn_to_the_mission_end_real_damage_agent` (the same fight
  package, the same no-poke fresh boot). d1 call sites byte-identical, the difficulty assert at
  every load. Build running; commit after it compiles.
