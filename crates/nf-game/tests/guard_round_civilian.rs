//! The wing guard's round and the office civilian (07000012 east wing, task level-12-wing-guard-civilian).
//!
//! The symptom: guard placement 9 fires down the east wing at Bond's back and a round kills the
//! office civilian (placement 1, spawner group 2, death channel 0x63) — "Killed Civilians". The
//! game's rules, from the decompile:
//!
//! - **No fire gate refuses a shot because a friendly stands in the line.** Every fire-request
//!   arming site reads only the sight-ack flag (+0x228 & 4), GunThroughWall (+0x1e), the unseen
//!   counter (+0x274) and the aim angle (+0x1d4) — no team byte, no drone-list walk, no
//!   guard→target ray (`DroneFunc_CombatState` 0x148338:110-126, `DSTATE_AimStandFire` 0x166328:
//!   121-151, `SetCombatMoveAnim` 0x13cfb0:183/386/396, `DroneWeap_DoFiring` 0x17a080:24-25,
//!   `DroneWeap_Ready2Fire` 0x178c48, `DroneWeap_FireWeapon` 0x178f68). The only line-dependent
//!   brake is the sight ray itself (`Collide_LineOfSight` mask 0xa27 — bit 0x100 clear, so class-2
//!   drones are candidates [M `Collide_PickObj` 0x1eae80:70-85]): a body on the ray kills the
//!   sight, and the fire dies within ~16 ticks unless the guard carries prop 0x17 — which guard 9
//!   does (placement 9's behaviour block, read through `PropertyTables` from the disc's
//!   ACTION.ELF).
//! - **The round stops on the civilian.** The bullet's collision pick offers every object but the
//!   shooter and the disabled (`obj+0xF0 & 0x20`) [M `Collide_PickObj` 0x1eae80:57-85 — no team
//!   read], and `NDrone2_DealWithObjHit` 0x13fff0 case 5 lands a team-1 shooter's round on the
//!   other side — teams 2/3, the civilian's side [M :181-232]. Her death channel 0x63 then fails
//!   the mission as Killed Civilians [M `DroneFunc_SetDeathChannel` 0x13e660:21-44 →
//!   `SetMissionFailReason` 0x13e560 team-3 reasons 1/3].
//!
//! This file pins the second rule at the port's decision points. The first rule is pinned by
//! absence-of-code plus the walkreal ledger (level12_wing_civ_probe.rs): guard 9 fires down the
//! wing at Bond's back on the real-input route, as the game's chain says he must.

use nf_game::ally::{death_channel_fail, FAIL_CIVILIANS};
use nf_game::drone::drone_round_applies;

/// [M 0x13fff0:181-232] `bVar4` = the victim's side (team 1 one way, teams 2/3 the other). A
/// team-1 shooter's round lands on the other side — the office civilian (team 3) takes it exactly
/// like a wall would stop it, and a fellow guard (team 1) does NOT block his own side's round…
/// wait, the sides: a team-1 shooter lands on NOT-side-1 — a team-1 victim is his own side, the
/// round does not apply to him.
#[test]
fn a_guard_round_lands_on_the_civilian_and_not_on_a_fellow_guard() {
    // 0x19 = the wing guard's weapon (placement 9 param 9 [D 07000012 audit row 9]).
    assert!(drone_round_applies(0x19, 1, 3, 10.0, 0),
        "a team-1 guard's bullet round applies to a team-3 civilian [M 0x13fff0:181-232]");
    assert!(!drone_round_applies(0x19, 1, 1, 100.0, 0),
        "…and not to a fellow team-1 guard [M 0x13fff0:181-232]");
    // The round needs a live victim [M :229-230].
    assert!(!drone_round_applies(0x19, 1, 3, 0.0, 0), "a dead victim takes nothing [M :229-230]");
    // The never-land kinds [M :154-173] — the gadget/explosive families that ride other paths.
    for kind in [0x1bu16, 0x2a, 0x2e, 0x34, 0x3a, 0x46, 0x4e, 0x50] {
        assert!(!drone_round_applies(kind, 1, 3, 10.0, 0), "kind {kind:#x} never lands [M :154-173]");
    }
}

/// [M 0x13e660:21-44 → 0x13e560] the civilian's death channel 0x63 fails the mission through
/// `SetMissionFailReason`, team 3 → reasons 1/3 = the Killed Civilians label + ch 0x60 — whatever
/// the killer's side (a guard's kill is not Bond's fault, the fail still fires).
#[test]
fn the_civilians_death_fails_the_mission_as_killed_civilians() {
    for by_player in [true, false] {
        let r = death_channel_fail(3, by_player);
        assert_eq!((r.label, r.channel), (FAIL_CIVILIANS, 0x60),
            "death channel 0x63, team 3, killer {by_player} → Killed Civilians + ch 0x60 [M 0x13e660:21-44]");
    }
}
