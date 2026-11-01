//! goggles-draws censuses [D], numbers and names only. Opt-in via NF_ISO. Docs:
//! docs/systems/goggles.md.
//!
//! 1. The night-sight quarter bit: `View_SetupRenderModes_Tweak` 0x1e5888:13-20 quarters the alpha
//!    of an object with obj+0xF0 & 0x20000 in goggle mode 1. obj+0xF0 comes from the glist flags
//!    (`celglist+0x44`, `MapObject::flags`); no ACTION.ELF store ORs the bit in [M grep]. No glist
//!    in any FILES.BIN archive carries it.
//! 2. The SP Env_Create placements (0xDF) and their drop type (param 0; 0 = snow) — the breath
//!    gate `GlobalEnv == 0 || GlobalEnv+0x4b != 0` [M Env_Breath 0x17bed0-0x17bee4].
//! 3. The keyed trigger placements (count = extra[7] & 0xffff ≥ 2 with a key list) — the moving
//!    tripwires [M Trigger_Create 0x1b1720:56-87].

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn archives(filter: impl Fn(&str) -> bool) -> Option<Vec<(String, Level)>> {
    let iso = std::env::var("NF_ISO").ok()?;
    let img = IsoImage::open(&iso).ok()?;
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).ok()?;
    Some(entries.iter().filter(|e| e.name.ends_with(".bin") && filter(&e.name)).filter_map(|fe| {
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()?;
        Some((fe.name.clone(), Level::parse(&data).ok()?))
    }).collect())
}

#[test]
fn no_glist_carries_the_night_quarter_bit() {
    let Some(all) = archives(|_| true) else { eprintln!("NF_ISO not set - skipping"); return };
    assert_eq!(all.len(), 99, "every FILES.BIN archive parses");
    let mut objects = 0usize;
    let mut with_400 = 0usize;
    for (name, level) in &all {
        for map in &level.maps { for o in &map.objects {
            objects += 1;
            assert_eq!(o.flags & 0x20000, 0, "{name}: {} carries 0x20000", o.name);
            if o.flags & 0x400 != 0 { with_400 += 1; }
        } }
    }
    assert!(objects > 10000, "the scan saw the objects ({objects})");
    // Sister (can fail): the scan sees glist bits at all — the x-ray-only 0x400 is on the disc.
    assert_eq!(with_400, 72, "0x400 glists seen");
}

#[test]
fn sp_env_drop_types_and_keyed_tripwires() {
    let Some(sp) = archives(|n| n.starts_with("070000") && !n.starts_with("0700002") && !n.starts_with("0700004") || n == "07000041.bin" || n == "0700004a.bin" || n == "0700004b.bin") else { eprintln!("NF_ISO not set - skipping"); return };
    let mut envs: Vec<(String, Vec<u32>)> = Vec::new();
    let mut keyed: Vec<(String, String, u32, usize)> = Vec::new();
    for (name, level) in &sp {
        let Some(m) = level.main_map() else { continue };
        let paths = level.path_blocks();
        let mut e = Vec::new();
        let mut rows: std::collections::BTreeMap<(String, u32), usize> = Default::default();
        for p in &level.maps[m].placements {
            let k = p.flags & 0xff;
            if k == 0xdf { e.push(p.params.iter().find(|x| x.0 == 0).map_or(0, |x| x.1)); }
            if !(k == 0xdc || k == 0xe8 || (0xea..=0xef).contains(&k) || k == 0xf4) { continue; }
            if (p.extra[7] & 0xffff) < 2 { continue; }
            if paths.get((p.extra[8] & 0xffff) as usize).map_or(0, |b| b.len() / 28) < 2 { continue; }
            let t = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { Some((m, p.index as usize)) };
            let nm = t.and_then(|(a, b)| level.maps[a].objects.get(b)).map(|o| o.name.clone()).unwrap_or_default();
            *rows.entry((nm, k)).or_default() += 1;
        }
        if !e.is_empty() { envs.push((name.trim_end_matches(".bin").to_string(), e)); }
        for ((nm, k), n) in rows { keyed.push((name.trim_end_matches(".bin").to_string(), nm, k, n)); }
    }
    envs.sort(); keyed.sort();
    let s = |v: &str| v.to_string();
    assert_eq!(envs, vec![
        (s("07000001"), vec![1]), (s("07000002"), vec![1]), (s("07000003"), vec![1]), (s("07000004"), vec![1]),
        (s("07000005"), vec![0, 0]), (s("07000006"), vec![0]), (s("07000007"), vec![0]), (s("07000008"), vec![0]),
        (s("07000009"), vec![1]), (s("0700000a"), vec![1]), (s("0700000b"), vec![1]), (s("0700000c"), vec![255]),
        (s("0700000d"), vec![255]), (s("07000014"), vec![255, 2]), (s("07000041"), vec![1]),
    ], "the 0xDF drop types (0 = snow: 05-08 breathe in every mode)");
    assert_eq!(keyed, vec![
        (s("0700000a"), s("laserbeam"), 0xeb, 3), (s("0700000b"), s("laserbeam-large"), 0xeb, 8), (s("07000014"), s("laserbeam"), 0xea, 8),
    ], "the keyed trigger placements are the tripwire beams");
    // Sister (can fail): the scan covered the SP set, including levels with neither.
    assert!(sp.len() >= 21, "{} SP archives", sp.len());
}
