//! difficulty3-07000001-pins: everything difficulty 3 (00 Agent) changes on 07000001 against
//! difficulties 1 (Operative) and 2 (Agent), pinned per address.
//!
//! The difficulty reaches the level through GameState+0x28 (0x2a3790) — the menu wheel's pick
//! written at confirm [M C_SBNFDFCTY_Handler 0x216ca8:15 `GameState._40_4_ =
//! difficulty[idx*0x18+0xc]`; rows [L ee_start_07000001.bin 0x2df4c0/0x2df4d8/0x2df4f0: +0xc =
//! 1/2/3, Txt 0x30000a5/a6/a7]] — and 07000001's objects read it through exactly five gates:
//! `Drone_Create` 0x136be0:49 (the placement's param 2 ≤ difficulty), `NDrone2_DefaultInit`
//! 0x14b300:504-535 (the captain criterion: prop 9 on 1, 5 on 2, 8 on 3) and :728-739 (the
//! mode-0x18 TakeXHits float — no 07000001 placement runs mode 0x18 [D param 9 ∈
//! {1,5,14,18,28}]), `Player_HandlePain` 0x1902c8:29-41 (× Plr_DMod_Easy/Normal/Hard),
//! `DroneWeap_DoBulletAccuracy` 0x179740:81-95 (× DroneFiring_Accuracy_Easy/Normal/Hard), and
//! `NDrone2_HitDamage` 0x145558 (Bond→guard × DroneDamage_Easy/Normal/Hard). The remaining
//! decompile readers of GameState+0x28 are outside this level's reach: `Copter_Create` 0x17d340
//! (no 0x2F placement [D census]), `Lock_Create` 0x1ae1f8:67-69 (no 0x2E placement [D census]),
//! `Hint_Create` 0x1b3248:12-25 (the level's only hint #330 carries stage param 2 = 3, a
//! ceiling that never refuses at difficulties 1..3), `SoundTrigger_Create` 0x1b2628:24-40 (its
//! gate reads param 6 = 0 on the level's only trigger #319 → ungated), `Player_RamLoad`
//! 0x1957a8 (the restart boost: all three rows 50.0 [L ee_start_07000001.bin 0x30cc34-3c]),
//! `PlrStat_GetScore` 0x1984b0 (the score page, not the sim), `Player_InitWeapon` 0x1a7ef8:24-35
//! (the MAX_AUTO_AIM_DIST_MIDL/NEAR stores — never read in ACTION.ELF; the auto-aim knob the
//! game reads is Autoaim_*Mul [L 0x30cd5c/60/64 = 1.9/1.0/0.0]), `Boot_GetPTPData` 0x1c8198
//! (boot plumbing) and the five writers. `Pickup_Create` 0x18dda0 never reads GameState+0x28
//! [M whole-decompile grep — not among the 19 GameState._40_4_ files], so no pickup or ammo
//! amount differs.
//!
//! Every load here is `load_with_carry_difficulty` — the front-end's launch path [M], the menu
//! way: no health/ammo writes, no teleport. The walks are walkreal_07000001_live's business
//! (level-01-agent-hall-fight-2 owns that file); this file pins the load-time state only.

use nf_view::scene::Sim;

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

fn load_at(difficulty: u32) -> Sim {
    let iso = iso();
    let scene = nf_view::load_with_carry_difficulty(&iso, "07000001.bin", false, None, difficulty)
        .expect("load 07000001");
    scene.sim.expect("sim")
}

/// The gold-medal `bonus[0]` word for basemap 0x0700001 [M PlrStats_HasGoldMedal 0x199a40] —
/// the same builder the interactables kind-6 test uses [L ee_start_07000001.bin 0x30cba0:
/// BaseMap 0x7000001].
fn gold_bits() -> u64 {
    let mut p = nf_game::progress::Progress::default();
    p.set_level_bonus(0x0700_0001, 3, 0);
    p.bonus[0]
}

/// One roster row: (placement, skin, weapon, accuracy, aggression, health, captain).
type Row = (usize, u32, u32, u8, u8, f32, bool);

fn roster(s: &Sim) -> Vec<Row> {
    s.drones
        .list
        .iter()
        .map(|d| {
            (
                d.placement,
                d.skin,
                d.weapon,
                d.accuracy,
                d.aggression,
                d.health,
                d.captain,
            )
        })
        .collect()
}

/// 07000001's full load-time guard table, read off the three live loads (the values below are
/// the pin). The disc columns: skin = placement param 0 [D], weapon/behaviour words = params
/// 1/9/17.. [D]; the stat columns (accuracy 8 / aggression 1 / health 12; the contact 9/1/10)
/// are the level's drone_stats table [M behaviour_util_get 0x1c7a98 over the placements' stats
/// sections]. The captain criterion [M 0x14b300:504-535] then reskins kind-1 guards to
/// 0x05000038, doubles the health (× DroneCaptain_Mod_Health 2.0 [L 0x30cb2c]) and halves the
/// accuracy stat (× 1/Mod_BulletAccuracy 2.0, truncated [L 0x30cb28]).
const D1: [Row; 20] = [
    (158, 0x0500_0036, 0x0e, 8, 1, 12.0, false),
    (161, 0x0500_004f, 0x01, 9, 1, 10.0, false), // the contact
    (162, 0x0500_0036, 0x12, 8, 1, 12.0, false),
    (163, 0x0500_0036, 0x12, 8, 1, 12.0, false),
    (164, 0x0500_003a, 0x12, 8, 1, 12.0, false),
    (165, 0x0500_0039, 0x1c, 8, 1, 12.0, false),
    (166, 0x0500_0036, 0x1c, 8, 1, 12.0, false),
    (167, 0x0500_0038, 0x1c, 4, 1, 24.0, true), // captain on Operative already (prop 9 [D])
    (168, 0x0500_0039, 0x12, 8, 1, 12.0, false),
    (169, 0x0500_003a, 0x0e, 8, 1, 12.0, false),
    (182, 0x0500_0039, 0x12, 8, 1, 12.0, false),
    (249, 0x0500_003a, 0x0e, 8, 1, 12.0, false),
    (250, 0x0500_0036, 0x12, 8, 1, 12.0, false),
    (261, 0x0500_0039, 0x12, 8, 1, 12.0, false),
    (270, 0x0500_0039, 0x0e, 8, 1, 12.0, false),
    (271, 0x0500_0036, 0x0e, 8, 1, 12.0, false),
    (278, 0x0500_003a, 0x12, 8, 1, 12.0, false),
    (294, 0x0500_0039, 0x12, 8, 1, 12.0, false),
    (298, 0x0500_003a, 0x0e, 8, 1, 12.0, false),
    (299, 0x0500_0039, 0x12, 8, 1, 12.0, false),
];

/// Agent (difficulty 2) adds the prop-5 captain: #163 joins #167.
const D2: [Row; 20] = {
    let mut r = D1;
    r[3] = (163, 0x0500_0038, 0x12, 4, 1, 24.0, true);
    r
};

/// 00 Agent (difficulty 3) adds the prop-8 captain: #165 joins #163 and #167. Every other row
/// — skins, weapons, stats — is byte-identical to Operative.
const D3: [Row; 20] = {
    let mut r = D2;
    r[5] = (165, 0x0500_0038, 0x1c, 4, 1, 24.0, true);
    r
};

/// The difficulty lands in the sim, and the spawned set differs across 1/2/3 ONLY in the
/// captain criterion — the level's own Drone_Create 0x136be0:49 gate never fires differently:
/// every kind-0x0F placement on the created map carries param 2 = 1 [D] (admitted at
/// difficulty 1 already). The one kind-0x0F elsewhere in the archive — map 12 placement 5,
/// param 2 = 2 — is dead data: the game's dynamic-create pass parses only the LAST map's 0x1A
/// block [M parsemap_handle_block_id 0x1d1020 case 0x1a `DynamicPtr = FileNextBlock`;
/// 0x1d1cf0 the per-file loop runs the pass once; 0x1d6360:146 — map 53 = main 01000093 is
/// 07000001.bin's last map, so map 12's record never reaches the dispatcher].
#[test]
fn the_spawned_set_is_identical_at_difficulties_1_2_and_3() {
    let s1 = load_at(1);
    let s2 = load_at(2);
    let s3 = load_at(3);
    assert_eq!(s1.drones.difficulty, 1, "GameState+0x28 == 1 on the Operative load [M 0x216ca8:15]");
    assert_eq!(s2.drones.difficulty, 2);
    assert_eq!(s3.drones.difficulty, 3);
    assert_eq!(s3.drones.progress.difficulty, 3, "the Progress mirror follows [M 0x1957a8:23]");

    for s in [&s1, &s2, &s3] {
        assert_eq!(s.drones.list.len(), 20, "the 20 created-map kind-0x0F placements [D census]");
    }
    // The placement indices in create order [M 0x1d04f0 placement order].
    let placements: Vec<usize> = s1.drones.list.iter().map(|d| d.placement).collect();
    assert_eq!(
        placements,
        vec![158, 161, 162, 163, 164, 165, 166, 167, 168, 169, 182, 249, 250, 261, 270, 271, 278, 294, 298, 299],
        "the kind-0x0F placements in disc order [D]"
    );
    // The full tables, row-exact.
    assert_eq!(roster(&s1), D1, "the Operative roster");
    assert_eq!(roster(&s2), D2, "the Agent roster — only #163's captain flip");
    assert_eq!(roster(&s3), D3, "the 00 Agent roster — #165 joins; nothing else moves");

    // The spawner's five group-1 templates (168/169/261/298/299, param 4 = group 1 [D]; spawner
    // #297 param 1 = 1 [D]) ride in the roster at every difficulty — their param 2 = 1 admits
    // them everywhere, so the ch-0xB reinforcement wave exists on 00 Agent exactly as on
    // Operative [M DroneSpawner_SpawnDrone 0x137cb0 reads no difficulty].
    for p in [168usize, 169, 261, 298, 299] {
        assert!(D3.iter().any(|r| r.0 == p), "template #{p} in the 00 Agent roster");
    }
    // The contact (#161, skin 0x0500004F [D param 0]) is never a captain — he is the escorted
    // ally whose death raises the mission-fail channel (param 12 = 96 = 0x60 [D]).
    assert!(!D1[1].6 && !D2[1].6 && !D3[1].6, "the contact never captains");
}

/// The tuning rows at load (level-group values — the SAME table at every difficulty; the
/// difficulty acts through the readers, pinned below) and the three in-level reader selections.
#[test]
fn the_difficulty_rows_and_their_readers() {
    let s1 = load_at(1);
    let s3 = load_at(3);
    // Pain rows [L ee_start_07000001.bin 0x30cc50/54/58 = 0.5/0.7/1.2 — the 0x07000001..04
    // group [M ReadTuningVars 0x1d3a80 0x1d3e00]].
    assert_eq!(s1.tuning.Plr_DMod_Easy.to_bits(), 0.5f32.to_bits());
    assert_eq!(s1.tuning.Plr_DMod_Normal.to_bits(), 0.7f32.to_bits());
    assert_eq!(s1.tuning.Plr_DMod_Hard.to_bits(), 1.2f32.to_bits());
    // Accuracy rows [L ee_start_07000001.bin 0x30caf0/f4/f8 = 0.5/0.7/1.0].
    assert_eq!(s1.tuning.DroneFiring_Accuracy_Easy.to_bits(), 0.5f32.to_bits());
    assert_eq!(s1.tuning.DroneFiring_Accuracy_Normal.to_bits(), 0.7f32.to_bits());
    assert_eq!(s1.tuning.DroneFiring_Accuracy_Hard.to_bits(), 1.0f32.to_bits());
    // Bond→guard damage rows [L ee_start_07000001.bin 0x30cab0/b4/b8 = 1.5/1.0/0.8 — the Easy
    // 1.5 is the ELF static, the 01-04 group writes only Normal/Hard [M 0x1d3a80]].
    assert_eq!(s1.tuning.DroneDamage_Easy.to_bits(), 1.5f32.to_bits());
    assert_eq!(s1.tuning.DroneDamage_Normal.to_bits(), 1.0f32.to_bits());
    assert_eq!(s1.tuning.DroneDamage_Hard.to_bits(), 0.8f32.to_bits());
    // The TooClose band [L 0x30cb14/b18/b1c = 3.0/2.0/2.0] — difficulty-independent.
    assert_eq!(s1.tuning.DroneFiring_TooClose_Distance.to_bits(), 3.0f32.to_bits());
    assert_eq!(s1.tuning.DroneFiring_TooClose_Accuracy.to_bits(), 2.0f32.to_bits());
    assert_eq!(s1.tuning.DroneFiring_TooClose_Damage.to_bits(), 2.0f32.to_bits());
    // The captain mods [L 0x30cb24/b28/b2c = 2.0/2.0/2.0].
    assert_eq!(s1.tuning.DroneCaptain_Mod_BulletDamage.to_bits(), 2.0f32.to_bits());
    assert_eq!(s1.tuning.DroneCaptain_Mod_BulletAccuracy.to_bits(), 2.0f32.to_bits());
    assert_eq!(s1.tuning.DroneCaptain_Mod_Health.to_bits(), 2.0f32.to_bits());
    // The restart boost rows — all three 50.0, so a death-restart costs nothing extra at d3
    // [L 0x30cc34-3c; M Player_RamLoad 0x1957a8:21-34].
    assert_eq!(s1.tuning.ContinueHealthBoostEasy.to_bits(), 50.0f32.to_bits());
    assert_eq!(s1.tuning.ContinueHealthBoostMedium.to_bits(), 50.0f32.to_bits());
    assert_eq!(s1.tuning.ContinueHealthBoostHard.to_bits(), 50.0f32.to_bits());
    // The rows are level-level, not difficulty-level: d3's table equals d1's.
    assert_eq!(s1.tuning.Plr_DMod_Hard.to_bits(), s3.tuning.Plr_DMod_Hard.to_bits());
    assert_eq!(s1.tuning.DroneFiring_Accuracy_Hard.to_bits(), s3.tuning.DroneFiring_Accuracy_Hard.to_bits());

    // READER 1 — the pain scale Bond takes [M Player_HandlePain 0x1902c8:29-41: ×Plr_DMod_Easy
    // on 1, Normal on 2, Hard on 3]: the port's level-group factor.
    assert_eq!(nf_game::bullet::damage_factor(0x0700_0001, 1), 0.5);
    assert_eq!(nf_game::bullet::damage_factor(0x0700_0001, 2), 0.7);
    assert_eq!(nf_game::bullet::damage_factor(0x0700_0001, 3), 1.2);

    // READER 2 — the guards' hit chance [M DroneWeap_DoBulletAccuracy 0x179740:48-51+81-95]. A
    // guard with accuracy stat 5 at 10 u, target still: p = (100 − 5·5) × row.
    use nf_game::drone::{hit_chance, Motion};
    let base = |difficulty: u32| {
        hit_chance(&s1.tuning, 0x0700_0001, difficulty, 5, 10.0, Motion::default(), 0)
    };
    assert_eq!(base(1), 75.0 * 0.5, "Operative × Accuracy_Easy [L 0x30caf0]");
    assert_eq!(base(2), 75.0 * 0.7, "Agent × Accuracy_Normal [L 0x30caf4]");
    assert_eq!(base(3), 75.0 * 1.0, "00 Agent × Accuracy_Hard [L 0x30caf8]");
    // Inside the 3.0 u band the ×2.0 TooClose accuracy lands before AND after the difficulty
    // factor [M 0x179740:49-51 and :97-99] — at d3 the product is 300: every round fires
    // (compared against Rand_FRand(100) [M :100-101]).
    let close = hit_chance(&s1.tuning, 0x0700_0001, 3, 5, 2.5, Motion::default(), 0);
    assert_eq!(close, 300.0, "00 Agent TooClose: 75 × 2.0 (early) × 1.0 (the row) × 2.0 (again)");

    // READER 3 — Bond's damage to guards [M NDrone2_HitDamage 0x145558: the final block —
    // ×DroneDamage_Easy on 1, Normal on 2, Hard on 3]. A 10.0 torso hit on a plain guard:
    let target = nf_game::damage::Target { damageable: true, armour: 0, mode: 0, skin: 0x0500_0036, kind: 0 };
    let hit = |difficulty: u32| {
        nf_game::damage::hit_damage(&s1.tuning, target, 10.0, 4, 0x0e, false, difficulty, 0x0700_0001).loss
    };
    assert_eq!(hit(1), 15.0, "Operative: 10 × Torso 1.0 × DroneDamage_Easy 1.5 [L 0x30cab0]");
    assert_eq!(hit(2), 10.0, "Agent: × DroneDamage_Normal 1.0 [L 0x30cab4]");
    assert_eq!(hit(3), 8.0, "00 Agent: × DroneDamage_Hard 0.8 [L 0x30cab8]");
}

/// No lock, pickup, ammo, hint or sound-trigger delta — the level carries none of the gated
/// kinds and the ungated ones create identically at 1/2/3.
#[test]
fn no_lock_pickup_ammo_hint_or_trigger_delta() {
    let s1 = load_at(1);
    let s2 = load_at(2);
    let s3 = load_at(3);
    // Pickups: the four ungated placements (armour #275/#326 kind 3 id 71 50, ammo #324/#325
    // kind 1 id 6 20 rounds [D params]) — the two kind-6 "Pickup 007" bonus pickups are
    // gold-medal gated [M Pickup_Create 0x18dda0:93-95], not difficulty gated.
    for s in [&s1, &s2, &s3] {
        assert_eq!(s.drones.pickups.len(), 4, "armour ×2 + ammo ×2 at every difficulty [D; M 0x18dda0]");
    }
    let mut armour = 0;
    let mut ammo = 0;
    for p in &s3.drones.pickups {
        if p.kind == 3 { armour += 1; assert_eq!(p.rounds, 50, "armour amount 50 [D #275/#326 param 2]"); }
        if p.kind == 1 { ammo += 1; assert_eq!(p.rounds, 20, "ammo amount 20 [D #324/#325 param 2]"); }
    }
    assert_eq!((armour, ammo), (2, 2));
    // The medal cross: with the gold word carried, the two kind-6 placements create on the
    // same tick schedule at d1 and d3 alike — the gate reads the medal word only [M 0x18dda0:
    // 93-95]. (The kind-6 rows appear once the level's intro window has run — the same state
    // the interactables kind-6 test polls after `past_intro`.)
    let mut m1 = nf_view::load_with_carry_medals(&iso(), "07000001.bin", false, None, 1, gold_bits())
        .expect("gold load d1");
    let mut m3 = nf_view::load_with_carry_medals(&iso(), "07000001.bin", false, None, 3, gold_bits())
        .expect("gold load d3");
    for scene in [&mut m1, &mut m3] {
        for _ in 0..4200 {
            let sim = scene.sim.as_mut().unwrap();
            // The walker's documented cut-scene stand-in (common/walker.rs tick_inner): the
            // intro's NIS holds the world `disabled` until skipped; the request lets the
            // script player finish so ch 0xB fires and the level's real load state shows.
            if sim.disabled { sim.drones.scripts.skip_request = true; }
            sim.step_bond(Default::default(), false, false, false);
        }
    }
    let count = |s: &Sim| s.drones.pickups.len();
    assert_eq!(count(m1.sim.as_ref().unwrap()), 6, "the medal load creates the two kind-6 pickups at d1");
    assert_eq!(count(m3.sim.as_ref().unwrap()), 6, "…and identically at d3 — no difficulty term");
    assert_eq!(count(m1.sim.as_ref().unwrap()), count(m3.sim.as_ref().unwrap()));

    // Hints: the level's only hint #330 carries stage param 2 = 3 — the Hint_Create 0x1b3248
    // ceiling that never refuses at difficulties 1..3 [M :12-25; D #330 params (2, 3)] — so the
    // hint exists at 1, 2 and 3 alike.
    assert_eq!(nf_game::player::hint_stage_gates(3, 1), true);
    assert_eq!(nf_game::player::hint_stage_gates(3, 2), true);
    assert_eq!(nf_game::player::hint_stage_gates(3, 3), true);
    let hints = |s: &Sim| s.hints.len();
    assert_eq!(hints(&s1), hints(&s2), "hint set identical d1/d2 [M 0x1b3248 stage 3 ceiling]");
    assert_eq!(hints(&s1), hints(&s3), "hint set identical d1/d3");
    assert!(hints(&s1) >= 1, "hint #330 created [D census; M 0x1b3248 param 2 = 3 never refuses 1..3]");

    // Sound triggers: the level's only trigger #319 carries gate param 6 = 0 — ungated [M
    // 0x1b2628:24-40; D #319 params (6, 0)] — the same single trigger at every difficulty.
    let trigs = |s: &Sim| s.drones.sound_triggers.list.len();
    assert_eq!(trigs(&s1), trigs(&s2));
    assert_eq!(trigs(&s1), trigs(&s3));
    assert_eq!(trigs(&s1), 1, "#319 created [D; M 0x1b2628 gate param 6 = 0]");

    // Locks: 07000001 has no 0x2E placement [D census] — Lock_Create's Operative-only
    // auto-unlock [M 0x1ae1f8:67-69] has no object to act on; the port's lock list is empty
    // at every difficulty.
    assert_eq!(s1.drones.locks.len(), 0, "no 0x2E placements [D census]");
    assert_eq!(s3.drones.locks.len(), 0);
}

/// Bond's own state at load is difficulty-blind: the same health, armour and loadout — the
/// only difficulty code in `Player_InitWeapon` 0x1a7ef8 is the MAX_AUTO_AIM_DIST_MIDL/NEAR
/// stores (:24-35, 18/12 on 1, 14/8 on 2, 0/0 on 3), and those globals are never read in
/// ACTION.ELF; the auto-aim knob the game actually reads is Autoaim_*Mul [L 0x30cd5c/60/64 =
/// 1.9/1.0/0.0].
#[test]
fn bond_loads_identically_at_every_difficulty() {
    let s1 = load_at(1);
    let s2 = load_at(2);
    let s3 = load_at(3);
    for s in [&s1, &s2, &s3] {
        assert_eq!(s.drones.health.health, 100.0, "the cold load's health [M Player_Init 0x1910f8:128-138]");
        assert_eq!(s.drones.health.armour, 0.0);
        // The loadout switch rides as the request (the hands record 0x47 is the pre-switch
        // holder [M Player_WeaponNone's id]; the per-level loadout switch requests the
        // level-start pistol [M Player_InitWeapon 0x1a7ef8's 07000001 arm]).
        assert_eq!((s.gun.weapon, s.gun.switch_request), (0x47, 2), "hands held, pistol requested");
    }
    assert_eq!((s1.gun.clip, s1.gun.spare), (s3.gun.clip, s3.gun.spare), "ammo identical d1 vs d3");
    // The auto-aim multipliers [L ee_start_07000001.bin 0x30cd5c/60/64]: 1.9 on Operative,
    // 1.0 on Agent, 0.0 (off) on 00 Agent — the difficulty the player FEELS in the aim.
    assert_eq!(s1.tuning.Autoaim_EasyMul.to_bits(), 1.9f32.to_bits());
    assert_eq!(s1.tuning.Autoaim_NormalMul.to_bits(), 1.0f32.to_bits());
    assert_eq!(s1.tuning.Autoaim_HardMul.to_bits(), 0.0f32.to_bits());
}

