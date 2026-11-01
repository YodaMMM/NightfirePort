//! The drone hit chance (`DroneWeap_DoBulletAccuracy` 0x179740), the damage of a drone round on Bond
//! (`Drone_ModBulletDamage` 0x139f50) and Bond's health floor (`Player_HandlePain` 0x1902c8), from the
//! level tuning `ReadTuningVars` 0x1d3a80 writes (`nf_game::tuning::read_tuning_vars`). No disc needed.

use nf_game::drone::{hit_chance, player_hit_damage, Motion};
use nf_game::tuning::{read_tuning_vars, Tuning};

fn tuning(level: u32) -> Tuning {
    let mut t = Tuning::default();
    read_tuning_vars(&mut t, level);
    t
}

const STILL: Motion = Motion { moving: false, first_moved: false, first_stopped: false };

/// p = (100 − 5·acc) × the difficulty factor; 0700001B's group writes Easy 0.6 / Normal 0.8 / Hard 1.2
/// [M 0x1d3a80 case 0x700001b]; difficulty 0 and 2 read Normal, 3 and 4 Hard [M 0x179740:74-88].
#[test]
fn the_hit_chance_follows_the_level_s_difficulty_factors() {
    let t = tuning(0x0700_001b);
    let p = |diff| hit_chance(&t, 0x0700_001b, diff, 5, 10.0, STILL, 0);
    assert!((p(1) - 45.0).abs() < 1e-3, "Easy: 75 × 0.6, got {}", p(1));
    assert!((p(2) - 60.0).abs() < 1e-3, "Normal: 75 × 0.8, got {}", p(2));
    assert!((p(0) - 60.0).abs() < 1e-3, "difficulty 0 reads Normal, got {}", p(0));
    assert!((p(3) - 90.0).abs() < 1e-3, "Hard: 75 × 1.2, got {}", p(3));
    assert!((p(4) - 90.0).abs() < 1e-3, "difficulty 4 reads Hard, got {}", p(4));
    // Sister: 07000005's group writes 0.8 / 0.9 / 1.3 — a different level must not give 1B's numbers.
    let t5 = tuning(0x0700_0005);
    assert!((hit_chance(&t5, 0x0700_0005, 2, 5, 10.0, STILL, 0) - 67.5).abs() < 1e-3);
    assert!((hit_chance(&t5, 0x0700_0005, 2, 5, 10.0, STILL, 0) - p(2)).abs() > 1.0);
}

/// TooClose (3.0 on the campaign groups) applies its ×2 twice [M 0x179740:49-51, 90-92]; moving ×0.75,
/// first moved ×0.25 [M :52-54, :71-73].
#[test]
fn too_close_doubles_twice_and_motion_cuts() {
    let t = tuning(0x0700_0002);
    // 0700_0002 Normal 0.7: 75 × 2 × 0.7 × 2 = 210 under 3.0.
    assert!((hit_chance(&t, 0x0700_0002, 2, 5, 2.9, STILL, 0) - 210.0).abs() < 1e-3);
    assert!((hit_chance(&t, 0x0700_0002, 2, 5, 3.0, STILL, 0) - 52.5).abs() < 1e-3, "3.0 is not too close");
    let moving = Motion { moving: true, ..STILL };
    let first = Motion { moving: true, first_moved: true, first_stopped: false };
    assert!((hit_chance(&t, 0x0700_0002, 2, 5, 10.0, moving, 0) - 52.5 * 0.75).abs() < 1e-3);
    assert!((hit_chance(&t, 0x0700_0002, 2, 5, 10.0, first, 0) - 52.5 * 0.75 * 0.25).abs() < 1e-3);
}

/// The first-stopped cut: ×TargetFirstStopped_Accuracy (0.1 on 0C/0D) only on 0x0700000C/0D and only
/// for the anim set 0x13 [M 0x179740:55-70]. Sister: the old model had no such cut (0.1× missing).
#[test]
fn first_stopped_cuts_only_the_0c_0d_snipers() {
    let t = tuning(0x0700_000c);
    let stopped = Motion { first_stopped: true, ..STILL };
    let base = hit_chance(&t, 0x0700_000c, 2, 5, 10.0, STILL, 0x13);
    let cut = hit_chance(&t, 0x0700_000c, 2, 5, 10.0, stopped, 0x13);
    assert!((base - 60.0).abs() < 1e-3, "75 × 0.8, got {base}");
    assert!((cut - 6.0).abs() < 1e-3, "× 0.1 for the 0x13 set, got {cut}");
    assert_eq!(hit_chance(&t, 0x0700_000c, 2, 5, 10.0, stopped, 6), base, "other anim sets keep the full chance");
    let t5 = tuning(0x0700_0005);
    assert_eq!(hit_chance(&t5, 0x0700_0005, 2, 5, 10.0, stopped, 0x13), hit_chance(&t5, 0x0700_0005, 2, 5, 10.0, STILL, 0x13),
        "outside 0C/0D the flag is ignored");
}

/// The back-shot ×2 keys on the shooter's ANIM SET 0x12 [M 0x139f50:24-48 — `*(short *)(drone + 0xda)`],
/// not on the weapon id 0x12; TooClose ×2 under 3.0 [M :21-23]; 0x63–0x66 untouched; voice type 0xC → 0.
#[test]
fn the_back_shot_reads_the_anim_set_not_the_weapon() {
    let t = tuning(0x0700_0002);
    let behind = (0.3, 0.0); // the bullet travels along Bond's facing: a shot into his back
    let front = (std::f32::consts::PI, 0.0);
    let dmg = |weapon, set, dist: f32, (by, fy): (f32, f32)| player_hit_damage(&t, 0x0700_0002, 1.5, weapon, 0, dist, 0, set, by, fy);
    assert_eq!(dmg(0x12, 0x12, 10.0, behind), 3.0, "anim set 0x12 from behind: × 2");
    assert_eq!(dmg(0x12, 0x12, 10.0, front), 1.5, "from the front: no back-shot");
    // Sister: the weapon-0x12 guard (anim set 6 [M DroneWeap_ChangeWeapon config]) takes NO back-shot —
    // the old rule keyed on the weapon id and doubled this hit.
    assert_eq!(dmg(0x12, 6, 10.0, behind), 1.5);
    assert_eq!(dmg(0x12, 6, 2.0, front), 3.0, "TooClose × 2");
    assert_eq!(player_hit_damage(&t, 0x0700_001b, 1.5, 0x12, 0, 10.0, 0, 0x12, 0.3, 0.0), 1.5, "0700001B never back-shots");
    assert_eq!(player_hit_damage(&t, 0x0700_0002, 15.0, 0x65, 0, 1.0, 0, 0x12, 0.3, 0.0), 15.0, "0x63–0x66 skip every modifier");
    assert_eq!(player_hit_damage(&t, 0x0700_0002, 1.5, 0x12, 0xc, 10.0, 0, 6, 0.0, 0.0), 0.0, "voice type 0xC deals nothing");
}

/// `Player_HandlePain` zeroes a health under 1.0 [M 0x1902c8:131-134].
#[test]
fn a_hit_leaving_under_one_point_kills() {
    let mut h = nf_game::bullet::PlayerHealth::default();
    h.health = 1.5;
    let mut rng = nf_game::drone::Rng::new(1);
    // 07000005 Easy: × 0.6 → 0.6 off 1.5 leaves 0.9 < 1 → 0.
    h.pain(&mut rng, 1.0, 1, 0x0700_0005);
    assert_eq!(h.health, 0.0, "0.9 left is a kill");
    // Sister: 1.4 left survives.
    let mut h2 = nf_game::bullet::PlayerHealth::default();
    h2.health = 2.0;
    h2.pain(&mut rng, 1.0, 1, 0x0700_0005);
    assert!((h2.health - 1.4).abs() < 1e-5, "1.4 left survives, got {}", h2.health);
}

/// `behaviour_util_get` 0x1c7a98's stats section: 07000002 placement #442's parameters 15..32 [D the
/// level census] decode to the type-2 records the live table holds ([L] ee_02b.bin `drone_stats` 0x31aa30:
/// kind 0 = 9/0/12/60/2/60/60/0, kind 1 = 8/1/12/80/3/80/80/1, kind 2 = 7/2/12/100/3/100/100/0), and a later
/// placement without the section reads the record the first one wrote. Sister: the old fixed stats
/// (accuracy 5, aggression 0, health 10, reaction 0) are not what a type-2 guard gets.
#[test]
fn the_stats_section_fills_the_shared_table() {
    use nf_formats::drone::{DroneStats, DroneStatsTable};
    let p442: [(u32, u32); 18] = [(15, 2), (16, 0), (17, 0x5b0003), (18, 0xa196958e), (19, 0xecf327e5), (20, 0x40291660), (21, 0),
        (22, 0x5b0003), (23, 0xa196948e), (24, 0xecf327e5), (25, 0x40291660), (26, 0x80010006), (27, 0x11e06009), (28, 0x3c3c),
        (29, 0x1a806108), (30, 0x15050), (31, 0x1b206207), (32, 0x6464)];
    let p443: [(u32, u32); 11] = [(15, 2), (16, 0), (17, 0x5b0003), (18, 0xa196948e), (19, 0xecf327e6), (20, 0x40291660), (21, 0),
        (22, 0x5b0003), (23, 0xa196948e), (24, 0xecf327e7), (25, 0x40291660)];
    let get = |ps: &[(u32, u32)], k: u32| ps.iter().find(|(i, _)| *i == k).map_or(0, |&(_, v)| v);
    let mut t = DroneStatsTable::default();
    let ty = t.behaviour_get(|k| get(&p442, k));
    assert_eq!(ty, 2);
    assert_eq!(t.0[2][0], [9, 0, 12, 60, 2, 60, 60, 0]);
    assert_eq!(t.0[2][1], [8, 1, 12, 80, 3, 80, 80, 1]);
    assert_eq!(t.0[2][2], [7, 2, 12, 100, 3, 100, 100, 0]);
    let want = DroneStats { accuracy: 8, aggression: 1, health: 12, b6: 80, b7: 3, reaction: 80, recover: 80, ba: 1 };
    // #443 carries no section: it reads #442's record.
    let ty2 = t.behaviour_get(|k| get(&p443, k));
    assert_eq!(t.stats(ty2), want);
    // A fresh table (the drone alone): the defaults.
    let mut fresh = DroneStatsTable::default();
    let ty3 = fresh.behaviour_get(|k| get(&p443, k));
    let def = fresh.stats(ty3);
    assert_eq!(def, DroneStats { accuracy: 5, health: 10, ..Default::default() });
    assert_ne!(def, want, "sister: the defaults are not the type-2 record");
}

/// What the type-2 stats change in the firing chain [M 0x179740 / 0x179d40 / 0x179ba8 / 0x126468]: accuracy 8
/// → base 60 (not 75); aggression 1 → the 12-round 0x12 burst × 0.5 = 6 (not × 0.3 = 3) and the burst delay
/// 45 × 1.33 (not × 1.66).
#[test]
fn type_2_guards_fire_at_the_stats_chance() {
    let t = tuning(0x0700_0002);
    // Normal 0.7: (100 − 5·8) × 0.7 = 42; the old acc-5 value 52.5 fails.
    assert!((hit_chance(&t, 0x0700_0002, 2, 8, 10.0, STILL, 0) - 42.0).abs() < 1e-3);
    assert!((hit_chance(&t, 0x0700_0002, 2, 5, 10.0, STILL, 0) - 42.0).abs() > 5.0);
    // Easy 0.5 → 30, Hard 1.0 → 60.
    assert!((hit_chance(&t, 0x0700_0002, 1, 8, 10.0, STILL, 0) - 30.0).abs() < 1e-3);
    assert!((hit_chance(&t, 0x0700_0002, 3, 8, 10.0, STILL, 0) - 60.0).abs() < 1e-3);
}

/// 07000009's lobby guards (the only type-0xD placement kind in the level) really do load with
/// health 1: the level's only stats section sits on drone placement #319 (hash 02000637, params
/// 15..32 [D census]), tag 0x80010006 → three 2-word records. The decoder is pinned by disassembly
/// and live bytes: the widths 0x303330 = 8,3,8,8,5,8,8,1 [D ACTION.ELF], field f → word f through
/// the 0x303350 jump table [D], LSB-first inside the record, the kind advance `(tag&0xffff)/3`
/// words [M 0x1c7a98 disasm 0x1c7cac `sll t2,t4,2` + 0x1c7d74 `addu s0,s0,t2`], the whole field
/// `& 0xff` [M 0x1c7d00]. Applied to #319's words the type-0xD records are k0 60/0/1/60/2/60/60/0,
/// k1 80/1/1/80/3/80/80/1, k2 100/2/1/100/3/100/100/0 — the same decode that reproduces the live
/// drone_stats byte-for-byte on the shared snapshots (ee_start_07000001.bin @0x31aa30 type 2 =
/// 9/0/12/60/2/60/60/0 | 8/1/12/80/3/80/80/1 | 7/2/12/100/3/100/100/0 [L], this file's #442 test).
/// DoModeSettingsNEW reads kind 1 only [M 0x14c704 `addiu a1,zero,1`] and stores record word 2 to
/// +0xAC as `(float)(int)w` [M 0x14c70c-0x14c730: `lw 8(a0)` / `cvt.s.w` / `swc1 172(s4)`] — raw
/// health, not an index: `NDrone2_HitDamage` 0x145558 subtracts the damage from +0xAC and the
/// `> 0` test at :156 is the survival gate; nothing rescales +0xAC on the non-captain path [M
/// 0x14b300:556-561 is the captain branch, and 0xD is not a captain kind 1/2/3/6/7]. Every 09
/// type-0xD placement runs DoModeSettingsNEW (param 5 = 0x64 ≥ 0x24 [D census]; [M 0x14b300
/// disasm 0x14bccc-d4: `sltiu v0,a1,36` on `*(setup+0x48)` = param 5]) and #319 precedes
/// #321/#322/#323/#507/#518 in placement order, so all six read #319's kind-1 record: accuracy 80,
/// health 1. Sister: accuracy 80 sends `DoBulletAccuracy`'s p to −300 — the guards always take the
/// wobble arm (never the aimed zero-offset), the same legal negative-p the f32 chain handles.
#[test]
fn the_07000009_type_0xd_section_decodes_health_1() {
    use nf_formats::drone::{DroneStats, DroneStatsTable};
    let p319: [(u32, u32); 18] = [(15, 0xd), (16, 0), (17, 0x5b0003), (18, 0xa0000000), (19, 0xccf007a1), (20, 0x41b92400), (21, 3),
        (22, 0x5b0003), (23, 0x20000000), (24, 0xccb00013), (25, 0x5800000), (26, 0x80010006), (27, 0x11e0083c), (28, 0x3c3c),
        (29, 0x1a800950), (30, 0x15050), (31, 0x1b200a64), (32, 0x6464)];
    let get = |k: u32| p319.iter().find(|(i, _)| *i == k).map_or(0, |&(_, v)| v);
    let mut t = DroneStatsTable::default();
    let ty = t.behaviour_get(get);
    assert_eq!(ty, 0xd);
    assert_eq!(t.0[0xd][0], [60, 0, 1, 60, 2, 60, 60, 0], "kind 0 [D #319 words 27/28]");
    assert_eq!(t.0[0xd][1], [80, 1, 1, 80, 3, 80, 80, 1], "kind 1 — the record DoModeSettingsNEW reads [M 0x14c704]");
    assert_eq!(t.0[0xd][2], [100, 2, 1, 100, 3, 100, 100, 0], "kind 2 [D #319 words 31/32]");
    let want = DroneStats { accuracy: 80, aggression: 1, health: 1, b6: 80, b7: 3, reaction: 80, recover: 80, ba: 1 };
    assert_eq!(t.stats(ty), want);
    // The applied health is the +0xAC float a hit must out-damage to kill: 1.0, not 10 (the
    // zero-health-word default) and not a multiplier of anything.
    assert_eq!(t.stats(ty).health, 1);
    // Sister: the old fixed stats (accuracy 5, health 10) are not what a 09 lobby guard gets —
    // and a decode that read health at kind-1 word 0 would give 80, neither of which the section holds.
    let mut fresh = DroneStatsTable::default();
    let no_section = |k: u32| if k < 26 { get(k) } else { 0 };
    let ty3 = fresh.behaviour_get(no_section);
    let def = fresh.stats(ty3);
    assert_eq!(def, DroneStats { accuracy: 5, health: 10, ..Default::default() }, "without the section: the defaults [M 0x14c710-14]");
    assert_ne!(def, want);
    // Accuracy 80 drives p negative — the chain must survive it (f32, no underflow panic).
    let t9 = tuning(0x0700_0009);
    assert!(hit_chance(&t9, 0x0700_0009, 2, 80, 10.0, STILL, 0) < 0.0, "(100 − 5·80) × 0.8 < 0");
}
