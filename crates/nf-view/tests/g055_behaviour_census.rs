//! G-055 census over every level archive: placement parameter 5 (`Drone+0xC48`, 100 = the second
//! behaviour block `+0x4E8` exists [M NDrone2_DoModeSettingsNEW 0x14c600:61-70, DroneFunc_Set2ndBehaviour
//! 0x14a618:5-11]) and the skin-0x0500001E placements (voice set `+0xD8 = 0xB` [M NDrone2_DefaultInit
//! 0x14b300:158-162]) whose first-block alertness starts at 1.0 or more — the only placements that take
//! `GetDroneTypeAttackTypeFriend`'s creation swap [M 0x14cbd8:119-124].
use nf_formats::level::Level;

#[test]
fn g055_behaviour_census() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
    let elf = img.read_file(img.find("/ACTION.ELF").expect("ACTION.ELF")).expect("read ACTION.ELF");
    let props = nf_formats::drone::PropertyTables::from_action_elf(&elf).expect("property tables");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let (mut total, mut p5_100, mut other) = (0u32, 0u32, Vec::new());
    let mut kind_b = Vec::new();
    for &lv in nf_view::dev::LEVELS {
        let name = format!("{lv:08x}.bin");
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)) else { continue };
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let level = Level::parse(&data).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        for (mi, map) in level.maps.iter().enumerate() {
            for (i, p) in map.placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0x0f) {
                let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
                total += 1;
                if param(5) == 100 { p5_100 += 1 } else { other.push(format!("{lv:08x} m{mi} #{i:x} p5={} skin={:08x} p1={:08x} n={} maps={}", param(5), param(0), param(1), map.placements.len(), level.maps.len())) }
                if param(0) == 0x0500_001e {
                    let (normal, _) = nf_formats::drone::behaviour_words(&param);
                    let a = nf_formats::drone::ALERTNESS_LEVELS[props.get(0x20, &normal) as usize & 3];
                    kind_b.push(format!("{lv:08x} #{i:x} alert={a}"));
                }
            }
        }
    }
    println!("drone placements {total}, p5 == 100: {p5_100}, other: {other:?}");
    println!("skin 0x0500001e placements: {kind_b:?}");
    // [D] 555 drone placements carry the second block (p5 = 100); the only other kind-0x0f records
    // are the 33 skinless p5 = 0 template records (one per archive carrying the template map,
    // docs/systems/ai-dmode-inits.md 2026-10-24 addendum) — the port's `alert_props` is None for them.
    assert_eq!(p5_100, 555);
    assert_eq!(other.len(), 33, "{other:?}");
    assert!(other.iter().all(|o| o.contains(" p5=0 skin=05000000 ")), "{other:?}");
    assert_eq!(kind_b.len(), 3, "{kind_b:?}");
    assert!(kind_b.iter().all(|k| k.ends_with("alert=1")), "{kind_b:?}");
}
