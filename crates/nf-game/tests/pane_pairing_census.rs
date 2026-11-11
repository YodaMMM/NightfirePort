//! pane-collision-pairing: the placed 0x20 panes' pairing, settled from the game + the disc.
//!
//! The game's rules this print rides on:
//! - `Break_Create` 0x12c230 takes the pane's glist from its CALLER (`Control_SetGList(obj,
//!   param_3)` [M 0x12c230:19 → 0x134960:8 obj+0xD8 = glist, obj+0xF0 = glist+0x44]) and builds
//!   the object matrix with quaternion 0 (`control_create_object(0xC, pos, …, 0)` [M 0x12c230:12]
//!   → the placement's `View_RotTransMatrix`).
//! - The placement path's glist is the record's resolved object: by index `pCelList[index]`
//!   (NULL — no create at all — when index ≥ MapEntityCount) when the record hashcode word is
//!   −1, else the hash table hit [M 0x1d1b68:77-87], and the create is SKIPPED when the record's
//!   flags word carries 0x8000/0x2000 [M 0x1d1b68:93] — such a row never becomes a pane in the
//!   game.
//! - `Collide_PickObj` 0x1eae80 admits a cel-linked object when its collision block is non-empty
//!   (`*(obj+0xD8)+8 != 0`) or obj+0xFA carries bit 8 [M 0x1eae80:88-90]; `Break_Kill` 0x12b970:60
//!   sets obj+0xFE bit 0 (the delete) and :59 reads the same block for the debris.
//!
//! The print, per placed 0x20 row on the five levels the drone.rs comment names (01/09/0a/0b/14):
//! the PART bounds (the port's resolved shape at `b.matrix`) against the PLACEMENT bounds [D]
//! (`b.lo/hi`), the raw shape extent (a pane is thin in exactly one axis — a fat shape is an
//! object that is not a pane), the game-create verdict (flags/no-glist), the duplicate check
//! (a static instance placing the same object), and the shape's materials. Then the census
//! assertion: every row the game WOULD create has its part within its own placement bounds
//! (per axis), on every SP level — the pairing proof the ride needs.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_game::breakable::Breaks;
use nf_game::world::CollisionWorld;

fn level(id: u32) -> Option<Level> {
    let path = std::env::var_os("NF_ISO")?;
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let name = format!("0700{:04x}.bin", id);
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name))?;
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Level::parse(&data).ok()
}

fn aabb_of(tris: &[[[f32; 3]; 3]]) -> ([f32; 3], [f32; 3]) {
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for t in tris { for v in t { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
    (lo, hi)
}

fn fmt_box((lo, hi): ([f32; 3], [f32; 3])) -> String {
    format!("[{:9.3}..{:9.3} {:9.3}..{:9.3} {:9.3}..{:9.3}]", lo[0], hi[0], lo[1], hi[1], lo[2], hi[2])
}

fn gap(a: ([f32; 3], [f32; 3]), b: ([f32; 3], [f32; 3])) -> [f32; 3] {
    [0, 1, 2].map(|k| (b.0[k] - a.1[k]).max(a.0[k] - b.1[k]).max(0.0))
}

fn diag(id: u32, out: &mut String) {
    let Some(level) = level(id) else { out.push_str(&format!("== 0700{:04x}: LEVEL FAILED TO LOAD ==\n", id)); return };
    let Some(m) = level.main_map() else { out.push_str(&format!("== 0700{:04x}: NO MAIN MAP ==\n", id)); return };
    let n20 = level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x20).count();
    out.push_str(&format!("== 0700{:04x}: {} 0x20 placements ==\n", id, n20));
    let (insts, _) = level.instances(true);
    let world = CollisionWorld::build(&level, &insts);
    let br = Breaks::create(&level);
    let mut static_objs: Vec<(usize, usize)> = insts.iter().map(|i| (i.map, i.object)).collect();
    static_objs.sort_unstable();
    let phantoms = br.list.iter().filter(|b| b.shape_key == 0).count();
    out.push_str(&format!("== 0700{:04x}: {} placed 0x20 rows, {} static instances, {} unresolved ==\n",
        id, br.list.len(), insts.len(), phantoms));
    for (k, b) in br.list.iter().enumerate() {
        let p = &level.maps[m].placements[b.placement];
        let res = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
        // The game's create gate [M 0x1d1b68:93-96]: flags & 0xa000 == 0 AND the glist non-NULL.
        let game = if p.flags & 0xa000 != 0 { "SKIPPED-BY-FLAGS" }
            else if res.is_none() { "NO-GLIST" } else { "created" };
        let place = (b.lo, b.hi);
        let (part, raw, ntris, key_kind) = match world.shape(b.shape_key) {
            Some(tris) => {
                let part = aabb_of(&tris.iter().map(|t| t.map(|v| nf_formats::placement::transform(&b.matrix, v))).collect::<Vec<_>>());
                let raw = aabb_of(tris);
                (part, raw, tris.len(), if b.shape_key & 0x8000_0000 != 0 { "anon" } else { "hash" })
            }
            None => (([0.0; 3], [0.0; 3]), ([0.0; 3], [0.0; 3]), 0, "NONE"),
        };
        let d = gap(part, place);
        let ext = [0, 1, 2].map(|k| (raw.1[k] - raw.0[k]).abs());
        let thin = ext.iter().filter(|&&e| e < 0.5).count();
        let dup_static = static_objs.iter().filter(|&&o| Some(o) == res).count();
        let obj_desc = res.map(|(m, o)| format!("({m},{o})")).unwrap_or("none".into());
        let hash_note = res
            .map(|(m, o)| {
                let hc = level.maps[m].objects[o].hashcode;
                format!("hc={hc:08x} key_match={}", level.by_hash.get(&hc).copied() == Some((m, o)))
            })
            .unwrap_or_default();
        let mats = res.and_then(|(m, o)| level.maps[m].objects[o].collision.as_ref())
            .map(|c| { let mut s: Vec<u8> = c.triangles_with_material().into_iter().map(|t| t.1).collect(); s.sort_unstable(); s.dedup(); s })
            .unwrap_or_default();
        // The record's params, the disc's kind/health/filter words [D].
        let param = |kk: u32| p.params.iter().find(|(j, _)| *j == kk).map_or(0, |&(_, v)| v);
        out.push_str(&format!(
            "k={k:3} plc={:4} obj={} flags={:04x} {} key={:08x}({}) tris={:3} thin={} dup_static={} kind={} hp={} filt={:#x}\n   part={} d=({:.2},{:.2},{:.2})\n   place={} ext=({:.2},{:.2},{:.2}) mats={:x?} {}\n",
            b.placement, obj_desc, p.flags, game, b.shape_key, key_kind, ntris, thin, dup_static,
            param(0) & 0xff, param(1) as i32, if param(2) == 0 { 0 } else { !(param(2) as u16) & 0xffff },
            fmt_box(part), d[0], d[1], d[2], fmt_box(place), ext[0], ext[1], ext[2], mats, hash_note,
        ));
    }
}

/// The task's five levels (01, 09, 0a, 0b, 14) — assertion-free print; run with --nocapture.
#[test]
fn pane_part_vs_placement_bounds_on_01_09_0a_0b_14() {
    let mut out = String::new();
    for id in [1u32, 9, 0x0a, 0x0b, 0x14] { diag(id, &mut out); }
    eprintln!("{out}");
}

/// The census: every row the game WOULD create (flags & 0xa000 == 0, glist resolved —
/// [M 0x1d1b68:93-96]) has its part within its own placement bounds on every SP level.
#[test]
fn no_created_pane_part_lies_outside_its_placement_bounds_on_any_sp_level() {
    let Some(path) = std::env::var_os("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let levels: &[&str] = &["07000001", "07000002", "07000003", "07000004", "07000005", "07000007",
        "07000008", "07000009", "0700000a", "0700000b", "0700000c", "0700000d",
        "07000011", "07000012", "07000013", "07000014", "07000015", "07000016", "0700001b",
        "07000041", "0700004a", "0700004b"];
    let mut checked = 0usize;
    let mut skipped = 0usize;
    for lv in levels {
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&format!("{lv}.bin"))) else { continue };
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let Ok(level) = Level::parse(&data) else { continue };
        let Some(m) = level.main_map() else { continue };
        let (insts, _) = level.instances(true);
        let world = CollisionWorld::build(&level, &insts);
        let br = Breaks::create(&level);
        for (k, b) in br.list.iter().enumerate() {
            let p = &level.maps[m].placements[b.placement];
            if p.flags & 0xa000 != 0 { skipped += 1; continue; } // the game never creates it [M 0x1d1b68:93]
            let Some(tris) = world.shape(b.shape_key) else { continue }; // NO-GLIST: no pane in the game
            let part = aabb_of(&tris.iter().map(|t| t.map(|v| nf_formats::placement::transform(&b.matrix, v))).collect::<Vec<_>>());
            let place = (b.lo, b.hi);
            let d = gap(part, place);
            let extent = [0, 1, 2].map(|k| (place.1[k] - place.0[k]).abs());
            for axis in 0..3 {
                assert!(d[axis] <= extent[axis].max(0.5),
                    "{lv} breakable k={k} plc={} axis {axis}: part lies {:.3} from the placement bounds (extent {:.3})\n part {} place {}",
                    b.placement, d[axis], extent[axis], fmt_box(part), fmt_box(place));
            }
            checked += 1;
        }
    }
    eprintln!("census checked {checked} created breakables across the SP levels ({skipped} flag-skipped rows excluded, the game never creates them)");
    assert!(checked > 0, "the census ran over the disc's levels");
}
