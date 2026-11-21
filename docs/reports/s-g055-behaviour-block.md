# s/g055-behaviour-block: GAPS G-055, the second behaviour block

Random stream: not proven either way. The change alters the 07000004/07000016 ninjas' start state
(0xA6 instead of 0xA7) and their engage/combat ranges; 0xA6's ENTER goes to 0xA7 in the same tick, and
the walkreal_07000004 (10) and walkreal_07000016 (19) binaries pass unchanged, but no stream digest was
compared. Kept on the branch.

## What was checked [M]
Every writer of `+0x4D8` / `+0x29` and every caller of Set1st/Set2nd/Using2nd/ChangeToAttackMode in
the ACTION.ELF decompile, against `Drone::props` / `alert_props` / `using_2nd`:
- Already ported: ChangeToAttackMode's EnemyMission, CivilianMission (0x15d860:60) and GoToGoal
  callers, GoToGoal's arrival tail (the only Using2ndBehaviour reader), FirstAttack's alt tail,
  PreDroneControl's flush, HandleImpact's 09/0A Set1st at all four sites (tase/smoke/flash/dart).
- Missing, now ported: `GetDroneTypeAttackTypeFriend` 0x14cbd8:119-127. Voice set `+0xD8 == 0xB`
  (skin 0x0500001E, the ninja) with first-block alertness >= 1.0 calls ChangeToAttackMode and returns
  before the `+0x5A2` alert fold and the `+0x5A4` / `+0x13A` fold. The port folded the ninja's start
  state 0xA6 NinjaStand to 0xA7. Now `type_setup` takes the arm and `start_state` never folds voice
  set 0xB. NinjaStand's ENTER writes (bounded, vision, `+0xF0 = 3.0`, `+0xFC = 5.0`
  [M 0x16b990 case 1]) were missing from the port's 0xA6 arm; added.
- Not reachable: `DoTypeSettingsOLD` 0x14d2e8:72 (OLD chain, no placement uses it).
- No direct `+0x4DC` / `+0x4E8` reads after creation except DoModeSettingsNEW's second-block alertness
  (the port's `alert_block`). No other `+0x29` reader.

## Census [D] (`g055_behaviour_census`, all levels via Level::parse)
p5 = 100 on all 555 drone placements; the only other kind-0x0f records are 33 skinless p5 = 0
template records (alert_props None, matching the game's no-swap path). Skin 0x0500001E: 07000004 #1,
07000016 #105, #126, all alertness 1.0, none with an alternate state, so the creation swap itself is a
no-op on the disc.

## Tests
- New `g055_kind_b_creation` (2 tests): fail on origin/master (start state 0xA7 vs 0xA6; engage range
  12.0 vs 3.0), pass now.
- New `g055_behaviour_census`: pass.
- walkreal_07000004_live 10/10, walkreal_07000016_live 19/19, ninja_melee_live, ninja_boss_cutscene_live,
  start_corridor_guard_07000011_live, nf-game lib swap/second_block/tower_impact filters: all pass.

## Unproven
- Whether a player sees a difference: the ninja's engage range 3.0 / combat range 5.0 now match the
  game's NinjaStand ENTER; not compared live.
- NinjaStand ENTER's eyes datum (`AnimDatumSetEntity(…, 100, 0x2000312, 0xff)`) is visual and not
  ported here (no draw changes).
- G-055's existing open item stays open: the 07000011 guard grenade that hits the corridor ceiling
  needs a live check.
