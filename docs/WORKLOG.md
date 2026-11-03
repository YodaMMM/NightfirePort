# WORKLOG

## player-upward-shot
Started 2026-11-03 01:28 UTC. Branch s/player-upward-shot cut from origin/master 6f16146e.
Task: on 0700000c Bond's aimed rounds at the deck snipers (weapon 0x1e drones ~33 u away and
above him) never land — three clips dry, target hp unchanged, strict gun rays true, while level
shots at the same range work. Trace one upward shot through the port's bullet path (flight
segment, per-tick HITTEST, Collide_Pick straddle cels, lip geometry), compare each step with the
game [M Bullet_update 0x12e7b0, Collide_Pick 0x1eb320, cel straddle 0x1eac98], fix any place the
port stops a round the game lets through, and pin the outcome with a real-input test from the
yard firing up at a deck sniper.

Work log:
- (01:28) branch start, WORKLOG entry.
- (01:35-01:55) research: the port's flight (PlayerBullet, drone.rs update_player_bullets — the
  per-tick cel-scoped `segment_bullet_scoped(find_cel(b.pos), …)` at :14571, flat whole-world
  fallback when find_cel → None) vs the game's pick law [M Bullet_update 0x12e7b0 fills
  HITTEST+0x80 = the bullet's LINKED cel obj+0x20 (sticky, refreshed by
  control_handle_cel_change 0x133288 which, for a class-5 bullet in open air, CLAMPS the
  position back into the old cel's bbox and relinks — the pick never becomes the whole world);
  Collide_Pick 0x1eb320:125 straddle = Collide_StraddleCels 0x1eac98 BFS from that cel through
  the portals the tick segment crosses].
- (02:25) the diagnostic trace (crates/nf-view/tests/upward_shot_trace_live.rs, measurement-spot
  teleport at the yard ladder base [L the real-health run's pre-climb stand spot], aim at the
  exposed chest box of the deck sentry #18 (36.4, 12.06, -208.79) d 34.9 like stand_and_fire):
  the predictor exposes the box (eye + gun rays), the lane's world stop is 4.7 u BEHIND the box
  — and the FLIGHT passes ~0.8 u ABOVE the aim line at the box (no Drone impact, world impact
  on the wall behind, hp 10.0 unchanged). The shot dir bends up vs the view aim — next: the
  zero-spread (aiming) trace and the scene.rs muzzle/aim construction [M Player_WeaponInitBullet
  0x1a4bf0 + Player_GetAimingPoint 0x1a4a98].
- (02:50) **THE VERDICT — no port flight bug; the harness fired hip.** The aimed act (input.aiming
  held → the raise completes ~t20 → sim.aiming true → player_spread mul = 0 [M 0x12c5a0:139-170;
  the 0x1e record carries 0x400000 [L ee_start_05.bin @ 0x2c10b8 flags68 0x408050]]) lands: the
  trace's round impacts `Drone { drone: 18, part: 4 }` at (36.16, 12.27, -208.68), hp 10 → -50.
  The hip act the walker's stand_and_fire always fired (its Input never sets `aiming`) carries the
  record's +0x24 = 100.0 base cone [L 0x2c10b8] = 0.14 rad = **4.9 u scatter radius at 35 u**
  against a ~0.5 u chest box — the game's own hip sniper misses the same way [M 0x12c5a0:139-170
  mul = 1 without the aim bit]. "Level shots work" = the level fights ran at d 7-10 where the same
  cone is ~1.1 u. The flight (cel scope 78/77 the whole lane, no find_cel None, no flat fallback,
  straddle covers the lane, the world stop 4.7 u BEHIND the box) matches the game's pick law.
  ALSO pinned: step_bond's 2nd arg is the TRIGGER (probe_0d_bullet's `!clip_empty` convention) —
  my first aimed runs passed false and never fired.
- (03:00) next: opt-in `aimed_fights` in DamageLog::stand_and_fire (the lane_fights pattern) +
  the real-input test from the level start to the yard ladder base (legs 1-6 factored), firing up
  at #18 through the aimed act; symptom pin (hip = no hits) + the fix pin (aimed = the kill).
- (03:10) the fix shipped: OPT-IN `DamageLog::aimed_fights` (the lane_fights pattern; default off
  — zero behaviour change for every other walk) — `stand_and_fire` holds the aim button under the
  flag AND tightens the tap gate to 0.01/0.01 (the plain 0.03 rad/0.04 norm = ±1.05/±2.2 u at 35 u
  is wider than the chest box; the zero-spread round goes exactly along the view). The real-input
  proof (crates/nf-view/tests/upward_shot_0700000c_live.rs, both green):
  `the_hip_pull_dries_three_clips_on_the_deck_sentry` (the symptom: the walk's own legs 1-6 reach
  the yard ladder base, the hip stand dries its rounds, hp 10 → 10) and
  `the_aimed_upward_round_lands_and_drops_the_deck_sentry` (the emplacement drops — the pair
  #18/#36 at one spot; a round into the exposed box hits whichever twin the ray meets first).
  walkreal0c.rs: legs 1-6 factored into `walk_spawn_to_the_yard_ladder_base` (the band walk calls
  it and continues at leg 7 — no behaviour change).
