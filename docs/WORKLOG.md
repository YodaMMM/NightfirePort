
## markers-world-objects
Started 2026-11-08 18:08 UTC. Branch s/markers-world-objects cut from origin/master 2998b6a76.
Task: settle the 38 [S] markers in crates/nf-game/src/world.rs, triggers.rs, gas.rs,
breakable.rs, player.rs, pickup.rs, rigid_body.rs against the decompile — port, prove-equal,
or prove-unreachable; keep the rand draw count; pin play changes with real-input tests.

Work log:
- (18:08) branch created, WORKLOG entry, checkpoint pushed. Sweeping the [S] markers next.

## los-mask-envelope
Started 2026-11-08 22:05 UTC. Branch s/los-mask-envelope cut from origin/s/markers-world-objects (cd56542), merged origin/master (b059df94).
Task: port the five unported mask-application sites from the LOS_MASKS_ON comment (world.rs:70-75):
(1) obj+0xF0 skip word of every picked object [M 0x1eae80:82]; (2) type-byte arms [M 0x1eae80:83-87];
(3) SIGHT_THROUGH_PART in los_walk (mask bit 4, 0x1eae80:79-81); (4) obj-pass drop under mask bit 0x8
for dynamic non-pane parts; (5) subject exclusions reach all callers [M 0x1eae80:57-59] — in the
game's order, then flip the gate. Then every walkreal_*_live one level per call, repair any moved
walk gated to its level, rand draws unchanged, two real-input sight pins, ai+turret+glass binaries,
nf-game, fast-runner suite on the final tree.

Work log:
- (22:05) branch created, WORKLOG entry. Note: docs/reports/markers-world-objects.md does not exist on
  that branch (WORKLOG-only stub); the five items come from the LOS_MASKS_ON comment in world.rs.
- (22:20-22:40) the five envelope items ported into los_walk/line_of_sight_hit/line_of_sight_scoped's
  dynamic blocks (Moving::pick_pair: obj+0xF0 = placement glist+0x44 via Control_SetGList 0x134960:11-12,
  RB exception [M 0x1d6bf0:52 no SetGList], live 0x10 via the id bit; classes pane 0x20/RB 0x1a/door
  0x1c/DynObj 0x52; arms in the game's order); LOS_MASKS_ON=true; the 3885 mask test un-ignored.
- (22:45-23:00) nf-game lib 749 green; glass_pass_every_level_live 3 green; gun_turrets_live 3,
  turret_hits_live 3, cameras_turrets_census_live 1, ai_missing_states_live 10, ai_points_live 4,
  cover_selection_live 2, dtype_sniper_alert_live 1 — all green. ai_live: 36+1 — the second-hit part
  pin (port-derived, "not yet checked live") contradicted the masked vision chain; un-pinned the part,
  kept the damage/kill chain [M]; 37 green.
- (23:00-23:30) walkreal_07000001_live 6/1: the d2 real-damage walk DIES t8977 (guard #4 10 hits);
  baseline (HEAD~1) passes — first divergence the t678 #8 fight (baseline 100->100, masked 100->93.7):
  the masked 0xa27 vision (glass 0x10 / node-pass / 0x40-material drops [M 0x1eae80:70-82 +
  0x1ebaa8:300-319]) engages guards the strict pre-mask ray did not. walkreal_07000002_live 3/1: the
  real-health agent walk misses the doorway-sensor ch 0x81 assert (timeline/path shift suspected,
  unrooted). Both repairs need a route/tactics re-tune against the game-faithful vision — next window.

## level-0b-roof-sprint
Started 2026-11-09 15:23 UTC. Branch s/level-0b-roof-sprint cut from origin/s/los-mask-join (5e36d06ba).
Task: with the masked sight rays from s/los-mask-join, Bond dies at the parachute on the Agent and
00 Agent roof-deck walks of 0700000b. Find the player's real way across the roof under the game's
rules (cover, timing, which guards to drop first) and make both walks pass. Reproduce on the real
level through real input first, not a teleport; compare with the decompile; no draw changes.

Work log:
- (15:23) branch created, WORKLOG entry, checkpoint pushed. Reading docs/reports/los-mask-join.md,
  walkreal_0700000b_live.rs and the walker helpers next.
