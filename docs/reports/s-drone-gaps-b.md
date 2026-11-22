# s/drone-gaps-b: drone.rs "not ported" notes, line 8000 to the end

The shared random stream moves where a scripted guard is shot or blasted, or a stay-put guard
reaches AimStandFire (it now talks from CombatNoMove). Walks 01, 05, 06, 08, 0c, 12, 14 and
enemy_accuracy_live are green on this branch.

## Ported (each with a unit test in `drone_gaps_b_tests` that fails on the old code)

1. PlayScript impact resume (`+0xCD8`) [M 0x15ba68:30-81, 161-176]: a guard playing a default /
   0x60005e6 / 0x60008ae start script that is shot leaves the script for Attack; the
   0x6000168/0x6000169/0x60007ac arms keep resume 0 (stay). `script_impact_resume`, `impact_args_for`.
2. PlayScript blast group (`HandleImpact(+0xCD8, 1)`): a grenade staggers and damages a scripted
   guard. Before, PlayScript had no blast arm and the GLOBAL fallback dropped the blast.
3. AimStandFire ENTER [M 0x166328:21-38]: mode 2 goes to SniperFire, a stay-put guard (`+0x4F8 & 0x10`
   or prop 0x40) goes to CombatNoMove (which talks and fires); AlertStatusSet(1); the UPDATE backoff
   test is skipped for stay-put guards. Before, a stationary guard stood in 0x68 silent.
4. CombatNoMove UPDATE [M 0x1643e8:49-56]: the movable exit is 0x68 (was 0x67, an extra aim-raise
   anim).
5. EnemyLookForOpponent team-2 arm [M 0x177ed0:12-14, 52-69]: the owned-cover gate, the mode 7/0xE
   run-past and the PlayScript full-alert gate now apply to allies too.
6. StandFiddle [M 0x1687c0:17-30]: timer C is now + 5 s (300 ticks; was 5 ticks) and the UPDATE
   drops the root step (`+0x38`).

## Not ported

| note | verdict |
|---|---|
| AstronautHitMove bump source (+0x470) | too big: no per-tick impact-source field |
| Global mode 0x1E | MP bot class only |
| PlayScript UPDATE SeenAndAttacking at full alert | no live evidence it fires (existing note) |
| AimStandFire `+0x19`/obj+0xdc+0x62 0x19 to 0x18 write | anim-side, not ported |
| TruckDriverInit sit target +0x694 | anim facing on the driving level |
| KikoMission voice 0x13 | no such voice on the levels |
| PressAlarm +0x594/+0x584 | follow-on anim and blend only |
| HostageGoToGoal msg 0x1b | no sender of msg 0x1b in the decompile |
| BOT movement-speed scale | MP bot class only |
| Camera_Shake / Camera_Update decay | view-side, needs the disassembly |
| subtitle 0x60d fallback | no row reaches it |
| mine cel scoping, 0x80 roll, use cel scope, door capsule radii | collision stand-ins, too big |
| FindOpponent class-0x3b object arm | too big: no object-class model |
| team-2 close probe | drone-vs-drone sight probe, too big |
| BOT_respawn class 0x11 | MP only |
| head-track snap delay, glass shatter, satchel pose, Swoosh ribbon | draw-side |

Stale notes corrected (already ported): CivilianPatrol scream and team switch, the BondTalk idle
lines (`bond_talk`), the Mayhew music readers (music.rs), the interrogate states, the facial speech
path (`speech_request`), the +0x4D4 stamp (`hit_talk_extra`).

## Open

- Held, game-true but walk-breaking: AimStandFire's `NDrone2_InvalidateAttackRoute` [M 0x166328:35,
  0x155020] (dropping `croute` on entry turns both 07000001 contact walks red; the real-damage one dies at the
  pre-stairwell guard) and CombatNoMove's `+0xD8 == 0xc` Rand(3) roll into CrouchCover
  [M 0x1643e8:103-111] (turns the 07000008 real-damage walks red; the escort VIP has prop 0x40 and
  sits in that state). Both are notes in the code; each needs a walk re-tune.

- KickObject arms timer C at +2 ticks where the game re-arms at FRAME_RATE_INT*2 (and its ENTER arms
  none); ActionAnim's re-arm is the same. Not changed here.
- AllyLeadDone ENTER's AlertStatusSet(1) is missing (the status byte has no reader).
