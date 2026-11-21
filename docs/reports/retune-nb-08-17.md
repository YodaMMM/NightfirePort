# retune-nb-08-17 — the 08 whole-mission 00 Agent real-damage walk, round 17

## What's done

- Branch `s/retune-nb-08-17` from `origin/master` (b5278f0d1, which carries
  walks-1121-08-cabin's fusebox-pit sight-abort) with `s/retune-nb-08-16` merged in (08-16
  was not on master). Two conflicts in `crates/nf-view/tests/walkreal_07000008_live.rs`,
  both resolved keeping both sides' complementary content: the excursion loop keeps
  master's `leg_abort_on_sight` statement with 08-16's retune-nb-08-9 dead-lever comment;
  the ignore attribute keeps the 08-16 round-16 ledger (master's walks-1121 ledger text
  lives in master's history and in the kept comment blocks). Setup from scratch (the
  sandbox reset): clone, work-slim + the 369823203-byte ISO archive verified, rustc 1.100,
  the extracted libasound2/libasound2-dev debs, `cargo fetch`.
- **The task's ask: the full upstream hit ledger.** The merged tree's baseline
  RESHAPED the death [L the 2026-11-21 NF_DMG_TRACE run]: **death t28183 (frame 28185) at
  (113.01, 13.47, −179.26), 59 hits for 100.0, 22 return-fire fights** — 1741 ticks earlier
  than round 16's t29924 and to a NEW killer. Every hit from the start (tick, source,
  amount):
  | ticks | source | amount |
  |---|---|---|
  | t601, 664, 720, 729, 840, 900, 960 | the y-46 deck trio (#14 1, #3 1, #13 4, #2 2; weapon 0xa) | 7 × −4.00 = **−28.0** |
  | t4560, 4800, 4860 | #15 (0xa) | 3 × −4.00 = **−12.0** |
  | t5201 | #16 (0xa) | **−4.0** |
  | t6666, 6670 | #5 (0x15) | 2 × −3.00 = **−6.0** |
  | t24695 | #26 (0x15) | **−4.0** |
  | t27776–27951 | the world (the hurt volume) | 14 × −2.00 = **−28.0** |
  | t28111, 28179, 28183 | #25 (0x15, (117.5,23.0,−133.5), d 47.0, hp 20) | 3 × −6.00 = **−18.0** |
  The trio rows all land on Bond WEDGED at (147.7–148.8, 46.1, −93.8/9) while the nav's
  return fire killed the four one pistol stand at a time (#14 t672, #3 t740, #13 t971,
  #2 t1033) — the fights worked; the cost is the rooted stands in the lanes. The 14 zap
  rows are the PRE-FIGHT approach burst: the fight only opens at frame 27917 [L the
  [08 gates] t0 row], and no stance scheduler exists before it. #25's three rows land
  DURING the t51–191 dive ride — the missile cam freezes the input. The dive launch
  landed −101.0 at fight-t 191 [L the [08 rocket] rows]. Master's sight-abort visibly
  paid: #15 12.0/3 (was 32.0/7) and #26 one row (was #27 21.0/7).
- **Ledger lever (a) THE JIGGLE FIGHTS — TRIED and DEAD on its own real run.** The
  [M]+[D]-backed idea: DroneWeap_DoBulletAccuracy 0x179740 multiplies the hit chance by
  TargetFirstMoved_Accuracy 0.25 for 120 ticks after each drone observes Bond START a move
  (the per-drone latch, restamped every move-start [M 0x179a48:65-73]) and by
  TargetMoving_Accuracy 0.75 while moving; the ACTION.ELF statics 0x30cb00-08 = 2.0/0.25/0.75
  [D], effective on 07000008 (the ReadTuningVars 05-08 tail does not override them [M
  nf-game/src/tuning.rs:49-59]). Implemented as the shared-helper opt-in
  (`jiggle_fights` in `crates/nf-view/tests/common/damage_log.rs`: a 1-tick alternating
  strafe press every 50 ticks inside `stand_and_fire`, skipped under the
  sniper/pre-throw settle patterns; the walk set it d3-only). MEASURED [L the lever run]:
  the deck took 8 rows (−32.0, WORSE than the rooted 28.0 — the roll variance went the
  wrong way) and the re-rolled timeline wedged the boarding descent HARD ("the lower level
  east nav 5", the half-open door pos01 0.50, 6 mitigation cycles, assert) at t26700 with
  health 7.0 — the shifted fight positions moved every downstream nav start state.
  Reverted (`git checkout` on the helper).
- **Ledger lever (b) THE POST-LAUNCH SURVIVAL CROUCH — TRIED and DEAD as a byte-identical
  no-op.** The west-band stand press gated to `rockets < 1` (the launch path it serves is
  measured dead after the dive launch — line_ok 0/210, round 16). MEASURED [L the lever
  run]: death t28183 byte-identical, 59 hits — the 14 standing zaps are PRE-FIGHT (no
  scheduler there) and the killer rows are #25 during the ride, so the press never mattered
  in this timeline. Reverted.
- The final tree is the merge itself (no surviving code change outside the ledger text
  inside the ignore attribute), so the affected-binary set is `walkreal_07000008_live`
  alone.

## How it was verified

- Each lever judged on its own real run of the d3 00-Agent walk with `NF_DMG_TRACE`:
  the baseline t28183 (the merged tree), the jiggle run (t26700 wedge assert), the
  crouch run (t28183 byte-identical) [L three runs this window].
- The whole `walkreal_07000008_live` binary on the final tree: **6 passed / 1 failed (the
  target walk), 165.28 s** — the d1/d2 real-damage sisters, the plain walks, the d2 Agent
  sister and both ally tests byte-exact green, matching the round-14/15/16 final-tree
  numbers [L the final-tree run, `--include-ignored --test-threads=1`]. No
  `crates/*/src/` change and no surviving `crates/nf-view/tests/common/` change (the
  jiggle flag was reverted), so every other binary is byte-identical in behaviour to the
  merged 08-16 state.
- The full suite (tools/test/suite.py): the background runner was started in the first
  minutes and died between tool calls (the same kill rounds 13-16 verified), and the
  10-minute foreground cap cannot hold the four-crate build on this sandbox. What ran:
  the whole affected binary above (6/1, 165.28 s) plus the three lever runs. The final
  tree changes only the ledger text inside the one ignore attribute, so the other
  binaries' behaviour is unchanged from the merged 08-16 state.

## What's left (the next window's levers, in priority order)

1. **THE PRE-FIGHT CROUCH** — press the crouch at the anchor arrival so the approach burst
   hits the crouched capsule (the standing burst rate 14 rows/175 ticks vs the crouched
   tail 7/1600 [L both baselines] — ~24.0 of entry health). The fight then opens CROUCHED:
   the t51 dive launch never fires (the crouched eye is lintel-blindfolded [L the crouched
   slit anchor ledger]) so NO ride happens and #25's snipes get ANSWERED by the fight
   machinery instead of hitting a frozen input — the fight is survival-only either way
   (the post-launch line_ok is measured dead, round 16), and the crouched eye's steep
   rising rays to #25 (y 23, d 47) keep the fights live.
2. The hall trio's rooted-stand cost (−28.0): the accuracy law's motion factors do not
   bite (the jiggle run), the lanes cover the whole deck (the 08-9 ledger), the early
   scoop+arm detour costs more than it saves (the arm cycle alone is 396 ticks in the
   lanes). The only untried shape is the fight ORDER (the most-hits-first pick over the
   last-hit pick) — #13's 4 rows landed while he was fought third.
3. The reply-cadence live check (carried rounds 11-16) now decides lever (1)'s value: if
   the w62 chain is provoked by the launch, the ride-free path also skips the chain.

## Live checks for the main agent

- The reply cadence (carried, rounds 11-16): a gdbwatch on the w62 spawn after the dive
  launch — provoked or scheduled? It decides whether the ride-free path also skips the
  chain.
- #25's identity/placement: a gdbwatch or a live boot at the boarding ledge — the merged
  tree's killer #25 (skin 0x500002f, (117.5, 23.0, −133.5), weapon 0x15, hp 20) at d 47
  through the cabin's bands; what wakes him and whether the game's own fight can reach him
  from the anchor.

## Usage

not visible to me
