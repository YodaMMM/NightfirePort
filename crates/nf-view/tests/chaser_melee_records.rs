//! The 07000016 chaser's melee damage records, pinned through the real loader.
//!
//! `DroneWeap_CloseCombatImpact` 0x1789c0 fires `Bullet_init(-1, obj, obj, record, obj+0x30,
//! facing, 0xffff)`: record 99 with the 2.0 range when the drone's anim-set short +0xDA is 7
//! (the kick), else record 100 (kind 1 → 0x65, kind 2 → 0x66) with the 1.75 range and the
//! 60° facing gate [M 0x1789c0:26-40]. The bullet's damage is the record's f32 at +0x0C
//! [M1 DroneInit_TakeXHits 0x14d944 `lwc1 f0,12(v0)`; docs/fields/weapon_data.md]; the records
//! are the level archive's weapon table [D].
//!
//! 07000016's chaser is the skin-0x0500001e dtype-0xf drone spawned by the launch chain
//! (spawner #257 group 6 [D]; #261). Her skin makes `NDrone2_DefaultInit`'s switch give kind
//! 0xb, ANIM SET 7 and the fists (weapon 0) [M 0x14b300's switch; nf-formats `drone_setup`] —
//! so her close-combat impact always rides the set-7 kick arm: record 99, the 2.0 band, no
//! facing gate. The player-side application is `Player_HandlePain` (0x1902c8) × the
//! per-level/difficulty factor (07000016's Plr_DMod row 0.4/0.6/1.0 [M 0x1d4424 group]):
//! the 50-raw punch lands 20.0 health at Operative, 30.0 at Agent.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn f32_at(r: &[u8], o: usize) -> f32 { f32::from_le_bytes(r[o..o + 4].try_into().unwrap()) }

/// The records the chaser's melee can fire, pinned to the disc's own bytes [D].
#[test]
fn the_chasers_melee_records_carry_the_game_damage() {
    let iso_path = iso();
    let scene = nf_view::load(&iso_path, "07000016.bin", false).expect("load 07000016");
    let sim = scene.sim.as_ref().unwrap();
    let wd = &sim.data().weapons;
    assert!(wd.records.len() > 100, "the archive carries the close-combat records");
    // The set-7 kick (her arm — skin 0x0500001e → anim set 7 [M 0x14b300]): 50 raw [D +0x0C].
    assert_eq!(f32_at(&wd.records[99], 0x0c), 50.0,
        "record 99 — the anim-set-7 kick's Bullet_init record [M 0x1789c0:26-40] — damage [D +0x0C]");
    // The non-kick close-combat arm's records (kinds 0/1/2), for the sisters that are NOT her:
    for rec in [100usize, 0x65, 0x66] {
        assert_eq!(f32_at(&wd.records[rec], 0x0c), 15.0,
            "record {rec} — the non-kick close-combat arm [M 0x1789c0] — damage [D +0x0C]");
    }
}
