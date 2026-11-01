//! The x-ray-only glists and the goggles' assets across the 21 SP archives [D]. A glist whose
//! flags (+0x44) carry 0x400 is submitted to a view only while that viewer's goggle mode
//! (viewer+0x236) is 2 [M View_AddObjects 0x1e62d8:38-47, View_AddForcedObjects 0x1e5fb8:79-84,
//! View_DrawCelGlist 0x1e6538 / View_DrawObjectGlist 0x1e6780's sub-entry test]. Docs:
//! docs/systems/goggles.md. Opt-in via NF_ISO. Numbers and names only.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

const SP_LEVELS: [&str; 21] = [
    "07000001", "07000002", "07000003", "07000004", "07000005", "07000006", "07000007",
    "07000008", "07000009", "0700000a", "0700000b", "0700000c", "07000011", "07000012",
    "07000013", "07000014", "07000015", "07000016", "07000041", "0700004a", "0700004b",
];

fn levels() -> Option<Vec<(&'static str, Level)>> {
    let iso = std::env::var("NF_ISO").ok()?;
    let img = IsoImage::open(&iso).ok()?;
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).ok()?;
    Some(SP_LEVELS.iter().filter_map(|&id| {
        let fe = entries.iter().find(|e| e.name == format!("{id}.bin"))?;
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()?;
        Some((id, Level::parse(&data).ok()?))
    }).collect())
}

/// [D] every 0x400 glist that a main-map placement uses, per level: (name, placement kind
/// byte, the placement's skip bits, count). The created ones (skip bits clear) are the port's
/// x-ray slots; the "IR FootPrint" pair rides 0x2000-skipped placements in every archive (no
/// creator: neither its hash 0x2000567/0x2000568 nor its name appears in the ACTION.ELF
/// decompile [M grep]).
#[test]
fn x_ray_only_glists_per_level() {
    let Some(levels) = levels() else { eprintln!("NF_ISO not set - skipping"); return };
    assert_eq!(levels.len(), 21);
    let mut created: Vec<(&str, String, u8, usize)> = Vec::new();
    for (id, level) in &levels {
        let Some(m) = level.main_map() else { continue };
        // The footprint pair: placed (by hash, skip bit 0x2000, kind byte 0x31) in a side map.
        let footprints = level.maps.iter().flat_map(|mm| mm.placements.iter())
            .filter(|p| p.by_hash() && p.flags & 0x2000 != 0)
            .filter(|p| level.by_hash.get(&p.hashcode).is_some_and(|&(om, oi)| level.maps[om].objects[oi].name.starts_with("IR FootPrint") && level.maps[om].objects[oi].flags & 0x400 != 0))
            .count();
        assert_eq!(footprints, 2, "{id}: the IR FootPrint pair, skip-flagged");
        let mut rows: std::collections::BTreeMap<(String, u8), usize> = Default::default();
        for p in &level.maps[m].placements {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { continue };
            let o = &level.maps[om].objects[oi];
            if o.flags & 0x400 == 0 || p.flags & 0xa000 != 0 { continue; }
            *rows.entry((o.name.clone(), (p.flags & 0xff) as u8)).or_default() += 1;
        }
        for ((name, kind), n) in rows { created.push((id, name, kind, n)); }
    }
    let want: Vec<(&str, String, u8, usize)> = vec![
        ("07000003", "HOTOBJECT".into(), 0x20, 1),
        ("0700000a", "laserbeam".into(), 0xeb, 10),
        ("0700000b", "laserbeam-large".into(), 0xeb, 8),
        ("07000014", "laserbeam".into(), 0xea, 8),
    ];
    assert_eq!(created, want, "the created x-ray-only placements");
    // Sister (can fail): none of them carries the 0x10 no-draw bit (they would never draw).
    for (id, level) in &levels {
        for map in &level.maps { for o in &map.objects {
            if o.flags & 0x400 != 0 && (o.name == "HOTOBJECT" || o.name.starts_with("laserbeam")) { assert_eq!(o.flags & 0x10, 0, "{id} {}", o.name); }
        } }
    }
}

/// [D] the meshes and textures the goggles draw resolve in every SP archive: the glow's
/// "lazer_muzzle" 0x200025f + "BoneGlow" 0x200055a, the beam sprites "lazer" 0x2000131 /
/// "watch_lazer" 0x2000133, the noise 0x0300004A (64×64) / 0x03000098 (32×32), the fill
/// 0x03000002, the bar 0x0300017F (256×32) and the icon 0x0300017E (16×16).
#[test]
fn goggle_assets_resolve_everywhere() {
    let Some(levels) = levels() else { eprintln!("NF_ISO not set - skipping"); return };
    for (id, level) in &levels {
        let name = |h: u32| level.by_hash.get(&h).map(|&(m, o)| level.maps[m].objects[o].name.clone());
        assert_eq!(name(0x0200_025f).as_deref(), Some("lazer_muzzle"), "{id}");
        assert_eq!(name(0x0200_055a).as_deref(), Some("BoneGlow"), "{id}");
        assert_eq!(name(0x0200_0131).as_deref(), Some("lazer"), "{id}");
        assert_eq!(name(0x0200_0133).as_deref(), Some("watch_lazer"), "{id}");
        let tex = |h: u32| level.maps.iter().find_map(|m| m.textures.iter().find(|t| t.hashcode == h).map(|t| (t.width, t.height)));
        assert_eq!(tex(0x0300_004a), Some((64, 64)), "{id}");
        assert_eq!(tex(0x0300_0098), Some((32, 32)), "{id}");
        assert_eq!(tex(0x0300_0002), Some((32, 32)), "{id}");
        assert_eq!(tex(0x0300_017f), Some((256, 32)), "{id}");
        assert_eq!(tex(0x0300_017e), Some((16, 16)), "{id}");
        // Sister (can fail): the two glow meshes are distinct objects.
        assert_ne!(level.by_hash.get(&0x0200_025f), level.by_hash.get(&0x0200_055a));
    }
}
