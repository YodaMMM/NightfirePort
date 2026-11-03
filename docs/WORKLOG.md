
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
- (16:23) ALL FOUR SISTERS GREEN on real health, real input: 09 100.0/50.0 untouched (the
  patrol-beat clears held), 0a 100.0 untouched, 08 24.8 (the head-hunt fights — the plain pick
  let the walkway shotgun guard #20 grind 10 × −4.2 before the plain wrapper died at 25137),
  0b 72.4 (the lane sprint — see the report). The 0b tactic: the terminal-room GT pair
  #174/#175 arms on the dome fence's ch 0x0D OR the files take's ch 0x06 (OR #290 → ch 0x0E =
  p4 14 [D]; the level census has no ch-28 writer, so the exec trio #291-293 never wakes) and
  fires the eased barrel on the 0.1 s cadence [M 0x180498 case 4] — every in-lane tick is a
  fire tick, so the armed transits run the raw stick loop (`lane_sprint`, the deck-sprint
  pattern, all gated on `real`): the GT drain fell 76.4 → 19.2. The four binaries re-run green
  (serial, NF_SILENT=1: 08 5/0, 09 7/0, 0a 11/0+3 ign, 0b 6/0), nf-game 102 binaries green,
  nf-formats' 3 disc_golden fails are the slim-ISO allowance (missing /DRIVING/MIS01.MUS#0).
  The full suite.py did not fit the window (its build step alone — all ~100 nf-view binaries —
  exceeded the tool timeout on this 2-core box; the runs would not have fit either).
