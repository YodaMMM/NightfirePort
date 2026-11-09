
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
- (15:35-16:00) reproduced both reds (agent dies tick 15851 at the take slot, 21 hits for 100.0:
  #4 61.2, world 19.2, #6 11.2, #7/#8 2.8 each); read the los-mask-join report: the parapet sight
  is game-verified (live check), so 0b stays a re-route problem; Bond's only weapon is the K5
  (darts never land in Attack [M 0x164210]) and the deck squad spawns at/after control return, so
  "drop first" is closed — cover/timing it is.
- (16:00-16:20) diag dump at control return: hp 80.8, NO fusebox on 0b, no engaged hurt volume —
  the "live wire" 19.2 is the ARMED GT PAIR #174/#175 (z −21.6) firing across the dome-floor
  lane sprint (39 rounds, 6 × 3.2 land); the report's fusebox-slot lead does not apply to 0b.
  The body armour (kind 3, ch 0x0) at (17.53, 47.68, 28.46) exists but the east lane is held by
  guard #1 (state 0x5c at (10.8, 11.9)) + turret #293 — the armour detour walk DIED there
  (31 hits, tick 19554); detour abandoned.
- (16:25-16:50) the death's core is the take stand: the sprint broke at dxz < 1.0 and the take
  loop stood 95 ticks at the slot (3 × 11.2 + 5.2 from #4) while ch 0x3F never fired. Changed
  the sprint to run THROUGH the slot (break only on ch 0x3F, the take required on the pass).
  probe_0b_take sweep (fresh load, no fire): the take fires from z ≥ ~41.0 across x 1.5-3.2 but
  the BODY pushing north from the south is walled at z 40.99 at x 2.0-2.6 — the south approach
  stalls one tick short; d1 (green) still takes it on the pass, so a crossing x exists — the
  push sweep's first run used the wrong yaw convention (fixed, not yet re-run). NEXT: re-run the
  fixed push sweep, route the sprint through the crossing x, then the 00 Agent sister.
