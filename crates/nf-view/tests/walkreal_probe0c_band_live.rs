//! walkreal probe #7 (level-0c-gate-band-crossing): the gate band x 55.5..67, straight from the
//! disc parse. Dumps (1) every placement of every map over the crossing box, (2) every
//! collision-bearing object whose transformed tris reach the band slab (y 9.5..12, x 53..70,
//! z −210..−204) with the placement's cel/instance status — the "who floors the band" census the
//! fix hangs on, (3) every kind-7 script's Entity/Event ops (spawns + channels), (4) path17 in
//! full. Diagnostic only, run with --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn in_box(t: [f32; 3], x0: f32, x1: f32, z0: f32, z1: f32) -> bool {
    t[0] >= x0 && t[0] <= x1 && t[2] >= z0 && t[2] <= z1
}

#[test]
fn probe_band_placements_and_collision() {
    let iso = iso();
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lb = LevelBin::parse(&data).expect("levelbin");
    let level = Level::parse(&data).expect("level");
    let main = level.main_map().expect("main map");
    eprintln!("maps {} (main {main}), main placements {}", level.maps.len(), level.maps[main].placements.len());

    // 1. Every placement over the crossing box (any y), every map.
    eprintln!("=== placements in the box x 40..82 z -218..-192 (all maps) ===");
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            if !in_box(p.translation, 40.0, 82.0, -218.0, -192.0) { continue; }
            let obj = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let (oname, ohash, ocol, oflags) = match obj {
                Some((om, oi)) => {
                    let o = &level.maps[om].objects[oi];
                    (o.name.trim().to_string(), o.hashcode, o.collision.as_ref().map(|c| c.triangles().len()), o.flags)
                }
                None => ("<unresolved>".into(), 0, None, 0),
            };
            eprintln!("map{mi} plc{pi} class {:#04x} flags {:#010x} cel {} hash {:#010x} t ({:.2},{:.2},{:.2}) obj \"{oname}\" objhash {ohash:#010x} objflags {oflags:#x} col_tris {ocol:?} params {:?}",
                p.flags & 0xff, p.flags, p.creates_cel(), p.hashcode, p.translation[0], p.translation[1], p.translation[2], p.params);
        }
    }

    // 2. The band-slab census: every collision-bearing object/placement pair whose WORLD tris
    //    enter the slab y 9.5..12 x 53..70 z −210..−204. Includes creates_cel rows, other-map
    //    0x19 rows, and — the point — any row the port's instance lists skip.
    eprintln!("=== collision in the band slab y 9.5..12 x 53..70 z -210..-204 ===");
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { continue };
            let Some(col) = level.maps[om].objects[oi].collision.as_ref() else { continue };
            let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
            let mut hits = 0usize;
            let mut sample: Option<[f32; 3]> = None;
            for t in col.triangles() {
                let w: Vec<[f32; 3]> = t.iter().map(|&v| plc::transform(&matrix, v)).collect();
                let ok = w.iter().all(|v| v[1] >= 9.5 && v[1] <= 12.0 && v[0] >= 53.0 && v[0] <= 70.0 && v[2] >= -210.0 && v[2] <= -204.0);
                if ok { hits += 1; if sample.is_none() { sample = Some(w[0]); } }
            }
            if hits > 0 {
                eprintln!("map{mi} plc{pi} class {:#04x} cel {} flags {:#010x} obj \"{}\" {:#010x}: {hits} tris in slab, e.g. {:?}",
                    p.flags & 0xff, p.creates_cel(), p.flags, level.maps[om].objects[oi].name.trim(), level.maps[om].objects[oi].hashcode, sample.unwrap());
            }
        }
    }

    // 3. Every kind-7 script: Entity spawns and Event ops, with channels.
    eprintln!("=== kind-7 scripts: Entity/Event ops ===");
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        let Ok(nis) = nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)) else { continue };
        let mut lines: Vec<String> = Vec::new();
        for (si, s) in nis.streams.iter().enumerate() {
            for c in s.decode() {
                match &c {
                    nf_formats::nis::Cmd::Entity { hash, parent, spline, obj_type, params } =>
                        lines.push(format!("  s{si}: Entity glist {hash:08x} parent {parent:?} spline {spline} type {obj_type:#04x} params {params:?}")),
                    nf_formats::nis::Cmd::Event { kind, params } =>
                        lines.push(format!("  s{si}: Event kind {kind:#04x} params {params:?}")),
                    _ => {}
                }
            }
        }
        if !lines.is_empty() {
            eprintln!("== script {:08x} streams {} ==", nis.hashcode, nis.streams.len());
            for l in lines { eprintln!("{l}"); }
        }
    }

    // 4. path17 in full, and every nav path with a node in the box.
    if let Some(net) = &level.maps[main].ai_network {
        for (pi, p) in net.paths.iter().enumerate() {
            let any = p.nodes.iter().any(|n| in_box(n.pos, 40.0, 82.0, -218.0, -192.0));
            if !any { continue; }
            eprintln!("== path{pi} \"{}\" nodes {} ==", p.name, p.nodes.len());
            for (ni, n) in p.nodes.iter().enumerate() {
                let links: Vec<String> = p.links.iter().enumerate()
                    .filter(|(_, l)| l.a as usize == ni || l.b as usize == ni)
                    .map(|(_, l)| if l.a as usize == ni { format!("n{}", l.b) } else { format!("n{}", l.a) })
                    .collect();
                eprintln!("  n{} ({:.2},{:.2},{:.2}) raw {:?} {}", ni, n.pos[0], n.pos[1], n.pos[2], n.raw, links.join(" "));
            }
        }
    }

    eprintln!("ok");
}
