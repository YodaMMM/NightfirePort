//! mp-pickups-respawn-2: the weapon-set tables, the pickup registry bookkeeping, the spawn
//! policy and the enviro toggles — numbers only (the ELF dumps and the decompiled arithmetic), no
//! ISO needed. Re-landed from the never-merged `s/mp-pickups-respawn` (task 648) onto master's
//! current shapes: the matrix / random row / loadout pins that mp-lobby-settings already landed
//! live in `equip.rs`'s own tests — this file carries the registry, the spawn policy, the
//! toggles, the corrected boot defaults and the botweap stride fix. Evidence tags in the module
//! docs of `nf_game::mp::pickups` / `nf_game::mp::equip` / `nf_game::pickup`.

use nf_game::drone::Rng;
use nf_game::mp::equip::{PICKUP_MATRIX, USEABLE_GUNS};
use nf_game::mp::{car_kind, gun_imp_class, weapon_for_kind, MpMatch, MpSettings};

fn match_with(level: u32) -> MpMatch {
    let mut s = MpSettings::deathmatch(level);
    s.scenario = nf_game::mp::scenario::DEATHMATCH;
    MpMatch::init(s)
}

/// `PickupMatrix` @0x2b9100 (110 bytes = 11 rows × 5 u16, `UseableGuns` @0x2b9170 right behind)
/// are the on-disc bytes [D ACTION.ELF file 0x1b3000/0x1b3070; M the readers index
/// `PickupMatrix + set·10 + (kind−7)·2` in BYTES — Pickup_CreateFromSet 0x18f1b0:56,
/// MP_EquipPlayer 0x1855f8:18, BOTWEAP_InitWeapon 0x129e48:44]. The whole table is pinned here
/// (equip.rs's own tests pin rows 0/10 and the set-1 row through `weapon_row`); a 20-byte-row
/// reading would need 220 bytes and overlap UseableGuns.
#[test]
fn weapon_set_tables_pin_the_elf_values() {
    let expected_matrix: [[u16; 5]; 11] = [
        [0x6, 0x1c, 0x1e, 0x18, 0x2c],
        [0x2, 0x6, 0xa, 0x10, 0x42],
        [0x12, 0x15, 0x18, 0x1c, 0x52],
        [0xe, 0x1e, 0x11, 0x24, 0x32],
        [0x2a, 0x2c, 0x3a, 0x2e, 0x1a],
        [0x35, 0x34, 0x3a, 0x37, 0x3b],
        [0x2, 0x11, 0x37, 0x24, 0x42],
        [0xa, 0x1e, 0x52, 0x1a, 0x32],
        [0x1a, 0x3b, 0x32, 0x3a, 0x2c],
        [0x15, 0x16, 0x37, 0x11, 0x24],
        [0x0, 0x0, 0x0, 0x0, 0x0],
    ];
    assert_eq!(PICKUP_MATRIX, expected_matrix);
    assert_eq!(
        USEABLE_GUNS,
        [
            0x2, 0x6, 0xa, 0xc, 0xe, 0x10, 0x12, 0x14, 0x16, 0x19, 0x1a, 0x1c, 0x1e, 0x24, 0x2a,
            0x2c, 0x2e, 0x33, 0x37, 0x35, 0x36, 0x34, 0x3a, 0x3b, 0x42, 0x11, 0x52
        ]
    );
    // The row-0 alias in pickup.rs (the SP set — MPSettings+0x1B4 = 0 there [L ee_start_05.bin]).
    assert_eq!(nf_game::pickup::PICKUP_MATRIX_ROW0, expected_matrix[0]);
    // The botweap re-export is the SAME table (mp-pickups-respawn-2: its old copy was the first
    // 100 bytes read as 5 rows of 10 u16 — sets interleaved [M 0x129e48:44's byte stride]).
    assert_eq!(nf_game::mp::botweap::PICKUP_MATRIX, expected_matrix);
}

/// `Pickup_CreateFromSet` 0x18f1b0: the weapon is `PickupMatrix[set][(kind-7)]` (byte offsets
/// `set·10 + (kind−7)·2`), set = the u16 at MPSettings+0x1B4 [M :56]. Kinds outside 7..=11 never
/// reach the table (`Pickup_Create` 0x18dda0:104's `uVar7 - 7 < 5` intercept) and the Random set
/// reads the live matrix's re-rolled row 10 [M 0x18f098].
#[test]
fn create_from_set_maps_kinds_per_set() {
    // Set 0 (Normal): kind 7 → 0x6 (P99), 8 → 0x1c, 9 → 0x1e, 10 → 0x18, 11 → 0x2c.
    for (kind, want) in [(7u32, 0x6u16), (8, 0x1c), (9, 0x1e), (10, 0x18), (11, 0x2c)] {
        assert_eq!(weapon_for_kind(&PICKUP_MATRIX[0], kind), Some(want), "set 0 kind {kind}");
    }
    // Set 4 (Explosives 1) row: 0x2a, 0x2c, 0x3a, 0x2e, 0x1a; set 8 ends 0x2c.
    assert_eq!(weapon_for_kind(&PICKUP_MATRIX[4], 7), Some(0x2a));
    assert_eq!(weapon_for_kind(&PICKUP_MATRIX[4], 11), Some(0x1a));
    assert_eq!(weapon_for_kind(&PICKUP_MATRIX[8], 11), Some(0x2c));
    // Out-of-range kinds never reach the table.
    assert_eq!(weapon_for_kind(&PICKUP_MATRIX[0], 6), None);
    assert_eq!(weapon_for_kind(&PICKUP_MATRIX[0], 12), None);
}

/// `MP_RegisterPickup` 0x189d80: sequential slots from `PickupNextAddIndex`, the count at
/// `MPSettings+0x1D8` [M :107], the full ring evicts the smallest non-zero `PICKUPINFO+0x2E`
/// lifetime among the first `count` NON-NULL rows [M :52-73] THROUGH
/// `MP_UnregisterPickup` [M :76] — so the count is net unchanged and
/// `PickupLastDeletedIndex` ends at the reused slot [M 0x18a058:24]. The cursor wraps to 0 only
/// when the slot was > 0x3e [M :103-105].
#[test]
fn register_pickup_ring_eviction_and_count() {
    let mut m = match_with(0x0700_0023);
    for i in 0..64 {
        let slot = m.register_pickup(i, [i as f32, 1.0, 2.0], 0x100 + i as u32, 100 + i as u16);
        assert_eq!(slot, i as usize);
    }
    assert_eq!(m.settings.registered_pickups, 64);
    // Slot 62's next cursor stays 63; slot 63's wraps to 0 [M 0x189d80:103-105].
    assert_eq!(m.pickups.next_add, 0, "slot 63 > 0x3e wraps the cursor");
    // Full: the smallest lifetime (100, slot 0) is evicted; the count is NET unchanged
    // (the game's unregister −1 then register +1 [M :76 + 0x18a058:29]).
    let slot = m.register_pickup(0x1234, [9.0; 3], 0x999, 55);
    assert_eq!(slot, 0);
    assert_eq!(m.pickups.records[0].obj, Some(0x1234));
    assert_eq!(m.settings.registered_pickups, 64, "eviction nets the count out");
    assert_eq!(m.pickups.last_deleted, Some(0), "unregister records the slot [M 0x18a058:24]");
    // Unregister frees the slot and the next add reuses it first [M 0x189d80:29-31].
    m.unregister_pickup(3);
    assert_eq!(m.settings.registered_pickups, 63);
    assert_eq!(m.pickups.last_deleted, Some(3));
    let slot = m.register_pickup(0x77, [1.0; 3], 0x1, 10);
    assert_eq!(slot, 3);
    assert_eq!(m.settings.registered_pickups, 64);
    // Note on the eviction scan's NON-NULL counting [M 0x189d80:52-73]: it is the decompile's
    // literal form but unobservable in reachable states — every null row is findable by the
    // slot search (the forward scan covers next_add..0x40, the wrap scan 1..next_add), so an
    // eviction only fires on a fully-occupied ring where both readings coincide. The port
    // implements the literal scan; there is no reachable discriminator to pin [S].
}

/// `MP_Pickup_Process` 0x18a168: a set stamp survives while the `MPGame+0x19C` clock is behind
/// it and is zeroed once the clock passes it [M :17-26]; exactly 4 stamps per record [M :18-26];
/// the walk visits rows until `count` NON-NULL rows have been seen [M :9-14];
/// `BOTSTATE_setPickupVisitTime` 0x129958 writes clock+45 (+0.1 when that would be 0.0) [M
/// 0x129958:9-15].
#[test]
fn pickup_process_and_visit_stamps() {
    let mut m = match_with(0x0700_0023);
    let s0 = m.register_pickup(1, [0.0; 3], 0, 0);
    let s1 = m.register_pickup(2, [1.0; 3], 0, 0);
    m.pickups.set_visit_time(s0, 0, 100.0);
    m.pickups.set_visit_time(s0, 3, 50.0);
    m.pickups.set_visit_time(s1, 1, 200.0);
    assert_eq!(m.pickups.records[s0].visit[0], 145.0);
    assert_eq!(m.pickups.records[s1].visit[1], 245.0);
    // Clock 144.9: the 145.0 stamp stands, the 95.0 one is behind the clock and zeroes.
    m.pickup_process(144.9);
    assert_eq!(m.pickups.records[s0].visit[0], 145.0);
    assert_eq!(m.pickups.records[s0].visit[3], 0.0);
    // Clock 145.5: the 145.0 stamp zeroes, the 200/245 ones stand.
    m.pickup_process(145.5);
    assert_eq!(m.pickups.records[s0].visit[0], 0.0);
    assert_eq!(m.pickups.records[s1].visit[1], 245.0);
    // The +0.1 guard: clock −45 → stamp 0.0 → +0.1 [M 0x129958:12-14].
    m.pickups.set_visit_time(s1, 2, -45.0);
    assert_eq!(m.pickups.records[s1].visit[2], 0.1);
    // Sister: the walk counts NON-NULL rows — null rows in between do not consume the count.
    // count 1: only the FIRST non-null row is visited, however many null rows precede it.
    let mut m2 = match_with(0x0700_0023);
    let a = m2.register_pickup(1, [0.0; 3], 0, 0); // row 0
    let b = m2.register_pickup(2, [1.0; 3], 0, 0); // row 1
    m2.pickups.set_visit_time(a, 0, 100.0);
    m2.pickups.set_visit_time(b, 0, 100.0);
    m2.unregister_pickup(a); // row 0 null, count 1
    m2.pickup_process(200.0);
    assert_eq!(m2.pickups.records[b].visit[0], 0.0, "the walk reached past the null row");
}

/// `MP_GetSpawnPoint` 0x185098's policy word `MPSettings+0x1C0` (the menu's Near/Far/Random
/// values 0/1/2): 0 = the nearest candidate (> 2.0 squared units from every other player), 1 =
/// the farthest, 2 = `Rand_Rand(candidates)`; the team ranges split at 0x20 for team games [M
/// :85-98, 100-183]. Numbers only. (The port's `get_spawn_point` already matched the decompile —
/// this pins it.)
#[test]
fn spawn_policy_picks_the_games_point() {
    // FFA deathmatch, four spawn points at known distances from one other player.
    let mut m = match_with(0x0700_0023);
    let spots: [[f32; 3]; 4] =
        [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [30.0, 0.0, 0.0], [100.0, 0.0, 0.0]];
    for p in spots.iter() {
        m.register_spawn_point(*p, [0.0, 0.0, 1.0], 0, &no_world());
    }
    // One player at (5,0,0): squared distances 25, 25, 625, 9025 — all 4 are candidates
    // (> 2.0 squared units from every other player).
    m.set_slot(1, [5.0, 0.0, 0.0]);
    let mut rng = Rng::new(3);
    // Far (policy 1): spot 3 (9025).
    m.settings.spawn_policy = 1;
    assert_eq!(m.get_spawn_point(0, 0, &mut rng), 3);
    // Near (policy 0): the smallest sq-dist — spots 0/1 tie at 25; the tracker keeps the FIRST
    // strictly-smaller one, so the first 25 (spot 0) wins and spot 1 never beats it.
    m.settings.spawn_policy = 0;
    assert_eq!(m.get_spawn_point(0, 0, &mut rng), 0);
    // Random (policy 2): a fixed seed picks deterministically among the 4 candidates; the draw
    // is Rand_Rand(4) — replay the same seed against a bare Rng to pin the index.
    m.settings.spawn_policy = 2;
    let mut r2 = Rng::new(11);
    let want = r2.rand(4) as u32;
    assert_eq!(m.get_spawn_point(0, 0, &mut Rng::new(11)), want);
    // The nearest candidate excludes points within 2.0 squared units: park a player ON spot 2
    // (the spawn eye height is floor+1.6, the game's obj+0x30 [M 0x184f20:83]) and the near pick
    // moves off it.
    m.set_slot(2, [spots[2][0], 1.6, spots[2][2]]);
    m.settings.spawn_policy = 0;
    let picked = m.get_spawn_point(0, 0, &mut Rng::new(3));
    assert_ne!(picked, 2, "a spawn point occupied by another player is excluded");
    assert_eq!(picked, 0, "spot 0 (sq-dist 25) is the nearest remaining candidate");
    // Team games split the table at 0x20: with only team-0 points registered, a team-1 spawn in
    // a team game falls back to the range start (0x20) [M 0x185098:165-183].
    let mut t = match_with(0x0700_0023);
    t.settings.scenario = nf_game::mp::scenario::TEAM_DEATHMATCH;
    t.settings.team_game = 1;
    t.register_spawn_point([0.0; 3], [0.0, 0.0, 1.0], 0, &no_world());
    t.settings.spawn_policy = 1;
    assert_eq!(t.get_spawn_point(1, 0, &mut Rng::new(1)), 0x20);
    // FFA uses the whole range [0,0x40) — the same table is visible to team 0.
    assert!(m.get_spawn_point(0, 0, &mut Rng::new(1)) < 0x40);
}

fn no_world() -> nf_game::world::CollisionWorld {
    nf_game::world::CollisionWorld::from_triangles(vec![])
}

/// `Upgrade_MPWeapon` 0x1a9040 with the boot `Upgrades` table (all zero [D the ELF's 0x2beea8 —
/// indices 0..45 read zero in this task's dump]): classes 2/4 → `UpgradedPPK_MP[0]` = 2, 6/8 →
/// `UpgradedP99_MP[0]` = 6, the snipers 0x1e/0x20/0x22 → 0x1e, the silenced 0x24/0x26/0x28 →
/// 0x24, 0x50 → `UpgradedGrapple_MP[0]` = 0x50; anything else unchanged [M 0x1a9040:12-41].
/// (master's `upgrade_mp_weapon` models the boot state only — the per-slot index dimension of
/// the old branch is dropped with it; the pick tables' upgraded entries are pinned here [D].)
#[test]
fn upgrade_mp_weapon_boot_tables() {
    assert_eq!(nf_game::mp::equip::upgrade_mp_weapon(2), 0x2);
    assert_eq!(nf_game::mp::equip::upgrade_mp_weapon(4), 0x2);
    assert_eq!(nf_game::mp::equip::upgrade_mp_weapon(6), 0x6);
    assert_eq!(nf_game::mp::equip::upgrade_mp_weapon(8), 0x6);
    assert_eq!(nf_game::mp::equip::upgrade_mp_weapon(0x50), 0x50);
    assert_eq!(nf_game::mp::equip::upgrade_mp_weapon(0x1e), 0x1e);
    assert_eq!(nf_game::mp::equip::upgrade_mp_weapon(0x20), 0x1e);
    assert_eq!(nf_game::mp::equip::upgrade_mp_weapon(0x24), 0x24);
    assert_eq!(nf_game::mp::equip::upgrade_mp_weapon(0x1a), 0x1a, "unrelated classes unchanged");
    // Sister: an unlisted id passes through.
    assert_eq!(nf_game::mp::equip::upgrade_mp_weapon(0x2c), 0x2c);
}

/// `MP_EquipPlayer` 0x1855f8 through master's `mp_equip_player`: the set's kind-7 weapon
/// (upgraded, `+0x92 · 2` rounds [M :15-23]), the 0x1a → 0x1b companion ammo [M :24-26], the
/// grapple gate at MPSettings+0x1D0 [M :27-34] and the Protection/Demolition team kit 0x3b ×1
/// [M :42-47]. (The watch-gadget arm `M_CONTROL+0x94 == 0x5000085` → 0x45 is not modelled on
/// master [S, equip.rs's doc] — not asserted here.)
#[test]
fn equip_loadout_numbers() {
    let w = weapon_table();
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.scenario = nf_game::mp::scenario::DEATHMATCH;
    // Set 0 (Normal): kind-7 weapon 0x6 → boot upgrade pick 0 → 0x6, `clip_size·2` = 60 rounds
    // [M :15-23] — the port's equip splits them: clip 30, pool 30.
    let row = PICKUP_MATRIX[0];
    let mut inv = nf_game::pickup::Inventory::empty(0x73, 0x21);
    let start = nf_game::mp::equip::mp_equip_player(&s, &row, 0, &mut inv, &w, |_| true);
    assert_eq!(start, 0x6);
    assert!(inv.owned[0x6], "set 0: 0x6 owned");
    assert_eq!(inv.clip[0x6], 30, "the clip holds the weapon's clip_size");
    assert_eq!(inv.carried[w.ammo_type(0x6)], 30, "60 − 30 rounds ride the pool");
    assert!(!inv.owned[0x50] && !inv.owned[0x3b], "no grapple/kit without the toggles");
    // Set 8 (State of the Art): kind-7 weapon 0x1a (clip 5 → 10 rounds) → the 0x1b companion
    // joins with 0x1b's clip_size rounds [M :24-26].
    let mut inv = nf_game::pickup::Inventory::empty(0x73, 0x21);
    let start = nf_game::mp::equip::mp_equip_player(&s, &PICKUP_MATRIX[8], 0, &mut inv, &w, |_| true);
    assert_eq!(start, 0x1a);
    assert!(inv.owned[0x1a]);
    assert_eq!(inv.clip[0x1a], 5, "clip_size(0x1a)");
    // The 0x1b companion ammo joins with clip_size(0x1b) = 7 rounds [M :24-26] — the pool ends
    // at 5 (the 0x1a equip's extra) + 7 = 12.
    assert_eq!(inv.carried[w.ammo_type(0x1b)], 12, "the 0x1b companion ammo");
    // Grapple on (+0x1D0 != 0): Upgrade_MPWeapon(slot, 0x50) = 0x50 with 0 rounds [M :27-34].
    s.grapple = 1;
    let mut inv = nf_game::pickup::Inventory::empty(0x73, 0x21);
    let _ = nf_game::mp::equip::mp_equip_player(&s, &row, 0, &mut inv, &w, |_| true);
    assert!(inv.owned[0x50], "the grapple weapon");
    s.grapple = 0;
    // The kit: Protection (0x20000080) team 0 or Demolition (0x20000040) team 1 → 0x3b ×1 [M
    // :42-47]. Protection attacker (team 1) gets nothing.
    s.scenario = nf_game::mp::scenario::PROTECTION;
    s.players[0].team = 0;
    let mut inv = nf_game::pickup::Inventory::empty(0x73, 0x21);
    let _ = nf_game::mp::equip::mp_equip_player(&s, &row, 0, &mut inv, &w, |_| true);
    assert!(inv.owned[0x3b], "the Protection defender's kit");
    s.players[0].team = 1;
    let mut inv = nf_game::pickup::Inventory::empty(0x73, 0x21);
    let _ = nf_game::mp::equip::mp_equip_player(&s, &row, 0, &mut inv, &w, |_| true);
    assert!(!inv.owned[0x3b], "the Protection attacker gets no kit");
    // Demolition attacker (team 1) does.
    s.scenario = nf_game::mp::scenario::DEMOLITION;
    let mut inv = nf_game::pickup::Inventory::empty(0x73, 0x21);
    let _ = nf_game::mp::equip::mp_equip_player(&s, &row, 0, &mut inv, &w, |_| true);
    assert!(inv.owned[0x3b], "the Demolition attacker's kit");
    // Sister: Deathmatch never kits.
    s.scenario = nf_game::mp::scenario::DEATHMATCH;
    let mut inv = nf_game::pickup::Inventory::empty(0x73, 0x21);
    let _ = nf_game::mp::equip::mp_equip_player(&s, &row, 0, &mut inv, &w, |_| true);
    assert!(!inv.owned[0x3b]);
}

/// A minimal `weapon_data` lift: records 1/6/0x1a/0x1b/0x50/0x3b with themselves as bases, the
/// selectable byte, a first-person hash (the equip gate) and the +0x92 clip sizes the test's
/// numbers come from (the same shape as pickup.rs's unit fixture). Ammo types: 0x6 → 1,
/// 0x1a/0x1b → 5, 0x50 → 2, 0x3b → 3.
fn weapon_table() -> nf_formats::weapon::WeaponData {
    let mut records = vec![[0u8; nf_formats::weapon::RECORD]; 0x73];
    for (w, clip, at) in [
        (1usize, 0u16, 0u8),
        (0x06, 30, 1),
        (0x1a, 5, 5),
        (0x1b, 7, 5),
        (0x50, 0, 2),
        (0x3b, 0, 3),
    ] {
        let r = &mut records[w];
        r[0..2].copy_from_slice(&(w as u16).to_le_bytes());
        r[2..4].copy_from_slice(&(w as u16).to_le_bytes());
        r[5] = 1;
        r[0x90] = at;
        r[0x92..0x94].copy_from_slice(&clip.to_le_bytes());
        r[0xdc..0xe0].copy_from_slice(&0x0500_0088u32.to_le_bytes());
    }
    let mut ammo_cap = vec![0i16; 34];
    ammo_cap[1] = 70;
    ammo_cap[3] = 10;
    ammo_cap[5] = 90;
    nf_formats::weapon::WeaponData { records, ammo_cap, ammo_raw: vec![[0u8; 12]; 34] }
}

/// The enviro toggles: `GunImp_Create` 0x131f20:50-62 (SP class 0xb, the +0x1B8 gate, Random set
/// → Rand(10), else the set id) and `Car_Create` 0x1305b0's obj+0x112 tree (the +0x1CC gate,
/// Ravine never, 1 tank / 2 heli / 3 = 51%-heli, ≥ 4 → heli [M :42-52], SP boot 0 → heli).
#[test]
fn gun_imp_and_car_toggles() {
    let mut rng = Rng::new(9);
    // SP: the emplacement exists with class 0xb whatever the toggles say.
    assert_eq!(gun_imp_class(0, 0, 0, &mut rng), Some(0xb));
    // MP, toggle off: nothing.
    assert_eq!(gun_imp_class(1, 0, 3, &mut rng), None);
    // MP, toggle on: the class is the set id; the Random set draws 0..9.
    assert_eq!(gun_imp_class(1, 1, 4, &mut rng), Some(4));
    for _ in 0..20 {
        let c = gun_imp_class(1, 1, 10, &mut rng).unwrap();
        assert!((0..10).contains(&c), "random class {c} out of 0..9");
    }
    // Vehicles: MP + 0 → none; Ravine (0x0700004b) → none always.
    assert_eq!(car_kind(1, 0, 0x0700_0023, &mut rng), None);
    assert_eq!(car_kind(1, 2, 0x0700_004b, &mut rng), None);
    assert_eq!(car_kind(1, 1, 0x0700_0023, &mut rng), Some(0)); // tanks
    assert_eq!(car_kind(1, 2, 0x0700_0023, &mut rng), Some(1)); // helicopters
    assert_eq!(car_kind(0, 0, 0x0700_0023, &mut rng), Some(1)); // SP boot 0: helicopter
    // The tree's other arms [M 0x1305b0:42-52]: ≥ 3 and != 3 → helicopter (the old branch's
    // `_ => 0` tank arm contradicted the decompile — fixed in this port).
    assert_eq!(car_kind(1, 4, 0x0700_0023, &mut rng), Some(1));
    // Random: 51/100 helicopter. With a fixed seed the draw is deterministic — pin it.
    let mut r = Rng::new(5);
    let want = if r.rand(100) < 0x33 { 1u8 } else { 0u8 };
    assert_eq!(car_kind(1, 3, 0x0700_0023, &mut Rng::new(5)), Some(want));
}

/// The boot defaults of the enviro toggles [M 0x1c7e78:38,42,45 — `MPSettings._440_4_`,
/// `_460_4_`, `_464_4_` all write 0]: +0x1B8 = 0, +0x1CC = 0, +0x1D0 = 0. (The old branch's
/// report pinned the grapple boot at 1 citing [M :45]; the decompile's line 45 writes 0 — the
/// port's `deathmatch()` default already matches the decompile, this pins it.)
#[test]
fn boot_defaults_for_the_toggles() {
    let s = MpSettings::deathmatch(0x0700_0023);
    assert_eq!(s.gun_emplacements, 0);
    assert_eq!(s.vehicles, 0);
    assert_eq!(s.grapple, 0);
}

/// `BOTWEAP_InitWeapon` 0x129e48:44-45's `startweap = weapon_data[PickupMatrix[set·10]].base` —
/// the byte stride means TRUE row `set`'s first entry. The botweap table now re-exports the
/// equip table (mp-pickups-respawn-2); a `BotWeaponTable` with per-id bases resolves set 1's
/// spawn weapon to base(PICKUP_MATRIX[1][0]) = base(0x2) — the old 10-wide table armed set 1
/// with row 2's 0x12 instead. Sister: set 3 resolves to base(0xe).
#[test]
fn botweap_start_weapon_follows_the_byte_stride() {
    let n = 0x73;
    let mut rows = vec![nf_game::mp::botweap::BotWeaponRow::default(); n];
    for (w, base) in [(0x2usize, 0x3u8), (0x12usize, 0x13u8), (0xeusize, 0xfu8), (0x2a, 0x2bu8)] {
        if w < n {
            rows[w].base = base;
        }
    }
    let t = nf_game::mp::botweap::BotWeaponTable { rows, ammo_cap: vec![0; 34] };
    assert_eq!(t.start_weapon(1), 0x3, "set 1's spawn weapon is base(0x2)");
    assert_eq!(t.start_weapon(3), 0xf, "set 3's spawn weapon is base(0xe)");
    // Sister: the old interleaved table would have given set 1 the row-2 weapon (0x12).
    assert_ne!(t.start_weapon(1), 0x13);
}
