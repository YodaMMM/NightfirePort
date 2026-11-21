//! g026_reload_census: per level, the drone placements (kind 0x0f) whose anim set can play the
//! alternate stand reload (anim state 0x4a, `NDrone2_DSTATE_AimStandReload` 0x1666d8:21-27 gates it
//! on `DroneAnim_CanDoAnimState(0x4a, 0)`), and the type-0xC drones (`+0xD8 == 0xc`) that reach the
//! CombatNoMove Rand(3) → CrouchCover roll [M 0x1643e8:103-111]. [D level data + ACTION.ELF tables]
//! Assertion-free probe: run with --ignored --nocapture.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;

#[test]
#[ignore = "assertion-free census dump; run with --ignored --nocapture"]
fn g026_reload_census() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let elf = img.read_file(img.find("/ACTION.ELF").unwrap()).unwrap();
    let tables = nf_formats::drone::DroneAnimData::from_action_elf(&elf).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let own = |st: usize| tables.states.get(st).map(|s| s.0).unwrap_or(0);
    println!("anim state 0x4a own danim {:#x}, 0x52 own danim {:#x}", own(0x4a), own(0x52));
    for &lv in nf_view::dev::LEVELS {
        let name = format!("{lv:08x}.bin");
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)) else { continue };
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let Ok(level) = Level::parse(&data) else { continue };
        let lb = LevelBin::parse(&data).unwrap();
        let have: std::collections::HashSet<u32> = lb.entries.iter().filter(|e| e.kind == 5).filter_map(|e| nf_formats::anim::Script::parse(lb.data(&data, e)).ok()).map(|s| s.hashcode).collect();
        let Some(m) = level.main_map() else { continue };
        let (mut n, mut alt, mut alt_loaded, mut t12) = (0, 0, 0, 0);
        let mut sets = std::collections::BTreeMap::new();
        for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x0f) {
            let param = |k: u32| p.params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v).unwrap_or(0);
            let s = nf_formats::drone::drone_setup(param(0), param(9), lv);
            n += 1;
            if s.kind == 0xc { t12 += 1; }
            if let Some(h) = tables.script_for(own(0x4a), s.anim_set) {
                alt += 1;
                if have.contains(&h) { alt_loaded += 1; }
                *sets.entry(s.anim_set).or_insert(0) += 1;
            }
        }
        println!("{lv:08x}: drones {n}, alt-reload set {alt} (script in level {alt_loaded}) sets {sets:x?}, type 0xc {t12}");
    }
}
