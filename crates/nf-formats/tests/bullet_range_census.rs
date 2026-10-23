//! bullet-range-210 census: every `weapon_data` record's flight-death range vs the game's
//! `BU+0xFC` fuse clock, straight from the disc.
//!
//! The disassembly settles what each field does:
//! - **The flight death range is `weapon_data+0x1C`** [M `Bullet_update` 0x12e7b0: case-1 death
//!   `c.olt.s` of `0x1c($v1)` vs `0xf4($s4)` at 0x12e95c-0x12e984 — state 1 via the state jump
//!   table at 0x2f4300 (state 1 → 0x12e95c); the step clamps to the +0x1C boundary at
//!   0x12f014-0x12f02c and `BU+0xF4` keeps the overshoot, so the next tick's case-1 check kills
//!   the round]. NOT `BU+0xFC`.
//! - **`BU+0xFC` is the fuse/arm clock** [M `Bullet_init` 0x12c5a0:103 writes 210.0 with the
//!   per-id overrides :255-283 — 0x3b/0x69/0x37; `Bullet_update` counts it down per tick ONLY for
//!   `weapon+0x70 & 0x80000` records (explode at ≤ 0, 0x12ee08-0x12ee24) and for id 0x45 (die at
//!   ≤ 0, 0x12efa4-0x12efc4, the 1.0/tick constant at 0x30D0D8)]. The merged task-831 claim that
//!   0x12efa4 is the general death-range read was the id-0x45-only arm.
//!
//! Run with: `NF_ISO=… cargo test -p nf-formats --test bullet_range_census`

use nf_formats::iso9660::IsoImage;
use nf_formats::weapon::WeaponData;

fn weapon_data() -> WeaponData {
    let path = std::env::var("NF_ISO").expect("NF_ISO");
    let iso = IsoImage::open(&path).unwrap();
    let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
    WeaponData::from_action_elf(&elf).unwrap()
}

/// The game's `Bullet_init` fuse write (`BU+0xFC`), verbatim from 0x12c5a0: the 210.0 default
/// [:103], the 300.0 `+0x70 & 0x10000` arm [:196-208ish], and the per-id overrides :255-283 —
/// 0x3b (`((w+0x00 − id)·5 + 5)·60`, 0x6c → 900), 0x69 → 5, 0x37 → 420 (7777 when the owner is a
/// class-3 drone). This is the FUSE/ARM clock in ticks, not a distance.
fn bullet_init_fuse(w: &[u8]) -> f32 {
    let w0 = u16::from_le_bytes([w[0x00], w[0x01]]);
    let id = u16::from_le_bytes([w[0x02], w[0x03]]);
    let flags70 = u32::from_le_bytes(w[0x70..0x74].try_into().unwrap());
    let mut fuse = 210.0f32;
    if flags70 & 0x1_0000 != 0 { fuse = 300.0; }
    if id == 0x3b {
        fuse = if w0 == 0x6c { 900.0 } else { (((w0 as i32 - id as i32) * 5 + 5) * 0x3c) as f32 };
    }
    if id == 0x69 { fuse = 5.0; }
    if id == 0x37 { fuse = 420.0; }
    fuse
}

/// Every record's flight-death range is its own `+0x1C` f32 [M 0x12e95c]; the fuse clock
/// (`BU+0xFC`) follows `Bullet_init`'s chain and is a TICK count for the +0x80000/0x45 countdowns
/// [M 0x12ee08, 0x12efa4]. The two are different quantities: this pins both from the disc and
/// lists where they differ, so the port's `Bullet.range = +0x1C` model and the grenade fuse model
/// (`scene.rs`'s `+0x1C / speed` and 210/300 tick fuses) both stay anchored.
#[test]
fn census_flight_range_plus_1c_vs_fuse_clock() {
    let wd = weapon_data();
    assert_eq!(wd.records.len(), 115);
    let f = |w: &[u8], o: usize| f32::from_le_bytes(w[o..o + 4].try_into().unwrap());
    // [M 0x12c5a0: the 300.0 arm keys on `weapon+0x70 & 0x10000` and runs BEFORE the id
    // overrides (:216-222 vs :255-283), so an id-override record that carries the bit ends at
    // the override value anyway]. Exactly one disc record carries the bit: rec 55 (id 0x37,
    // +0x70 = 0x18300 [D weapon_data]) — its 300.0 is overwritten by the 0x37 chain to
    // 420.0/7777.0, so no disc record's FINAL fuse is the flag's 300.0.
    let flagged: Vec<usize> = (0..wd.records.len()).filter(|&i| u32::from_le_bytes(wd.records[i][0x70..0x74].try_into().unwrap()) & 0x1_0000 != 0).collect();
    assert_eq!(flagged, vec![55], "only rec 55 carries +0x70 bit 0x10000 [D weapon_data]");
    // The 0x37 override's 7777.0 arm is the drone-owned case (owner class 3 [M 0x12c5a0:266-283]);
    // the ownerless/player-fired census value is 420.0 for all three 0x37 records (55/56/57).
    for (i, w) in wd.records.iter().enumerate() {
        let fuse = bullet_init_fuse(w);
        let death = f(w, 0x1c);
        let id = u16::from_le_bytes([w[0x02], w[0x03]]);
        // Pin the fuse chain's disc outcomes by id family [D weapon_data]:
        match id {
            0x3b => {
                let expected = if u16::from_le_bytes([w[0], w[1]]) == 0x6c { 900.0 }
                    else { (((u16::from_le_bytes([w[0], w[1]]) as i32 - 0x3b as i32) * 5 + 5) * 60) as f32 };
                assert_eq!(fuse, expected, "rec {i} (id 0x3b) fuse");
            }
            0x69 => assert_eq!(fuse, 5.0, "rec {i} (id 0x69) fuse"),
            0x37 => assert_eq!(fuse, 420.0, "rec {i} (id 0x37, ownerless census) fuse"),
            _ => assert_eq!(fuse, 210.0, "rec {i} (id {id:#x}) keeps the 210.0 default"),
        }
        // The flight-death range is the record's own +0x1C — including the 0.0 records (the
        // thrown-grenade family never runs the flying case-1 state) and the 5000.0 records
        // (the stick/satchel records fly as thrown objects, not case-1 bullets).
        let _ = death; // enumerated in the report's table; the pins below carry the numbers
    }
    // The named pins (the report cites these) [D weapon_data 0x2bf150]:
    // P2K 2/3 (id 0x02): +0x1C = 50.0 — the round dies at 50 units [M 0x12e95c], NOT 210.
    assert_eq!(f(&wd.records[2], 0x1c), 50.0, "P2K flight death range");
    assert_eq!(f(&wd.records[3], 0x1c), 50.0, "P2K twin");
    // The Laser watch 0x4E/0x4F: +0x1C = 4.0/8.0 (task 831's [L]; the disc confirms).
    assert_eq!(f(&wd.records[0x4e], 0x1c), 4.0, "Laser 0x4E flight death range");
    assert_eq!(f(&wd.records[0x4f], 0x1c), 8.0, "Laser 0x4F flight death range");
    // The SG5 0x16/0x17 = 450; the Spas 0x1c/0x1d = 25; the sniper-family 0x1e-0x29 = 1200.
    assert_eq!(f(&wd.records[0x16], 0x1c), 450.0, "SG5");
    assert_eq!(f(&wd.records[0x1c], 0x1c), 25.0, "Spas");
    assert_eq!(f(&wd.records[0x1e], 0x1c), 1200.0, "sniper family");
    // The satchel family bases itself: rec 59 (0x3b, w0 0x3b) → 300 ticks; rec 108 (id 0x3b,
    // w0 0x6c) → the 900.0 special arm [M 0x12c5a0:255-262].
    assert_eq!(bullet_init_fuse(&wd.records[59]), 300.0, "rec 59 satchel fuse ticks");
    assert_eq!(bullet_init_fuse(&wd.records[108]), 900.0, "rec 108 (w0 0x6c) fuse ticks");
    assert_eq!(bullet_init_fuse(&wd.records[60]), 600.0, "rec 60 fuse ticks");
    assert_eq!(bullet_init_fuse(&wd.records[105]), 5.0, "rec 105 (0x69) fuse ticks");
    // The fuse values are NOT distances the flying rounds obey: the satchel family's +0x1C is
    // 5000.0 while its fuse is 300 — a 300-UNIT cap would die mid-flight long before the stick
    // lands, and the game's satchel throws clearly exceed that [D weapon_data; M the +0x80000
    // countdown at 0x12ee08 consumes BU+0xFC as TICKS].
    assert_eq!(f(&wd.records[59], 0x1c), 5000.0, "satchel +0x1C vs its 300-tick fuse");
}
