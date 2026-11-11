//! effect-light-pick: the glist flags word (+0x44 → obj+0xF0 low 16) of the effect meshes, per
//! SP archive — the mask `Lights_CalcClosestLights` 0x1cb2e0 is called with for debris/gas/casing
//! (`View_DrawObjList` 0x1e69a0:78-81 passes obj+0xF0's low 16 bits). Kinds admitted per mask:
//! kind 0/2 need 0x80; kind 1 needs !0x100. The pins below are the masks the sim-side per-object
//! pick (`nf_game::effect::effect_light_picks`) gates with: most debris meshes admit kind 1, the
//! decals and the smoke meshes admit nothing. Opt-in via NF_ISO; numbers and names only.

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

#[test]
fn effect_mesh_glist_flags() {
    let Some(levels) = levels() else { eprintln!("NF_ISO not set - skipping"); return };
    let meshes: [(&str, u32); 12] = [
        ("decal-glass", 0x0200_0122), ("decal-stone", 0x0200_0126), ("decal-metal", 0x0200_0127),
        ("decal-wood", 0x0200_0128), ("decal-soft", 0x0200_0262), ("decal-ice", 0x0200_06fd),
        ("debris-05", 0x0200_012c), ("smoke-quad", 0x0200_0039), ("gas-1f", 0x0200_001f),
        ("gas-8bc", 0x0200_08bc), ("gas-26e", 0x0200_026e), ("breath", 0x0200_021d),
    ];
    for (name, hash) in meshes {
        let mut seen = std::collections::BTreeMap::<u32, Vec<&str>>::new();
        for (id, l) in &levels {
            for m in l.maps.iter().flat_map(|m| m.objects.iter()) {
                if m.hashcode == hash {
                    seen.entry(m.flags).or_default().push(id);
                }
            }
        }
        let summary: Vec<String> = seen.iter().map(|(f, ls)| format!("0x{f:08x} x{}", ls.len())).collect();
        println!("{name:>12} 0x{hash:08x}: {}", summary.join("  "));
    }
    // The masks the per-object pick rides: debris admits kind 1 (0x100 clear), the decals and
    // the smoke quad admit nothing (0x100 set, 0x80 clear) — the same census on every archive.
    let flag = |l: &Level, h: u32| l.maps.iter().flat_map(|m| m.objects.iter()).find(|o| o.hashcode == h).map(|o| o.flags as u16);
    let l05 = &levels.iter().find(|(id, _)| *id == "07000005").unwrap().1;
    assert_eq!(flag(l05, 0x0200_012c), Some(0x020), "debris 0x0200012c admits kind 1 (the unit test's piece mesh)");
    assert_eq!(flag(l05, 0x0200_012a), Some(0x020) );
    assert_eq!(flag(l05, 0x0200_012d), Some(0x221));
    for l in levels.iter().map(|(_, l)| l) {
        assert_eq!(flag(l, 0x0200_0122), Some(0x321), "decals admit no kind");
        assert_eq!(flag(l, 0x0200_0126), Some(0x321));
        assert_eq!(flag(l, 0x0200_0039), Some(0x329), "the smoke quad admits no kind");
    }
    // Casing meshes: every ammo_data casing_mesh hash present as a map object.
    let mut casings = std::collections::BTreeMap::<u32, u32>::new();
    for (_, l) in &levels {
        for m in l.maps.iter().flat_map(|m| m.objects.iter()) {
            if m.name.contains("Casing") || m.name.contains("casing") { *casings.entry(m.flags).or_default() += 1; }
        }
    }
    println!("casing-name glists: {:?}", casings);

    // Every EffectInfo debris + gas mesh and every ammo casing mesh, with the glist flags found
    // for it in the archives (the EffectInfo table is global in ACTION.ELF).
    let elf = std::fs::read(std::env::var("NF_ACTION").unwrap_or_default()).ok();
    let sfx = elf.as_deref().map(nf_formats::sfx::SurfaceSounds::from_action_elf);
    match sfx {
        Some(Ok(s)) => {
            let mut hashes = std::collections::BTreeSet::new();
            for e in &s.effects {
                if e.debris_mesh != 0 { hashes.insert(e.debris_mesh); }
                for g in &e.gas { if g.mesh != 0 { hashes.insert(g.mesh); } }
            }
            for m in s.ammo.casing_mesh.iter() { if *m != 0 { hashes.insert(*m); } }
            let mut by_hash: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
            for (_, l) in &levels {
                for m in l.maps.iter().flat_map(|m| m.objects.iter()) { by_hash.insert(m.hashcode, m.flags); }
            }
            for h in hashes {
                println!("EffectInfo/ammo mesh 0x{h:08x}: {}", match by_hash.get(&h) {
                    Some(f) => format!("glist flags 0x{f:08x} (low16 0x{f:04x})", f = f & 0xffff),
                    None => "NOT a map object".to_string(),
                });
            }
            for (i, e) in s.effects.iter().enumerate() {
                if e.debris_mesh != 0 { println!("EffectInfo[{i}] debris_mesh 0x{:08x}", e.debris_mesh); }
            }
            let smoke: [(&str, u32); 4] = [("cloud-0", 0x0200_026b), ("cloud-1", 0x0200_0273), ("cloud-2", 0x0200_0274), ("cloud-3", 0x0200_0275)];
            for (name, h) in smoke {
                println!("{name} 0x{h:08x}: {}", match by_hash.get(&h) {
                    Some(f) => format!("glist flags 0x{f:08x} centre {:?} radius {}", by_hash_centre(&levels, h), by_hash_radius(&levels, h)),
                    None => "NOT a map object".to_string(),
                });
            }
            // The glist local sphere centres/radii of the effect meshes (for the pick sphere).
            for (name, h) in [("debris-2c", 0x0200_012cu32), ("debris-2d", 0x0200_012d), ("gas-1f", 0x0200_001f), ("smoke-quad", 0x0200_0039)] {
                if let Some(o) = levels.iter().find_map(|(_, l)| l.maps.iter().flat_map(|m| m.objects.iter()).find(|o| o.hashcode == h)) {
                    println!("{name} centre {:?} radius {}", o.centre, o.radius);
                }
            }
        }
        Some(Err(e)) => println!("SurfaceSounds parse failed: {e}"),
        None => println!("NF_ACTION not set - EffectInfo census skipped"),
    }
}

fn by_hash_centre(levels: &[(&'static str, Level)], h: u32) -> [f32; 3] {
    levels.iter().find_map(|(_, l)| l.maps.iter().flat_map(|m| m.objects.iter()).find(|o| o.hashcode == h)).map(|o| o.centre).unwrap_or([0.0; 3])
}
fn by_hash_radius(levels: &[(&'static str, Level)], h: u32) -> f32 {
    levels.iter().find_map(|(_, l)| l.maps.iter().flat_map(|m| m.objects.iter()).find(|o| o.hashcode == h)).map(|o| o.radius).unwrap_or(0.0)
}
