# bot-push-01-escort-routes-2 — the last red 01 walk under the faithful push

Branch `s/bot-push-01-escort-routes-2` cut from origin/master 84e64088; origin/s/bot-push-01-escort-routes
(0c2f5964 — the per-bot push gates + the re-tuned 01 walks) merged clean at the start.

## What's done

1. **following_mayhew_down is GREEN on this merge** — `walkreal_07000001_live` runs 5 passed /
   1 failed, and the 5 include `following_mayhew_down_completes_the_level`, `the_exit_fade_starts`,
   `the_hall_guards_wait`, `the_plate_refuses_bond_alone` and the topped-up main walk: the
   merged branch's step-back follow (the PlayerInWay release reads only Bond's distance
   [M 0x15ea28 case 3]) holds on this tree. The red one in the file is the real-health sister
   `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` (the same 5/6 shape the
   previous report left).
2. **The 01 real-health sister: diagnosed to the tick, not yet green.** The death is now a
   measured mechanism, not a mystery:
   - Bond reaches the hall's east door with **clip 9 spare 0 and health 70.5** (the pre-hall
     resupply found no pickup in the walked-path corridor: `resupply done (false)`), where two
     shotgun guards (drone #6 skin 0x5000036 hp 12, drone #7 skin 0x5000038 hp 24) hug the
     door's gap at **1.7-3.2 u**.
   - At t3440 the shooters list is still EMPTY (they have not hit Bond yet and their aim gate
     does not read Bond); their FIRST volley lands inside one 10-tick window —
     **70.5 → 22.5 (48.0, eight pellets of the 24-hp captain at 3.2 u)** — and at t3450 the pick
     finally sees `cands [7]` and the fight starts. The door leaf eats the eye rays from the
     door line (aim None; the creep walks Bond INTO the hug), and the second volley
     (6 × 3.00 + 1.50 = 19.5 on the death frame) kills at t3461. 26 hits for 100.0 on the run.
   - The port-vs-game call stands: under 3.0 u every pellet lands with the ×2 TooClose damage
     [M DroneWeap_DoBulletAccuracy 0x179740:113-115; Drone_ModBulletDamage 0x139f50:21-23] — a
     real player at this door eats the first volley too. The harness gap is the answer layer:
     the fight must open BEFORE the hug (the named-guard chase pattern, aimed through the gap
     from outside the door), and Bond must not stage through the crowd at 9 rounds.
   - The tools built for it this session (all **thread-local opt-ins, default OFF** —
     `common/damage_log.rs` + two read sites in `common/walker.rs`): the fights' head-hunting
     pick (`with_real_damage_head_hunting`), the unseen-cover fallback + creep + dry-resupply
     with the lane fights' 48 u / 7.5 u window (`with_real_damage_unseen_fights`), the implied
     2.6 back-off out of the TooClose hug while the stand works, the active-threat cooldown
     bypass (a guard that hit Bond within the last 60 frames is answered despite a cooldown),
     and the stuck ladder's body-block arm (`with_real_damage_body_block`). The sister runs
     head-hunt + body-block + unseen; it still dies at the door — the next layer is the
     walk's own hall phase (stage/fight the crowd from outside the door before staging).
3. **walkreal_07000003's captor fight: the ammo half fixed.** The merged master tree's dry-fight
   regression ("servant 0's captor: out of ammo" at +1497, the assert at :273) is fixed with the
   01 named-loop pattern (a near-dry gun restocks first; a dry stand resupplies from the floor
   drops and re-engages) — the fight no longer dries. What remains is the next layer:
   servant 0's captor (drone 2, 12 hp) reads `exposed false` at 4.1 u (every part buried) and
   the flank arm's bot orbit wedges at the courtyard's south props (three 900-tick orbits at
   (33.12, 6.71, 4.20)) — the assert at :288 fires with the servant still 0xa. File-local to
   `walkreal_07000003_live.rs`.
4. **exit_aim_01_live 1/1 and flank_fall_repro 2/2 green** on this tree.

## The walkreal sweep on this tree (release, --test-threads=1, slim ISO)

Green (14 files): 07000005 (12), 07000009 (6), 0700000b (4+1 ign), 0700000c_barrelrow (1),
0700000d (1+1 ign), 07000011 (1), 07000011_route (1), 07000012 (1), 07000041 (7), 0700004a (2),
0700004a_satchel (2), 07000015 (4), exit_aim_01 (1), flank_fall_repro (2).

Red (11 files): 07000001 (the real-health sister, above), 07000003 (the captor wedge, above),
07000004 `the_kill_leads_into_the_games_outro`, 07000006 `walk_the_castle_interior_to_the_party`,
07000007 `walk_the_party_start_to_finish` + its real-health sister, 0700000a
`walk_office_floor_qworm_keyboard_hatch` + its real-health sister, 0700000c (1),
0700000c_eastroute (1), 07000013 (2), 07000014 (1), 07000016 (5).
Incomplete (2 files, timeouts, not failures): 07000002 (260 s), 0700001b (110 s).

None of the reds outside 01/03 touches this branch's diff: every shared change is gated on the
thread-local opt-ins (default false) and the two other edits are file-local. They are the merged
push gates' (player.rs/scene.rs, per-bot push + the +0x4f8 arming polarity) and the merged
0b/0c walks' timelines meeting this tree — the escort branch itself never swept these files, so
this sweep is the first whole-tree inventory. Which reds are new vs pre-existing on master is
the next session's first question (a master-branch run of 04/06/07/0a/0c/13/14/16 answers it).

## How Bond's movement changes when he brushes a guard (carried forward + this session)

- **The push runs every tick, geometry only, no ground preservation** [M Collide_Jointy
  0x1ec180:229-281] — a fraction of a unit per tick while the overlap persists.
- **Brushing a standing guard lifts Bond for a few ticks**: measured 5 ticks (apex y 6.49)
  against the 07000003 courtyard guard with the closest-segment distance 0.65.
- **A standing capsule holds Bond out at ~1.4 u spine-to-spine** (0.55 + 0.4 + margin) — this
  session's numbers agree: at the hall door the two shotguns' hugs land volleys from 1.7 u
  (six −3.00 pellets + one −1.50 on one frame) while Bond's own hold-out keeps him ~1.4 off
  each spine; the crowd's door gap slides him 7.37→7.51 x, 8.99→9.31 z across 900-tick windows
  (the plate sister's orbit logs).
- **The follow's kite**: a chasing guard inside 2.6 trades 0.75-damage punches (the 1.8-u fists
  [M Player_WeaponInitBullet]) while the follow backs at forward −1.0; the new implied 2.6
  back-off gives the return-fire stands the same step-out (measured: the #7 captain's volley
  falls from 6.0 a pellet at 3.2 u to the spread's reach past 3.0 — the death frame's pellets
  were 3.00/1.50 at 1.7 u, i.e. the ×2 TooClose band).

## What's left

1. **The 01 real-health sister**: re-tune the hall phase (clear the door crowd from OUTSIDE the
   east door before staging — the chokepoint fight the level offers; the named-guard chase
   pattern aimed through the gap), then the run.
2. **07000003**: the courtyard captor wedge (the bot orbit at the south props) — either a flank
   point past the props or the walk-to-a-clear-line approach; then 4/4.
3. **The sweep's 11 reds**: first the master-branch comparison (pre-existing vs the push gates),
   then per-file re-tunes; 02 and 1b need full-length runs (timeouts only).
4. **The full suite** (nf-game/nf-formats/nf-audio/nf-view) did not run this session — the
   walkreal sweep consumed the window. The lib crates are untouched by this diff except through
   the merge.

## Live checks for the main agent

1. 07000001's hall: Bond at the east door with the two shotgun guards inside — do their first
   volleys land before the player can react (the port measures 48.0 in one 10-tick window at
   3.2 u), and does the door leaf eat the aim rays from the door line (the port's aim reads
   None from there)?
2. 07000003's courtyard: servant 0's captor (drone 2) — is any part visible from the rescue
   spot at 4.1 u in the real game, and where does a player stand to get the shot?

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost figures
for the session.
