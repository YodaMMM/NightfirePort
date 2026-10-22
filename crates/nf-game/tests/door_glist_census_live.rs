//! Census: every door placement on every on-foot SP level (0700* archives), resolved through the
//! REAL `Doors::create` closure, with a classification for any glist-0 door. ISO-gated (NF_ISO).
//!
//! Game rule [M parsemap_block_map_data_dynamic 0x1d1b68:77-95]: a dynamic placement resolves its
//! glist by-index (`*puVar8 < MapEntityCount` → `pCelList + ref*0x50`, else NULL) or by-hash
//! (`hashtable_getitem`), and `parsemap_create_dynamic_objects` — which is what runs
//! `Door_Create` 0x136100 with that glist as its `param_4` — is CALLED ONLY WHEN THE GLIST
//! RESOLVED (0x1d1b68:93-95, `if (iVar5 != 0)` around the call, inside the `flags & 0xa000 == 0`
//! gate `door_placements()` already mirrors). `Door_Create` stores it at `obj+0xd8`
//! (`Control_SetGList` 0x134960) and ORs `obj+0xfa |= 0xc4` — bit 3 stays clear — so the use
//! pick's admission test for the 0x101 mask [M Collide_PickObj 0x1eae80:88-90,
//! `(mask == 0x100 || (obj+0xfa & 8)) || (obj+0xd8 != 0 && *(obj+0xd8+8) != 0)`] reduces to
//! `obj+0xd8 != 0 && *(glist+8) != 0`: a door the game creates always carries a glist, and a
//! placement whose glist does not resolve creates NO door at all.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};
use nf_game::door::{door_placements, Doors};

fn iso_path() -> Option<String> { std::env::var("NF_ISO").ok() }

fn sp_archives(iso: &IsoImage) -> Vec<(String, u64, u64)> {
    let (entries, bin) = disc::files_bin_entries(iso).expect("FILES.BIN");
    entries
        .iter()
        .filter(|e| {
            let n = e.name.to_ascii_lowercase();
            n.starts_with("0700") && n.ends_with(".bin")
        })
        .map(|e| (e.name.clone(), bin.lba as u64 * 2048 + e.offset as u64, e.size as u64))
        .collect()
}

fn parse_at(iso: &IsoImage, off: u64, size: u64) -> Level {
    let data = iso.read_bytes(off, size).expect("read archive");
    Level::parse(&data).expect("parse archive")
}

/// The EXACT closure `drone.rs` hands `Doors::create` (the resolution under test), plus a reason
/// tag for a `None` result.
fn resolve(level: &Level, m: usize, p: &nf_formats::placement::Placement) -> (Option<(u32, [f32; 3], f32)>, &'static str) {
    if p.by_hash() {
        match level.by_hash.get(&p.hashcode) {
            None => (None, "by-hash miss (game: hashtable_getitem NULL -> no create)"),
            Some(&(om, oi)) => {
                let o = &level.maps[om].objects[oi];
                if o.hashcode == 0 {
                    (None, "by-hash hit, object hashcode 0")
                } else {
                    (Some((o.hashcode, o.centre, o.radius)), "")
                }
            }
        }
    } else {
        match level.maps[m].objects.get(p.index as usize) {
            None => (None, "by-index OOB (game: ref >= MapEntityCount -> no create)"),
            Some(o) => {
                // The create site's anonymous-object branch: hashcode u32::MAX -> the synthetic
                // CollisionWorld key (0x80000000 | map<<16 | index) — never 0.
                if o.hashcode == u32::MAX {
                    (Some((0x8000_0000 | ((m as u32) << 16) | p.index as u32, o.centre, o.radius)), "")
                } else if o.hashcode == 0 {
                    (None, "by-index hit, object hashcode 0 (game: glist = the table entry itself, non-NULL)")
                } else {
                    (Some((o.hashcode, o.centre, o.radius)), "")
                }
            }
        }
    }
}

/// Probe dump — run with `--ignored -- --nocapture` to see every level's doors and any glist-0
/// rows with their reason. Assertion-free.
#[test]
#[ignore = "assertion-free census dump; run with --ignored -- --nocapture"]
fn census_dump() {
    let Some(path) = iso_path() else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).unwrap();
    let mut total = 0usize;
    let mut zero = 0usize;
    for (name, off, size) in sp_archives(&iso) {
        let level = parse_at(&iso, off, size);
        let Some(m) = level.main_map() else { continue };
        let ps = &level.maps[m].placements;
        let pis = door_placements(ps);
        if pis.is_empty() { continue; }
        let doors = Doors::create(ps, &level.path_blocks(), |p| resolve(&level, m, p).0);
        let zeros: Vec<_> = doors.list.iter().filter(|d| d.glist == 0).collect();
        total += doors.list.len();
        zero += zeros.len();
        eprintln!("{}: {} doors, {} glist-0", name, doors.list.len(), zeros.len());
        for d in &zeros {
            let p = &ps[d.placement];
            let (_, why) = resolve(&level, m, p);
            eprintln!("  glist-0: plc{} hash={:#010x} idx={} kind={} at=({:.2},{:.2},{:.2}) — {}",
                d.placement, p.hashcode, p.index, d.kind, p.translation[0], p.translation[1], p.translation[2], why);
        }
    }
    eprintln!("TOTAL: {} doors, {} glist-0", total, zero);
}

/// The pin: on every on-foot SP level, every door `Doors::create` produces carries a resolved
/// glist — the game never creates a glist-less door [M 0x1d1b68:93-95], so the port's create
/// skips unresolved placements and this must never regress (it is what retired the
/// placement-distance stand-in in `player_use`).
#[test]
fn every_sp_door_resolves_a_glist() {
    let Some(path) = iso_path() else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).unwrap();
    let mut levels = 0usize;
    let mut doors = 0usize;
    let mut bad = Vec::new();
    for (name, off, size) in sp_archives(&iso) {
        let level = parse_at(&iso, off, size);
        let Some(m) = level.main_map() else { continue };
        let ps = &level.maps[m].placements;
        if door_placements(ps).is_empty() { continue; }
        levels += 1;
        let ds = Doors::create(ps, &level.path_blocks(), |p| resolve(&level, m, p).0);
        doors += ds.list.len();
        for d in &ds.list {
            if d.glist == 0 {
                let p = &ps[d.placement];
                let (_, why) = resolve(&level, m, p);
                bad.push(format!("{} plc{} ({})", name, d.placement, why));
            }
        }
    }
    eprintln!("census: {} levels, {} doors, {} glist-0", levels, doors, bad.len());
    assert!(bad.is_empty(), "glist-0 doors (the game creates none [M 0x1d1b68:93-95]): {:?}", bad);
}
