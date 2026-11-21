//! g029-pickups: the gold-medal weapon upgrades and the pickup pane-fade rule, on the real level
//! loads.
//!
//! 1. `Menu_SetBonus` 0x1ff898 rebuilds the player-0 `Upgrades` row from the bonus word (a gold
//!    reward of kind 4 sets `Upgrades[bit / 3]` to its group's set count [M :73-98, 0x199210,
//!    0x1a98a8]), and `Player_InitWeapon` 0x1a7ef8 equips `Upgrade_Weapon(6)` / `(0x43)` / ... off
//!    that row [M :126-133]. Every handgun level ships the four handgun first-person models
//!    (0x05000088 / 0x050000a9 / 0x05000047 / 0x050000a8) [D the level archives' skin tables], so
//!    an upgraded kit is loadable. The port used to pass upgrade level 0 everywhere: a profile
//!    with the golds still started with the base records.
//! 2. `Pickup_Handler` 0x18e650 resets the health pane's fade (`BLData+0x8bc = 1.0`) only in its
//!    armour case [M :220-226]; the port reset it on every take.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The handgun gold group's first bit: 07000005's gold (bit 0 [L RewardsTable 0x2bc920]).
const HANDGUN_GOLD: u64 = 1 << 0;
/// 0700000C's gold: bit 12, the dart-gun byte `Upgrades[4]` [L RewardsTable].
const DART_GOLD: u64 = 1 << 12;

#[test]
fn a_handgun_gold_starts_07000005_with_the_upgraded_handgun() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let up = nf_view::load_with_carry_medals(&iso, "07000005.bin", false, None, 1, HANDGUN_GOLD).expect("load");
    let sim = up.sim.as_ref().unwrap();
    assert_eq!(sim.drones.inventory.upgrades[0], 1, "one handgun gold → Upgrades[0] = 1");
    assert_eq!(sim.gun.switch_request, 4, "the kit's request is Upgrade_Weapon(6) = UpgradedHandguns[1] = 4 [M 0x1a7ef8:126,170]");
    assert!(sim.drones.inventory.owned[4], "record 4 is owned");
    assert!(!sim.drones.inventory.owned[2], "and the base P2K record is not (the kit equips the upgraded id)");
    // Sister: the fresh profile keeps the base handgun.
    let base = nf_view::load_with_carry_medals(&iso, "07000005.bin", false, None, 1, 0).expect("load");
    let sim = base.sim.as_ref().unwrap();
    assert_eq!(sim.gun.switch_request, 2, "no golds → the P2K 2");
    assert!(sim.drones.inventory.owned[2] && !sim.drones.inventory.owned[4]);
}

#[test]
fn a_dart_gold_starts_07000009_with_the_upgraded_dart_gun() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let up = nf_view::load_with_carry_medals(&iso, "07000009.bin", false, None, 1, DART_GOLD).expect("load");
    let sim = up.sim.as_ref().unwrap();
    assert_eq!(sim.drones.inventory.upgrades[4], 1);
    assert_eq!(sim.gun.switch_request, 0x44, "the island kit holds Upgrade_Weapon(0x43) = 0x44 [M 0x1a7ef8 case 0x7000009]");
    assert!(sim.drones.inventory.owned[0x44] && !sim.drones.inventory.owned[0x43]);
    assert!(sim.drones.inventory.owned[2], "sister: the dart reward leaves the handgun at the P2K");
    let base = nf_view::load_with_carry_medals(&iso, "07000009.bin", false, None, 1, 0).expect("load");
    assert_eq!(base.sim.as_ref().unwrap().gun.switch_request, 0x43, "no golds → 0x43");
}

/// Stand Bond on pickup `k`'s placed spot until a take happens (the armour test's ring pattern).
fn take_at(scene: &mut nf_view::scene::Scene, k: usize) -> Option<nf_game::pickup::Taken> {
    let spot = {
        let sim = scene.sim.as_ref().unwrap();
        let m = sim.placed_pickup_pose(k).expect("a placed instance");
        [m[3][0], m[3][1], m[3][2]]
    };
    for off in [[0.0f32, 1.2, -0.6], [0.0, 1.2, 0.6], [0.6, 1.2, 0.0], [-0.6, 1.2, 0.0], [0.0, 0.9, 0.0]] {
        let before = scene.sim.as_ref().unwrap().drones.taken.len();
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.bond = nf_game::player::Bond::stand_at(&sim.world, [spot[0] + off[0], spot[1] + off[1], spot[2] + off[2]], 0.0);
            sim.drones.pain_alpha = 0.25;
        }
        for _ in 0..60 {
            let sim = scene.sim.as_mut().unwrap();
            sim.step_bond(Default::default(), false, false, false);
            if sim.drones.taken.len() > before { return sim.drones.taken.last().map(|(_, t)| *t); }
        }
    }
    None
}

#[test]
fn an_ammo_take_leaves_the_pane_fade_and_armour_resets_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let (ammo, armour) = {
        let sim = scene.sim.as_ref().unwrap();
        assert!(!sim.drones.decay_gate, "the default pane-fade option is off: pain_alpha only moves by writes");
        let find = |want: &dyn Fn(&nf_game::pickup::Pickup) -> bool| sim.drones.pickups.iter().enumerate()
            .find(|(k, p)| want(p) && !p.gone && sim.placed_pickup_pose(*k).is_some()).map(|(k, _)| k);
        (find(&|p| matches!(p.kind, nf_game::pickup::kind::AMMO | nf_game::pickup::kind::WEAPON) && p.weapon == 6),
         find(&|p| p.kind == nf_game::pickup::kind::ARMOUR))
    };
    let ammo = ammo.expect("07000005 places id-6 ammo boxes");
    let t = take_at(&mut scene, ammo);
    assert!(matches!(t, Some(nf_game::pickup::Taken::Ammo { .. })), "took the ammo box: {t:?}");
    let a = scene.sim.as_ref().unwrap().drones.pain_alpha;
    assert!((a - 0.25).abs() < 1e-6, "an ammo take does not touch BLData+0x8bc [M 0x18e650:220-226]: {a}");
    let armour = armour.expect("07000005 places an armour pickup");
    let t = take_at(&mut scene, armour);
    assert!(matches!(t, Some(nf_game::pickup::Taken::Armour)), "took the armour: {t:?}");
    assert_eq!(scene.sim.as_ref().unwrap().drones.pain_alpha, 1.0, "sister: the armour take resets the fade [M 0x18e650:225]");
}
