//! dead-pane-collision-2: the pane part ↔ placement pairing diagnostic + the sister census.
//!
//! The task's three hypotheses for the phantom collision the pane parts added (the tests broke
//! on 01/09 with blockers where no pane stands):
//! 1. the shape is already in the static world (a double — the object cel-placed elsewhere);
//! 2. its triangle data is already world space, so `b.matrix` moves it a second time;
//! 3. `shape_key` resolves to the wrong object (by_hash vs the anonymous synthetic key).
//!
//! The print: per breakable, the PART bounds (the shape the `set_dynamic` part carries,
//! transformed by the pane's `b.matrix`) against the PLACEMENT bounds (`Breaks::create`'s
//! `bounds()`, the glist bounds the collision pass would register — `b.lo/hi`). Plus the raw
//! (object-space) shape AABB, and how many static instances place the same object.
//!
//! The game's rule: the pane object carries its own collision block at `obj+0xd8 → +8`
//! [M Break_Kill 0x12b970:59-63 — `iVar15 = *(*(obj+0xd8)+8)`, the dead flag obj+0xFE |= 1,
//! and the `if (iVar15 == 0) return` silent-kill gate], created by `Break_Create` 0x12c230's
//! `Control_SetGList(obj, celglist)` — the glist the placement resolves to, and
//! `Collide_PickObj` walks the cel-linked objects' shapes at the object's own matrix.
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
    let Some(level) = level(id) else { return };
    let Some(mm) = level.main_map() else { return };
    let (insts, _) = level.instances(true);
    let world = CollisionWorld::build(&level, &insts);
    let br = Breaks::create(&level);
    // The static instances' (map, object) multiset — the double check (hypothesis 1).
    let mut static_objs: Vec<(usize, usize)> = insts.iter().map(|i| (i.map, i.object)).collect();
    static_objs.sort_unstable();
    out.push_str(&format!("== 0700{:04}: {} breakables, {} static instances ==\n", id, br.list.len(), insts.len()));
    for (k, b) in br.list.iter().enumerate() {
        let p = &level.maps[mm].placements[b.placement];
        let res = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < level.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
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
        let d_raw = gap(raw, place);
        let dup_static = static_objs.iter().filter(|&&o| Some(o) == res).count();
        let obj_desc = res.map(|(m, o)| format!("({m},{o})")).unwrap_or("none".into());
        let hash_note = res
            .map(|(m, o)| {
                let hc = level.maps[m].objects[o].hashcode;
                format!("hc={hc:08x} key_match={}", level.by_hash.get(&hc).copied() == Some((m, o)))
            })
            .unwrap_or_default();
        // The shape's triangle material bytes — the glass rule reads these (class 0x10).
        let mats = res.and_then(|(m, o)| level.maps[m].objects[o].collision.as_ref())
            .map(|c| { let mut s: Vec<u8> = c.triangles_with_material().into_iter().map(|t| t.1).collect(); s.sort_unstable(); s.dedup(); s })
            .unwrap_or_default();
        out.push_str(&format!(
            "k={k:3} plc={:4} obj={} key={:08x}({}) tris={:3} dup_static={} {}\n   part={} d=({:.2},{:.2},{:.2})\n   place={}\n   raw={} raw_d=({:.2},{:.2},{:.2})\n",
            b.placement, obj_desc, b.shape_key, key_kind, ntris, dup_static, hash_note,
            fmt_box(part), d[0], d[1], d[2], fmt_box(place), fmt_box(raw), d_raw[0], d_raw[1], d_raw[2],
        ));
        if !mats.is_empty() { out.push_str(&format!("   mats={mats:x?}\n")); }
    }
}

/// The task's four diagnostic levels (01, 08, 09, 0a). Assertion-free print: run with
/// `--nocapture` and read the table; the census assertion lives in the all-levels test below.
#[test]
fn pane_part_vs_placement_bounds_on_01_08_09_0a() {
    let mut out = String::new();
    for id in [1u32, 7, 8, 9, 0x0a] { diag(id, &mut out); }
    eprintln!("{out}");
}

/// The sister census over every SP level: no pane part lies more than its own bounds away
/// from its placement bounds (per axis: the gap between the part box and the placement box
/// never exceeds the placement box's own extent on that axis).
#[test]
fn no_pane_part_lies_outside_its_placement_bounds_on_any_sp_level() {
    let Some(path) = std::env::var_os("NF_ISO") else { return };
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let levels: &[&str] = &["07000001", "07000002", "07000003", "07000004", "07000005", "07000008",
        "07000007", "07000008", "07000009", "0700000a", "0700000b", "0700000c", "0700000d",
        "07000011", "07000012", "07000013", "07000014", "07000015", "07000016", "0700001b",
        "07000041", "0700004a", "0700004b"];
    let mut checked = 0usize;
    for lv in levels {
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&format!("{lv}.bin"))) else { continue };
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let Ok(level) = Level::parse(&data) else { continue };
        let (insts, _) = level.instances(true);
        let world = CollisionWorld::build(&level, &insts);
        let br = Breaks::create(&level);
        for (k, b) in br.list.iter().enumerate() {
            let Some(tris) = world.shape(b.shape_key) else { continue };
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
    eprintln!("census checked {checked} breakables across the SP levels");
    assert!(checked > 0, "the census ran over the disc's levels");
}
