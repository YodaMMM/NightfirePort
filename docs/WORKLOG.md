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
