//! The kind-6 pickups' gold-medal gate (task pickup-medal-gate). The medal-state math runs
//! headless; the create-gate half rides the disc's `weapon_data` like `pickup_live` does
//! (opt-in: set `NF_ISO`).
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] the EE snapshot.

use nf_game::progress::{has_gold_medal, medal_set, Progress, REWARDS_TABLE};

/// `PlrStats_HasGoldMedal` 0x199a40's state: the gold (counter 3) bit of `hash` in the bonus
/// word, read through `Menu_HasMedal` 0x202580 → `PlrStarts_ProcessRewardCounter` 0x199210.
/// The award arm is `Menu_SetLevelBonus` 0x1ffa10 (the port's `set_level_bonus`).
#[test]
fn the_gold_medal_bit_is_the_rewards_tables_counter_3_bit() {
    // No bits → no medal, for every campaign row.
    for e in REWARDS_TABLE.iter() {
        assert!(!has_gold_medal(0, e.hash), "zero bonus word → no gold medal for {:08x}", e.hash);
    }
    // The award arm lights the level's own gold bit and nothing else's.
    let mut p = Progress::default();
    assert!(!p.has_medal(0x0700_0011, 3, 0), "no medal before the award [M 0x199210 get arm]");
    p.set_level_bonus(0x0700_0011, 3, 0); // [M 0x1ffa10 — the results page's award]
    assert!(p.has_medal(0x0700_0011, 3, 0), "the gold medal recorded [M 0x199210]");
    assert!(has_gold_medal(p.bonus[0], 0x0700_0011), "HasGoldMedal's read over the same word [M 0x199a40]");
    // The gold bits are the 0x0700005/9/11 kind-4 group (bit 0 + rank [L ee_start_05 0x2bc920 rows
    // 1/5/7 desc[3] all (bit 0, kind 4); M 0x199210's packed-word scan]): 0x07000011's award lights
    // rank 2, so the sister levels' golds stay dark, and a bronze/silver-only run is not gold.
    assert!(!has_gold_medal(p.bonus[0], 0x0700_0005), "07000005's gold is the group's rank 0 — a different bit");
    assert!(!has_gold_medal(p.bonus[0], 0x0700_0009), "07000009's gold is the group's rank 1 — a different bit");
    let mut bronze = Progress::default();
    bronze.set_level_bonus(0x0700_0011, 1, 0);
    assert!(!has_gold_medal(bronze.bonus[0], 0x0700_0011), "bronze only → HasGoldMedal false [M 0x199a40 reads counter 3]");
}

/// The level the gate reads is the MISSION layer's BaseMap, not the archive id: `Mission_Init`
/// 0x182270 walks the 24-record mission table (the disasm pairing: the mission id at
/// 0x2a4350+k·0x28, the map hcode at +4) and `Mission_BaseMapHCode` 0x1830a0 returns it —
/// mission 0700000a plays under BaseMap 0x07000009 and 07000011 under 0x07000011 [L
/// ee_start_05.bin 0x2a4350, 24 × 0x28 records]. The port's `crate::mission::LEVELS` carries the
/// same rows, and the kind-6 levels' BaseMaps both sit in the RewardsTable.
#[test]
fn the_gate_reads_the_mission_tables_basemap() {
    let gold_of = |level: u32| {
        nf_game::mission::LEVELS.iter().find(|l| l.level == level)
            .map(|l| (l.mission, has_gold_medal(0, l.mission), {
                let mut p = Progress::default();
                p.set_level_bonus(l.mission, 3, 0);
                has_gold_medal(p.bonus[0], l.mission)
            }))
    };
    // The two kind-6 placement levels [D 0700000a/07000011.bin rows]: BaseMap found, gated shut
    // without the medal, open with it.
    assert_eq!(gold_of(0x0700_000a), Some((0x0700_0009, false, true)), "0700000a → BaseMap 0x07000009 [L 0x2a4350 row 8]");
    assert_eq!(gold_of(0x0700_0011), Some((0x0700_0011, false, true)), "07000011 → BaseMap 0x07000011 [L 0x2a4350 row 15]");
    // A level with no mission row (the front end 0x07000048) plays BaseMap −1 → no RewardsTable
    // row → the gate stays shut, as the game's [M 0x182270 → state 5; 0x199210 no-row arm].
    assert_eq!(gold_of(0x0700_0048), None);
    // No RewardsTable row → false at every kind (the info stays zeroed [M 0x199210]).
    assert!(!medal_set(0, 0x0700_000e, 3), "0x0700000e is not a scored map [L 0x2bc920 rows]");
    assert!(!medal_set(u64::MAX, 0x0700_000e, 3), "…even with every bit set — the row lookup fails first");
    // The game's kind guard `if (param_3 < 5)` [M 0x199210:24]: kind 5+ reads nothing.
    assert!(!medal_set(u64::MAX, 0x0700_0011, 5), "kind 5 → no read [M 0x199210:24]");
}

/// `Pickup_Create`'s gate [M 0x18dda0:88-104 — the order: the glist check, the id −1 check, then
/// `if ((uVar7 == 6) && (PlrStats_HasGoldMedal__Fv(), lVar3 == 0)) return 0;` BEFORE the 7..=11
/// set intercept]. A kind-6 placement with no gold medal creates NOTHING; with the medal it
/// creates the spinning bonus pickup. Opt-in: `NF_ISO`.
#[test]
fn the_kind6_pickup_creates_only_with_the_gold_medal() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = nf_formats::iso9660::IsoImage::open(&path).unwrap();
    let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
    let wd = nf_formats::weapon::WeaponData::from_action_elf(&elf).unwrap();
    let models = std::collections::HashMap::new();
    let at = [1.0, 2.0, 3.0];
    // A kind-6 row of the kind the two levels place (the model hash stands in for the placement's
    // celglist — the gate sits before any model use [M 0x18dda0:93-95]).
    let model = wd.model(6);
    let bounds = nf_game::pickup::Model { centre: [0.0; 3], radius: 0.5 };
    // The no-medal arm: the game returns 0 — no pickup object at all.
    assert!(nf_game::pickup::Pickup::create_placed(at, [0.0; 3], nf_game::pickup::kind::BOND_BONUS, 6, 0, 0, 0xFFFF, 0, 0, model, bounds, &wd, &models, false).is_none(),
        "no gold medal → Pickup_Create returns 0 for kind 6 [M 0x18dda0:93-95]");
    // The medal arm: the create proceeds (the kind-6 spin flag is `create_placed`'s own [M
    // 0x18dda0:138-144 `uVar7 == 6` → +0x38 = 1]).
    let p = nf_game::pickup::Pickup::create_placed(at, [0.0; 3], nf_game::pickup::kind::BOND_BONUS, 6, 0, 0, 0xFFFF, 0, 0, model, bounds, &wd, &models, true)
        .expect("the gold medal opens the gate [M 0x18dda0:93-95 else-arm]");
    assert!(p.spin, "kind 6 spins [M 0x18dda0:138-144]");
    // The gate is kind-6-only: every other kind creates with the same no-medal word.
    let wmodel = wd.model(0x1c);
    assert!(nf_game::pickup::Pickup::create_placed(at, [0.0; 3], nf_game::pickup::kind::WEAPON, 0x1c, 5, 0, 0xFFFF, 0, 0, wmodel, bounds, &wd, &models, false).is_some(),
        "kind 0 is not gated [M 0x18dda0:93 — uVar7 == 6 is the only medal read]");
    // …and the id −1 check still runs first [M 0x18dda0:89-91].
    assert!(nf_game::pickup::Pickup::create_placed(at, [0.0; 3], nf_game::pickup::kind::BOND_BONUS, 0xFFFF, 0, 0, 0xFFFF, 0, 0, model, bounds, &wd, &models, true).is_none(),
        "id 0xFFFF refuses even with the medal [M 0x18dda0:89-91]");
}
