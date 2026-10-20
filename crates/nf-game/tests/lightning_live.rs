//! The 0x2c lightning strikes on the disc's two storm levels — 07000001 (11 records) and
//! 07000041 (4) [D-census]: the pairing through `WorldFx::attach_level`
//! (`Lightning_Create` 0x18cde8's class-0x33 object walk), the strike geometry (ext+0x20 =
//! the master's translation, ext+0x30 = the partner's [M 0x18cde8:55-68]) and the
//! `Lightning_Update` 0x18cf68 visibility state. Opt-in: set `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};
use nf_game::world_fx::WorldFx;

fn level(iso: &IsoImage, name: &str) -> Level {
    let (entries, bin) = disc::files_bin_entries(iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Level::parse(&data).unwrap()
}

fn load(name: &str, hash: u32) -> Option<(Level, WorldFx)> {
    let path = std::env::var_os("NF_ISO")?;
    let iso = IsoImage::open(&path).unwrap();
    let lvl = level(&iso, name);
    let fx = WorldFx::attach_level(&lvl, hash);
    Some((lvl, fx))
}

fn close(a: [f32; 3], b: [f32; 3], what: &str) {
    assert!((a[0] - b[0]).abs() < 0.01 && (a[1] - b[1]).abs() < 0.01 && (a[2] - b[2]).abs() < 0.01,
        "{what}: {a:?} vs {b:?}");
}

/// 07000001's eleven 0x2c records (main-map #237-247, all group word 0 [D placement bytes]):
/// six key-1 (#237-242) and five key-2 (#243-247). Each later record pairs with the nearest
/// earlier still-class-0x33 record of its key (the newest-first object walk) → a chained
/// ladder of strikes (238,237)…(242,241) and (244,243)…(247,246), the first of each key inert.
#[test]
fn lightning_07000001_chained_ladders() {
    let Some((lvl, fx)) = load("07000001.bin", 0x0700_0001) else { eprintln!("skip: NF_ISO not set"); return; };
    assert_eq!(fx.lightnings.len(), 9, "6 key-1 records → 5 strikes, 5 key-2 → 4");
    assert_eq!(fx.unpaired_lightnings, vec![237, 243], "the first of each key keeps ext[0] = 0 [M 0x18cf68:11]");
    let s0 = &fx.lightnings[0];
    assert_eq!((s0.master_placement, s0.partner_placement, s0.key), (238, 237, 1));
    close(s0.start, [-14.82, 9.02, -2.42], "strike 0 start = master #238's translation [D]");
    close(s0.end, [-21.38, 28.71, -2.97], "strike 0 end = partner #237's translation [D]");
    let pairs: Vec<(usize, usize)> = fx.lightnings.iter().map(|s| (s.master_placement, s.partner_placement)).collect();
    assert_eq!(pairs, vec![(238, 237), (239, 238), (240, 239), (241, 240), (242, 241), (244, 243), (245, 244), (246, 245), (247, 246)]);
    assert!(fx.lightnings.iter().all(|s| s.group == 0), "every record's param 0 word is 0 [D]");
    // The keys re-derived from the placements themselves (the sister assertion).
    let key = |i: usize| lvl.maps[lvl.main_map().unwrap()].placements[i].params.iter().find(|(j, _)| *j == 1).map(|&(_, v)| v as u16).unwrap();
    assert_eq!(fx.lightnings.iter().map(|s| s.key).collect::<Vec<_>>(), vec![key(238), key(239), key(240), key(241), key(242), key(244), key(245), key(246), key(247)]);
}

/// 07000041's four 0x2c records (#82-85): keys 1,1,2,2 → two strikes; the `Lightning_Update`
/// visibility flips only with the storm's `SuperHappyLightningFlag` (the port's
/// `WorldFx::lightning` copy — the sim does not drive `View_DrawSky` yet, so a bare tick
/// keeps every bolt hidden).
#[test]
fn lightning_07000041_two_strikes_and_visibility() {
    let Some((_lvl, mut fx)) = load("07000041.bin", 0x0700_0041) else { eprintln!("skip: NF_ISO not set"); return; };
    assert_eq!(fx.lightnings.len(), 2);
    assert_eq!(fx.unpaired_lightnings, vec![82, 84]);
    let s0 = &fx.lightnings[0];
    assert_eq!((s0.master_placement, s0.partner_placement, s0.key), (83, 82, 1));
    close(s0.start, [7.97, 0.00, -45.77], "start = master #83 [D]");
    close(s0.end, [7.97, 14.17, -45.46], "end = partner #82 [D]");
    let s1 = &fx.lightnings[1];
    assert_eq!((s1.master_placement, s1.partner_placement, s1.key), (85, 84, 2));
    close(s1.start, [14.76, 38.57, -47.54], "start = master #85 [D]");
    assert!(fx.lightnings.iter().all(|s| !s.visible), "loaded hidden");
    // `Lightning_Update` 0x18cf68 through the real tick path: the flag arm shows, the clear arm hides.
    fx.lightning.super_happy = true;
    let mut ch = nf_game::triggers::Channels::default();
    let mut gas = nf_game::gas::GasPool::default();
    let mut rng = nf_game::drone::Rng::new(1);
    fx.tick(&mut ch, [0.0; 3], 0, &mut gas, &mut rng);
    assert!(fx.lightnings.iter().all(|s| s.visible), "SuperHappyLightningFlag → the draw arm [M 0x18cf68:27-42]");
    fx.lightning.super_happy = false;
    fx.tick(&mut ch, [0.0; 3], 1, &mut gas, &mut rng);
    assert!(fx.lightnings.iter().all(|s| !s.visible), "flag clear → the hide arm [M 0x18cf68:13-21]");
}
