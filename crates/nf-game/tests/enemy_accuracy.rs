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
