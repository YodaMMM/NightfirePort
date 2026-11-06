## door-walk-03 (2026-11-06T15:06Z)
- Claimed; branch s/door-walk-03 from origin/s/door-fix-on-master (3ee92532, one-way door refusal + hostage cue + death cry; 1 commit over master 38fd3151). The red: walkreal_07000003_live walk_the_full_route_and_complete_every_objective — "servant 1's captor #11 died to the chase": Bond reaches the chase dry (clip 1 spare 0) and shoots nothing for 10800 ticks; reverting only the door direction turns it green, so the route rode the old mirrored door (door #1 at (13.96, 6.63, -30.27), facing +z, mode 1: refuses from z -28.9, opens from z -31.6 [L the driver's probe]). Plan: find where the walk crosses that door, reroute it the way a player must (through the open direction), fix the ammo by picking up what the old route collected. Test-side only; door code untouched; no [L] pins loosened.
## door-walk-03 (2026-11-06T16:42Z)
- Root-caused the red: the walk never needed the mirrored door's passage — the nav A* from the mid-courtyard end snaps onto the building's upper-floor row (node (13.52,9.49,-19.06) at dxz 0.91 beats the ground n8 at 2.06 [L dw03_probe]), the row drops Bond at the now-refusing one-way door[1] (13.96,6.63,-30.27 approach 1), the wedge ladder stands him on the door top and the garden diagonal drops him through the moat into the void (y -62818 by the servant walk) — the dry chase and the 10800-tick nothing were the freefall's symptom. Second poison: a servant crossfire kill raises ch 0x60, the fail hook auto-restarts and sets level_exit, latching the walker's stop_on_exit — every later leg no-ops ("level_exit reached at +0 ticks").
- Test-side fix (walkreal_07000003_live.rs only): first visit rerouted the player's way — E on the lane door via use_door, retry + shoot the doorway blocker, then the lane nodes (10.65,-21.62)/(10.41,-28.74) and the moat band (9.95,-34.41)/(6.87,-33.47)/(2.08,-33.39) — all nav nodes, no upper row to snap to; the servants keep_alive'd (the escort-walk arm); the front-gate visit rerouted the same shape but UNVERIFIED. Verified by six logged runs: the 0x21 plate, both rescues (captor #11 and the east-wing chain) and obj1's 0x51 AND are green on the ground with ammo flowing (clip 6-7 spare 27-41 mid-fight); the remaining red is the front-gate visit's void fall, now rerouted. Report: docs/reports/door-walk-03.md. Left: run the full-route test, then walkreal 03 whole + 02/04/interactables_07000003, then the four-crate suite.

## door-walk-03b (2026-11-06T16:53Z)
- Claimed; branch s/door-walk-03b from origin/s/door-walk-03 (d4972b69f), merged origin/master a6665d115 (clean). Task: fix walk_the_full_route_and_complete_every_objective_real_damage_agent test-side (Bond died tick 52087 at (23.39, 6.37, 11.87)); read the death ledger first, the 06 stand_shooters_down sweeps are the candidate for far shooters past the 20 u window; no door/collision/game code. Also: the void-fall spot analysis (where Bond left the geometry, could a player do the same).

Progress log (continued):
- The Agent ledger read (the run's own log): Bond died at tick 52087 at (23.39, 6.37, 11.87)
  on the si-0 (0x50) servant's nav — the A* detours the east wing through the east lawn (the
  door (22.3, 5.3, 10.9) / the 0x23 plate), where the trio #4 (killed true) / #5 shotgun hp 9 /
  #3 captain weapon 0xe crossed him: 59.4 in 26 hits from #5 at 2.1 u + 21.0 in 2 from #3 at
  1.8 u finished a DRY Bond (weapon 0x12 clip 0 spare 0, nav 6 "cycling weapons (0)"). The
  mid-nav stand_and_fire holds position (no keep_off) and the failed #5 fight's 240-tick
  cooldown covered the death. The far shooters #11/#10 (41.9/31.9 u, past the 20 u window) cost
  only 10.2 and were already dead at the death — not the lethal mechanism.
- The fix (test-side, gated difficulty >= 2 so the two green d1 walks stay byte-identical):
  (1) the 00 Agent resupply arms extended to Agent (before the rescue / before each plate /
  before each servant — the identical dry-gun death mode documented one difficulty up); (2) an
  east-lawn saddle leg (walk_nav_to (23.5, 6.3, 8.4) + clear_near 9.0 + the kill-loot pass)
  before the si-0 servant walk — nav-reachable (the failed run's nav 3 arrived there at +298).
- MEASURED: the Agent walk GREEN first try — the trio answered during the 0x50-plate nav
  (#4/#5/#3 killed at ticks 105342-105721, health 85.6 held from tick 9967 to the end), the
  captor-room cluster (#14/#2/#18/#19) killed by the return fire, the exit lawn's #16 answered,
  133964 ticks, falls: []. Whole walkreal_07000003_live: 6 passed / 0 failed / 1 ignored
  (the 00 Agent per its own ledger). Sisters: walkreal_07000002 3/0/1, walkreal_07000004 10/0,
  interactables_07000003 9/0.
- The void-fall probe (scratch example, deleted after use): down-ray grid over the moat bend —
  the void strips x 11.5-13.0 and x 16.0-17.5, z −30.75..−35.25 (floor on both sides: the
  walkable band x ≤ 10.75 y 5.34-5.52, the mid strip x 13.75-15.25 y 5.25, the south skirt
  z ≤ −36 y 5.25). The red run left the geometry crossing x ≈ 13.0 at z ≈ −30.5..−31 off the
  door-top wedge (14.06, 9.59, −29.61). A player CAN reach the strips the same way (the upper
  row is real walkable geometry; the door top is reachable by the game's own rail/jump move);
  the rerouted walk never crosses them (falls: []). The live check (does the real game reset a
  void-fallen player?) is in the report.
