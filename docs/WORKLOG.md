# WORKLOG (branch running copy)

## smg-close-range-hits

Branch `s/smg-close-range-hits` created from `origin/master` (f3b24398).

Task: Bond's SMG (full-auto) rounds do zero damage to guards at close range on 07000001 — 32
rounds at 0.5-2.1 u, aim settled, rounds spawned (`last_shot` advances), zero damage, while the
pistol (0x2) hits the same guards at 3.5-9 u (docs/reports/level-01-agent-hall-fight.md on
origin/s/level-01-agent-hall-fight, "Remaining" item 3). Candidates: the first-segment spawn
origin for full-auto records (`fire_player`'s `shot_from` in crates/nf-view/src/scene.rs, the
muzzle/rocket-origin midpoint path) starting the segment past the target, or the drone hit-box
test's near-field gate. Port the game's rule with addresses [M Bullet_init 0x12c5a0,
Bullet_update 0x12e7b0, the drone collision handler].

Done = a real-input test: Bond fires the SMG at a guard 1 u and 2 u away and the guard takes the
record's damage per round; the same for the pistol as a sister; every walkreal_*_live green; the
full suite total line.

Progress:
- [18:26 UTC] Sandbox start: repo + work/ + ISO present; branch created from origin/master,
  WORKLOG started. Queue worktree to refresh next; then the level-01 report re-read and the
  bullet/drone code study.
- [19:05-20:05 UTC] The premise REFUTED by the repro: the SMG 0x12 registers per-record damage
  at 1.12 u (3 rounds → 3 impacts, 2.25/round, drop 6.750 == expected) and at 1.88-1.90 u, the
  pistol sisters land at 1.09/1.87/5.89 u — while the ch-0x5 "dormant" pair takes NOTHING at
  0.78-1.02 u because the game ITSELF holds them unhittable: NDrone2_DSTATE_WaitSwitch 0x15b960
  ENTER → NDrone2_Enable(0) 0x149dd0 sets obj+0xF0 |= 0x20 and Collide_PickObj 0x1eae80:26-30
  drops those — the port's skip is that rule, not a bug. The level-01 report's 32 zero-damage
  rounds measured the script hold. Prober's trap found: the escort's own 0x1c volleys land in
  the same player_impacts/dr.hit records (rec-dmg 3.0 vs 3.5 identifies them) — the test
  attributes by hit.weapon. Test: crates/nf-view/tests/smg_close_range_live.rs (ok, 18.23 s);
  no src/ changes on this branch. Report: docs/reports/smg-close-range-hits.md.

## real-health-agent-00agent-11 (started 2026-11-03 21:45 UTC)

Branch s/real-health-agent-00agent-11 cut from origin/master (68298698). The difficulty-01-07 line
(origin/s/real-health-agent-difficulty-01-07) is already merged into master — no merge needed.
Plan: add the Agent sister (difficulty 2) and the 00 Agent sister (difficulty 3) to
walkreal_07000011_live, loaded the menu way with load_with_carry_difficulty(..., d), asserting
Drones::difficulty == d at load; check the difficulty rows the game reads — the pain scale
(Player_HandlePain 0x1902c8), the accuracy (DroneWeap_DoBulletAccuracy 0x179740:74-88), the
min-difficulty spawns (Drone_Create 0x136be0:49) [M]; where Bond dies read the damage log against
that pipeline, fix the port where it differs (with the address) else the tactics; Operative walk
stays green; real input only.

## real-health-agent-00agent-11 (round 1 outcome, 2026-11-03 23:18 UTC)

The Agent (d2) sister walk_spawn_to_the_lower_floor_load_point_real_damage_agent is GREEN
(load_with_carry_difficulty + the Drones::difficulty assert; the keypad #177 solved on real input —
created LOCKED at d2/3, Lock_Create 0x1ae1f8:67-69 auto-unlocks only at d1). The 00 Agent (d3)
sister is RED at the east connector (tick 14455): the d3 min-difficulty spawns
([M Drone_Create 0x136be0:49]; 37 drones at load) outpace the 7-round pistol — the 0x0500004-skin
guards' 7.0/round volleys (accuracy Hard 1.0) drained 84 health before the first kill-drop
resupply. Built the opt-in lane_floor_drops band (the fight drops rest ON the floor ray, 1.09
under Bond's pos y — the lane 0.5 band could not see them); with it the NE-landing pass walked
onto a fresh kill drop and armed clip 30. The remaining levers listed in the report.

## level-08-00agent-doors (branch cut, 2026-11-03 22:31 UTC)

Task: re-route the late-mission leg of
walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent (ignored on
master after the door-routes merge: Bond dies tick 25773 near (113.19, 28.28, -122.97) to #20,
skin 0x500002f, weapon 0x15, 9.8 u, 6.0 a-hit) the way a player would — kill #20 first, cover,
approach. Real input only, no top-ups. Operative and Agent walks stay green. Branch cut from
origin/master 38052bf6.

## level-08-00agent-doors (round 1 outcome, 2026-11-04 00:05 UTC)

The pinned 25773 boarding death is FIXED: the d3 fights drain the pistol pool dry and the
platform-mouth stand's `switch_to_armed` wheel landed the OWNED AT-420 (2..=0x47 counts it) — the
re-route scoops the #15/#16/#17 drops at the long link, checks firearm-armed (launcher excluded)
at the platform mouth with resupply from the fresh #23/#19 drops, and the wave clears at zero
cost (#20 down in 12 ticks at 25250, #23/#24 by 25693, health 78.0 untouched). Port law fixed:
Break_Kill's "the collision goes away" [M 0x12b970:60] — the dead 0x20 pane's glist now leaves
the per-tick dynamic set (script_parts); the pane shoot-out itself works (kind 2, health 1.0 —
one tap each; the rows leave, the script objects delete). The Hind fight remains: the d3 hp is
598 = DiffMod 2.0 × 299 (6 rocket hits needed) and the carriage model's OWN glass quads still
stop the launch-corridor rays after the panes die — the collide-vs-draw glist split is the next
session's lead (the report carries the blast dumps, the pad probe, the levers).

## level-08-00agent-hind (branch cut, 2026-11-04 01:00 UTC)

Task: finish the bottom-terminal Hind fight of
walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent the way a
player would — the walk is still ignored on master, red at the Hind fight, with the blocker in
its ignore message. The doors session pinned the lead: after the panes die, the carriage model's
own glass quads stop the launch-corridor rays (collide-vs-draw glist split). Branch cut from
origin/s/level-08-00agent-doors 26c30220, merged origin/master cee0f0df (fast-forward; master
already carries the doors work and strips WORKLOG as a process file).

## level-08-00agent-hind (the flight-law evidence, 2026-11-04 02:05 UTC)

The Hind blocker root-caused from the decompile + disc dumps (all [M] with addr, [L] probe
outputs; the probe ran nf-extract coll-obj/coll-scan + a throwaway weapon-record dump):

- The bullet FLIGHT query is armed per tick [M Bullet_update 0x12e7b0 disasm 0x12f030-0x12f058]:
  kind 0x101/0x201 at HITTEST+0x94, the pick mask at +0x98 ZEROED every flying tick, and
  **+0x9a = 0x20 when weapon_data+0x6c bit 0x800 is clear**. The HITTEST base is obj+0xD4
  [M Collide_Update 0x1ea398:78-84].
- The contact walk [M Collide_Update 0x1ea398:216-229]: pass = (HITDATA+0x51 bit 1: the
  obj+0xF0 0x40 glist flag) | (surface byte bit 0x40); +0x80 rolls the RNG; **when +0x9a has
  0x20, a surface byte == 0x10 (glass) contact is ALWAYS a pass** — the round continues; the
  mirrored contact still lands on the pane's obj+0xD0 and Break_Update kills it.
- [L weapon records (ACTION.ELF weapon_data)]: every gun + the AT-420 0x2c (0x00012204) has
  +0x6c bit 0x800 CLEAR → the glass pass arms for all of them; 0x01/0x45 have it SET.
- [L coll-obj 07000008 map1]: OBJECT_Glass1a/6a, DOOR_Glass_1/4, Glass10 (the 16 ride panes,
  hashes 0x02000477/78/79/0x0200037f/0x02000382 from ride script 0x0600004C streams
  7-18/23-26) — collision ALL mat 0x10. OBJECT_Carriage (0x0200038a) — flags **0x1000**,
  250 tris ALL mat 0x06, bounds only (−0.42..0.66, −0.31..1.07, −1.98..1.74) — the hanger
  box ABOVE the cabin (the carriage origin rides 4.5 above the cabin floor), NOT the walls
  Bond sees. Object_Floorgrate (0x02000413) flags 0x1001, 13 tris.
- The port already carries the pane-level glass pass (drone.rs:14703-14722 glass_pass) but
  the WORLD flight trace (segment_bullet) still stops on mat-0x10 quads — the alive panes
  stop the world ray, so the 00agent corridor gate reads blocked even where the game flies.
- Port fix queued: the flight traces gain the glass pass (surface byte & 0x3f == 0x10 passes
  for records with +0x6c bit 0x800 clear), matching 0x1ea398:216-229.

## level-08-00agent-hind (round 1 outcome, 2026-11-04 02:26 UTC)

The flight's GLASS PASS is ported and verified: `segment_flight`/`segment_flight_scoped` drop
the surface-byte-0x10 hits for the rounds whose record +0x6C bit 0x800 is clear [M Collide_Update
0x1ea398:216-229; Bullet_update 0x12e7b0 disasm 0x12f030-0x12f058]; the player-bullet flight
(tw/tstop/copter-shield) and the 00agent d3 gates read it. The GUARD-side flight keeps the
documented deviation (glass_pass: false at the four spawn sites): arming it turned the d2 Agent
walk red at tick 25178 — the #23/#19 volleys pass the top-station glass the platform-mouth stand
hides behind; the route work is the next session's. walkreal_07000008_live 5 passed 0 failed
1 ignored (d2 re-verified green; nf-game lib 656/0/5).

The 00agent walk (run un-ignored once): the shoot-out taps 9 rounds, kills NOTHING (dead rows
[]; every pane reads OPEN — the rows' script objects ended at the ride's EntityEnd: no dynamic
quads AND the box-contact kill never fires — the orphaned-row lead), then rocket 1 (t138) flies
through the open pane plane and self-blasts. Next: re-source the bottom-terminal panes, then the
corridor gate's wall-band reach. The ignore stays on with the state in its message.

## level-08-pane-shootout (started 2026-11-04 06:57 UTC)

Branch cut from origin/s/level-08-00agent-hind (merged origin/master; master stripped the
process files, WORKLOG restored from the hind copy). Goal: the 08 bottom-terminal pane
shoot-out must kill panes [M Break_Update; the mirrored contact on the glass object's
obj+0xD0 chain], a shot-out window must break and clear for the player; then finish
walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent (real input
only, no top-ups; remove the ignore when green); every walkreal_*_live; full suite total line.
