//! mp-pickups-visible (docs/systems/mp-pickups-visible.md): the MP arenas' weapon pickups get
//! their weapon from the match's weapon set (`Pickup_CreateFromSet` 0x18f1b0 [M]) and draw the
//! WEAPON's own world model (`weapon_data[base]+0x80` [M]) — not the placement's `PICKUP`
//! marker (hashcode 0xFFFFFFFF, [D 07000023/24]), which is why nothing was visible.
//! Numbers/ids only, driven through the real load path (`nf_view::load_mp_carry`).
//! Sister: the SP level 07000001's pickups are unchanged.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement::Placement;
use nf_formats::weapon::WeaponData;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(img: &IsoImage, archive: &str) -> Vec<u8> {
    let (entries, bin) = nf_formats::disc::files_bin_entries(img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn weapon_data(img: &IsoImage) -> WeaponData {
    let elf = img.read_file(img.find("/ACTION.ELF").expect("ACTION.ELF")).expect("read");
    WeaponData::from_action_elf(&elf).expect("weapon_data")
}

/// The placement's param k (`level_tag+0x2C+4k`, absent reads 0) — the game's own lookup [M 0x1d04f0:457-462].
fn param(p: &Placement, idx: u32) -> u32 {
    p.params.iter().find(|(i, _)| *i == idx).map(|(_, v)| *v).unwrap_or(0)
}

/// The main map's weapon-set placements: (kind 7..11, translation).
fn set_placements(level: &Level) -> Vec<(u16, [f32; 3])> {
    let mi = level.main_map().unwrap();
    level.maps[mi].placements.iter()
        .filter(|p| p.flags & 0xff == 0xf0)
        .map(|p| (param(p, 0) as u16, p.translation))
        .filter(|(kind, _)| (7..=11).contains(kind))
        .collect()
}

fn same_spot(a: &[f32; 3], b: &[f32; 3]) -> bool {
    (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3 && (a[2] - b[2]).abs() < 1e-3
}

#[test]
fn mp_arena_set_pickups_follow_the_weapon_set_and_resolve() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = IsoImage::open(&iso).unwrap();
    let wd = weapon_data(&img);
    for (code, archive) in [(0x0700_0023u32, "07000023.bin"), (0x0700_0024u32, "07000024.bin")] {
        let level = Level::parse(&read_archive(&img, archive)).expect("the arena parses");
        let sets = set_placements(&level);
        assert!(sets.len() >= 10, "{archive}: the arena places its weapon-set pickups [D the dump: 16/23]");
        for set in [0u32, 1] {
            let mut ms = nf_game::mp::MpSettings::deathmatch(code);
            ms.weapon_set = set;
            let row = nf_game::mp::equip::weapon_row(&ms);
            let scene = nf_view::load_mp_carry(&iso, archive, false, None, ms).expect("the arena loads");
            let sim = scene.sim.as_ref().expect("the sim builds");
            assert!(sim.drones.pickups.len() >= sets.len(), "every set pickup is created");
            for (kind, t) in &sets {
                let p = sim.drones.pickups.iter().find(|p| same_spot(&p.start, t))
                    .unwrap_or_else(|| panic!("{archive} set {set}: the kind-{kind} placement's pickup exists at {t:?}"));
                let w = row[(*kind - 7) as usize];
                let base = wd.base(w as usize) as u16;
                assert_eq!(p.kind, nf_game::pickup::kind::WEAPON, "a set pickup is a kind-0 weapon pickup [M 0x18f1b0]");
                assert_eq!(p.weapon, base, "{archive} set {set} kind {kind}: weapon {w:#04x}'s base id [M PickupMatrix 0x2b9100 + weapon_data+0x02]");
                let model = u32::from_le_bytes(wd.records[base as usize][0x80..0x84].try_into().unwrap());
                assert_eq!(p.model, model, "{archive} set {set} kind {kind}: the weapon's own world model [M 0x18f1b0:56-61]");
                assert!(p.obj.is_none(), "{archive} set {set} kind {kind}: the PICKUP/0xFFFFFFFF marker is never drawn [D]");
                assert!(p.spin, "{archive} set {set} kind {kind}: MP pickups spin [M 0x18dda0:138-144]");
                // The model resolves to a mesh with triangles [D: the arenas ship their weapons' +0x80 meshes].
                let (om, oi) = *level.by_hash.get(&model)
                    .unwrap_or_else(|| panic!("{archive} set {set} kind {kind}: weapon model {model:#010x} resolves"));
                let tris: usize = level.maps[om].objects[oi].gfx.leaves.iter().flat_map(|l| &l.batches).map(|b| b.triangles().len()).sum();
                assert!(tris > 0, "{archive} set {set} kind {kind}: weapon model {model:#010x} has triangles (got {tris})");
            }
        }
    }
}

/// Sister: the SP level 07000001 loads without a match and its pickups are unchanged — no
/// weapon-set kinds, every pickup still drawn from its placement's own object, SP spin rule.
/// The gold-medal `bonus[0]` word for `base_map` [M PlrStats_HasGoldMedal 0x199a40;
/// Menu_SetLevelBonus 0x1ffa10 is the award arm]. pickup-medal-gate.
fn gold_bits(base_map: u32) -> u64 {
    let mut p = nf_game::progress::Progress::default();
    p.set_level_bonus(base_map, 3, 0);
    p.bonus[0]
}
#[test]
fn sp_level_pickups_are_unchanged() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = IsoImage::open(&iso).unwrap();
    let level = Level::parse(&read_archive(&img, "07000001.bin")).expect("07000001 parses");
    let mi = level.main_map().unwrap();
    let f0: Vec<&Placement> = level.maps[mi].placements.iter().filter(|p| p.flags & 0xff == 0xf0).collect();
    assert_eq!(f0.len(), 6, "07000001 places six pickups [D: kinds 1 x2, 3 x2, 6 x2]");
    // pickup-medal-gate: the two kind-6 rows create only with 07000001's gold medal recorded
    // [M 0x18dda0:93-95; the mission table pairs 07000001 → BaseMap 0x07000001 [L ee_start_05
    // 0x2a4350], whose gold bit is the 0x0700001/1b kind-4 group's rank 0 [L 0x2bc920 row 4]].
    let scene = nf_view::load_with_carry_medals(&iso, "07000001.bin", false, None, 1, gold_bits(0x0700_0001)).expect("07000001 loads");
    let sim = scene.sim.as_ref().expect("the sim builds");
    let pickups = &sim.drones.pickups;
    // Every 0xF0 placement created its pickup, at the placement's own spot (the index pairing holds).
    for p in &f0 {
        let t = p.translation;
        assert!(pickups.iter().any(|k| same_spot(&k.start, &t)), "07000001: the placement at {t:?} created its pickup");
    }
    for p in pickups {
        assert!(p.obj.is_some(), "07000001: a placed pickup draws its placement's own object (the fix-objects path is unchanged)");
        assert!(!p.spin || p.kind == nf_game::pickup::kind::BOND_BONUS, "07000001: only kind 6 spins in SP [M]");
    }
    // And the kinds really are the archive's [D].
    let mut kinds: Vec<u16> = f0.iter().map(|p| param(p, 0) as u16).collect();
    kinds.sort_unstable();
    assert_eq!(kinds, vec![1, 1, 3, 3, 6, 6], "07000001's kind census [D]");
}
