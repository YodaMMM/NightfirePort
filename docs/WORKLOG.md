
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

## level-0b-roof-sprint-2
Started 2026-11-09 16:47 UTC. Branch s/level-0b-roof-sprint-2 cut from origin/s/los-mask-join (5e36d06ba).
Task: continue blocked/10-level-0b-roof-sprint.md — read its report and next-steps list and carry them out
on the los-mask-join base. Find with the fixed probe the x where Bond's body can cross the parachute
housing's south face line (z about 41.0); route the final deck leg through it; trim the turret pair's
damage on the dome floor if needed; make the Agent and 00 Agent real-damage walks pass, then the d1 walk
and the whole walkreal_0700000b_live binary. If no crossing exists anywhere in the port's collision, say
so with the probe data. No damage/health lowering, no loosened pins, no new ignores, no draw changes.

Work log:
- (16:47) sandbox rebuilt (work-slim + onfoot ISO 369823203 ok); branch created, WORKLOG entry pushed.
  Reading the blocked report next.
- (17:00-19:20) probe v2 answered next-step 1: the standing-height sweep crosses the z≈41.0 face
  and TAKES at every x 0.6-3.4 (the old wall was the wrong-yaw artifact); no live check needed.
  NF_PICKUPLOG showed the take DOES fire in the real walk (~t15844, dist 1.768 vs 1.791, los
  true → Some(Label)) — Bond arrives at the face at 5.2 hp (dome pair 19.2 + #4 point-blank
  36.4) and dies 7 ticks after the take. Plate E-hold fixed to the game's press-edge + head-look
  law [M 0x1a5460:396-403, 0x1a5170:100-108]. Dome lanes measured (head-on 80.8 best); deck
  reroutes measured (west hug 87.2 through the band; the NW structure x −11..−14 z 24-28 slides
  shallow NE headings into its corner). Walk still red; plan in the report.

## level-0b-roof-sprint-3
Started 2026-11-09 20:55 UTC. Branch s/level-0b-roof-sprint-3 cut from origin/s/level-0b-roof-sprint-2
(a4e3541ae). Task: 0700000b roof-deck sprint, third pass — the crossing exists and the take fires
(sprint-2); the failure is margin. Bond arrives with 5.2 hp: the dome-floor turret pair #174/#175
costs 19.2 and guard #4 parks on the sprint line at (2.6, 58.0, 32.7) costing 36.4 in the last 8 u.
Finish sprint-2's "What's left": a lane past the turret pair that costs less, and dropping or
avoiding #4 before the final dash (as a player would), then the Agent and 00 Agent real-damage walks,
then the d1 walk and the whole walkreal_0700000b_live binary. No damage/health lowering, no loosened
pins, no new ignores, no draw changes.
