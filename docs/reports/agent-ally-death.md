# Protected actors and mission-fail reasons

## Verdict on the two claims

**`on_init_death` killer test: confirmed.** `DroneFunc_OnInitDeath` 0x145bd0:18-19 sets bVar7 from
`killer obj+0xFF == 3`. Class 3 is written only by `Player_Init` 0x1910f8:76 and `MP_ReSpawn` 0x185368:83;
drones are class 2 (`NDrone2_CreateObj` 0x14d7c8:14). The port read class 3 as a drone. Reasons 1/3 and 2/4
share a label and channel, so the only outcome that moved was prop 0x54 with no prop 0x53.

**`set_death_channel` dropping ch 0x63: confirmed.** 0x13e660:21-44 turns param 12 = 0x63 into a fail
reason. The port raised nothing. All four carriers on the disc are team 3, so the result is "Killed
Civilians" through ch 0x60, not a write to ch 0x63.

## The levels named in the audit

Census over `nf_view::dev::LEVELS`, props from the placement blocks, teams from the loaded level.

| Level | Actor | Props, param 12 | Bond's kill | A guard's kill | Old port |
|---|---|---|---|---|---|
| 07000001 | #161 | 0x54, 0x60 | fails | fails (channel) | right |
| 07000002 | #463 | 0x54, 0x60 | fails | fails (channel) | right |
| 07000007 | #413 | 0x54 from `GetDTYPENEW` | none | none | block 5 is `DeleteMe`, gone at start |
| 07000008 | #265 | 0x54, 0 | none | none | takes no damage |
| 07000012 | #23, #25 | 0x54 (normal block), 0 | fails | no fail | both wrong |
| 07000012 | #1 | 0x54 (normal block), 0x63 | fails | fails | Bond's kill wrong |
| 07000016 | #287 | 0x54, 0 | fails | no fail | both wrong |

So the visible change is on 07000012 and 07000016 only. In their alert block the 07000012 actors also carry
prop 0x53, and then any killer fails the mission.

## Fixed

- `on_init_death` follows the game's arms; `death_channel_fail` is the 0x63 arm.
- `hit_by_player` is now the last hitter. Blasts only ever set it, so a guard finishing an actor Bond had
  wounded counted as Bond's kill [M 0x13aab8:20].
- Channel-driven smashes have no owner [M 0x12c4f8:32]; they were credited to Bond. This also stops a guard
  killed by such a blast counting as dispatched.
- Reason 0xD was missing: tasing a mode 0x0C/0x11 ally now fails the mission, ch 99, no label [M 0x162b98:39-41].
- The dart's reason 0xE arm read the placement type; it now reads mode +0xC5 as the game does.
- The 07000016 spawner kill goes through `set_death_channel`.

## Sweep of the `SetMissionFailReason` callers

| Reason | Game site | Port | Result |
|---|---|---|---|
| 0 | `DSTATE_FailMission` 0x173f68 | none | unreachable: state 0xB5 needs mode 0xB, which `GetDTYPENEW` never returns |
| 1-4 | `OnInitDeath`, `SetDeathChannel` | `ally.rs` | fixed |
| 5, 6 | `PunchImpact` 0x145f28 | `drone.rs` | matches |
| 7 | `HandleImpact` 0x1469b8 | `on_tase` | matches |
| 8 | `FirstAttack` 0x148010 | `drone.rs` | matches |
| 9 | `HostageDead` 0x172628 | `drone.rs` | matches |
| 0xA | `HaveOpponentSight` 0x175ef0, prop 0x55 | none | no placement sets prop 0x55 [D]; its setters are `DoModeSettingsOLD` rows no placement reaches |
| 0xB | `PartyGirl` 0x160140 | `drone.rs` | matches |
| 0xC | `CivilianDoorGuard` 0x160648 | `drone.rs` | matches |
| 0xD | `Taser` 0x162b98 | was missing | fixed |
| 0xE | `StunDartImpact` 0x163430 | `drone.rs` | field fixed |
| 0xF | `ControlDTYPE_Zoe` 0x14b058 | `drone.rs` | matches |
| 0x10 | `PressAlarm` 0x168ea8 | `drone.rs` | matches |

## Tests

`crates/nf-view/tests/fix_ally_death_live.rs`, real levels. Each kill is the same blast with only the owner
changed; one more test uses Bond's rounds. On the old `ally.rs` and `drone.rs` three of its six tests fail:
the per-level table (07000012 and 07000016 rows), the tase, and the last-hit test. Putting back only the
one-way `hit_by_player` write fails the last-hit test. Also green on the new code: fix_actors, fix_mission,
fix_party_mission, fix_progress, civilian_no_shoot, fix_death_loop_07, fix_death_restart, fix_guest_reactions,
fix_stunner_press, tranq_dart, tase_scream, tase_silent, stun_ai_recover, fix_06_driver, walkreal 01/02/07/08/12/16,
nf-game door_guard, zoe_abandon, ai_missing_states, and the `ally::` unit tests. In one batch run
fix_ally_death_live failed once, with no assert recorded, right after an old-code build; it passed on two
full runs after that. The Bond's-rounds test passes on old code too: his
first round alerts the actor, and the alert block has prop 0x53.

## Not fixed, not proven

- **A guard's rounds cannot hit an ally in the port.** `Bullet::update` tests only the world and Bond. In the
  game a round can contact any drone: `Bullet_update` sets no drone exclusion, and `NDrone2_DealWithObjHit`
  0x13fff0 case 5 applies the hit when the shooter and victim are on opposite sides. So today only a guard's
  blast can kill an ally, and an ally's rounds never hurt a guard. Adding this would move the shared random
  stream and change fights on every level with armed allies, so it is left out. The tests use guard blasts.
- Whether a round also stops on a same-side body is read from the decompile only, not checked live.
- The death channel is raised in `Death_Anim`'s UPDATE arm in the port. In the game it is msg 2, which
  `docs/systems/drone-ai.md` documents as exit. An ally's channel fail can therefore land earlier than in the
  game, by up to the death animation. The UPDATE arm was added for Drake's defeat, so it is unchanged.
- 07000007 #414 (param 12 = 0x63) was not run on the level: raising its wait channel in a test did not wake
  it. It uses the same rule as 07000012 #1.
- 07000011 #130 is absent at the start of the level and was not run.
