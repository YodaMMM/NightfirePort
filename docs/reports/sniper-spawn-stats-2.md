# sniper-spawn-stats-2 — spawner spawn stats from the load-time table, without breaking 14/16/1b

Branch `s/sniper-spawn-stats-2` from origin/master 0ea03862. The sniper-range-accuracy-0c-0d merge
was already REVERTED on master (7b097fa9) — its branch tip is an ancestor, so re-merging is a no-op;
the stats pieces are re-applied by hand instead (crates/nf-game/src/drone.rs,
crates/nf-view/src/scene.rs).

## What's done

- `load_stats_table` extracted (the level's load-time `drone_stats` table: `behaviour_util_get`
  0x1c7a98 over every PLACED kind-0x0f placement passing `Drone_Create`'s difficulty gate
  [M 0x136be0:49], in placement order) — shared by `level_drones` and the Sim.
- `Drones::stats_table` seeded with that table at `scene::build` — the table a spawn's
  `DoModeSettingsNEW` read runs against [M 0x137cb0 → 0x1c7a98/0x14c600].
- `Drone::spawn_stats_params` (own field line, added to every literal incl. the `#[cfg(test)]`
  `drone_at`) — set for spawner templates by `level_drones` (`role != Placed`).
- The spawn re-roll at `Action::Spawn`: per spawn, the LOAD-TIME table + the template's OWN section
  (a fresh clone — `behaviour_util_get` 0x1c7a98's 0x8001-tag parse applies the section), then the
  DefaultInit consumers in the game's order: ninja health 100 [M 0x14b300:744-753], `captain_setup`'s
  health/accuracy mods [M :504-535], the kind-0x18 astronaut TakeXHits health [M :726-739 + 0x14d928].
  Drake's 60.0 init (TakeXHits(2, 0x33) ×3) pinned that order — the raw table read alone stomped it.

## The 8 reds — root cause and resolution

All 8 were red under the leaked model (the spawn re-reading the SHARED mutated table, so an earlier
same-type section floods later spawns — on 07000014 every type-8 spawn after carrier #270 read
acc 4 / react 140 / hp 25 instead of the defaults 5/0/10). The pinned model — which every
real-health-verified walk encodes — is: **a spawn reads the load-time record plus its OWN section,
never an earlier spawn's section.** With the per-spawn clone read, all 8 pass with their pins
unchanged, and the 0c damage half stays fixed:

- walkreal_07000014_live: 8/8 (the three named reds + the real-damage walk)
- walkreal_07000016_live: 14/14 (both named reds)
- drake_fight_live 3/3, walkreal_0700001b_live 17/17 (accuracy 9 held), astronaut_guards_live 4/4
- 0c real-damage damage half: tick 8700 = 80.2 taken over 69 hits, 8 return-fire fights, health
  19.8 — exactly the fixed state the 0c-0d branch shipped; 0d unchanged (the documented #31 death).

## The open question (the spawn-time global write) — live checks for the main agent

[M 0x1c7a98:85] any `behaviour_util_get` call whose placement words carry the 0x8001 stats tag
writes `drone_stats + type·0x60 + kind·0x20` — the GLOBAL table — so the game's spawn chain
(NDrone2_CreateObj 0x137cb0 → DoModeSettingsNEW) should leak an earlier same-type section into
later spawns. But every real-health-verified walk (14/16/1b) pins the NON-leaked values, and the
[L] snapshot contradicts template sections reaching the table at load: `ee_start_05.bin` shows the
type-1 record EMPTY at level start although template #3 (type word 1, param 15 = 1) carries a valid
section (gates all pass: param 16 = 0 < 6, param 17 = 0x5b0003, tag 0x80010006 at param 26) — while
the type-12 record holds exactly template #3's kind words ([L] 0x31aa30: type 1 zeroed; 0x31ab30:
type 12 = [10,0,10,60,2,60,60,0]/[9,1,10,80,3,80,80,1]/[8,2,10,100,3,100,100,0]). Either the game's
type word for the table is not the raw param-15 word (a remap the port has not found), or template
sections never reach the table. Please check live (fresh boot, 07000005):
1. gdbwatch 0x31aa30 + 0x31ab30 (type 1 / type 12, kind 1) from boot through the 0x22 wake —
   expect: type 1 stays zero (or snaps to template #3's words at its first spawn), type 12 already
   holds [9,1,10,80,3,80,80,1] at load.
2. gdbwatch the spawn write: on 07000014, raise the ch-11 spawner (template #270, type 8) and read
   `drone_stats + 8·0x60 + 0x20` before/after — does the record become [4,3,25,140,4,140,140,1] at
   the spawn, and does a LATER type-8 spawn (any ch-2/3/9 group) read it (`Drone+0xB4` = 4) or the
   defaults (5)?

## Runs (quoted)

- nf-game lib 655 passed / 0 failed / 5 ignored; nf-formats 23 ok; nf-audio ok; nf-view lib 100 ok
  (the four crates' lib suites, one run).
- EVERY `walkreal_*_live` binary (78): all green — 01 (6), 02 (2), 03 (5), 04, 05, 06 (5), 07 (3),
  08, 09, 0a, 0b, 0c + 0c_barrelrow + 0c_eastroute, 0d, 11, 11_route, 12, 13, 14 (8), 15, 16 (14),
  1b (17), 41, 4a, 4a_satchel, celprobe and the 55 probe binaries — 0 failed.
- Spawn-sensitive integration: ai_live 37, stealth_ai_live 9, enemy_accuracy_live,
  fix_guard_weapons_live, find_opponent_live, sensor_live, anim_events_live, door_guard_live,
  guards_0c_live, attack_reentry_live, start_corridor_guard_07000011_live,
  corridor_guards_07000012_live, fix_finale_live, stuck_guard_combat_05_live,
  civilian_first_sighting_alert_live, wiring_07000003_live, drake_fight_live 3, astronaut_guards_live
  4 — all green, 0 failed.
- NOT run: the bulk of nf-view's remaining integration binaries (~380; the sandbox kills background
  jobs between tool calls, so the fast suite runner could not be used and the window went to the
  walkreal sweep). No failure is known in that set from this task's changes; next session should run
  `python3 tools/test/suite.py` on the merged tree.

## What's left

- The full suite (tools/test/suite.py) on the final merged tree.
- The live checks above — they decide whether the spawn read should eventually be the global
  (leaking) table; the port ships the pinned non-leaking read until then.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures for the
session (no numbers were visible in any interface output to copy).

## ntfy failed

All three notification paths failed (the sandbox cannot reach ntfy.sh, same as previous sessions):

1. curl: `curl: (28) Connection timed out after 20001 milliseconds`
2. python urllib: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. wget: no output, killed by the 25 s timeout (exit 124) while hanging on the connect
