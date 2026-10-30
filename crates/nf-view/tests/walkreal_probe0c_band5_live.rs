//! walkreal probe #11 (level-0c-gate-band-crossing): the forklift hypothesis. Dumps (1) the sim
//! nav paths with a node in the crossing box — name, kind, params, nodes, links — and every
//! path's kind census, (2) who starts script 06000762 (placements carrying that hash in their
//! params, scripts whose Events reference it), (3) the map9 forklift objects' full inventories
//! (glist spheres via the sim world), (4) all placements whose params mention map9's hashes.
//! Diagnostic only, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_forklift() {
    let isopath = iso();

    // 1. The sim's nav paths in the box.
    {
        let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
        w.wait_enabled(3000, "the intro");
        w.wait(60);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== sim nav paths (whole network census: name, kind, nodes) ===");
        for (pi, p) in sim.nav.paths.iter().enumerate() {
            let inbox = p.nodes.iter().any(|n| n.pos[0] > 40.0 && n.pos[0] < 82.0 && n.pos[2] > -218.0 && n.pos[2] < -192.0);
            eprintln!("path{pi} \"{}\" kind {:#x} nodes {} {}", p.name, p.kind, p.nodes.len(), if inbox { "<== BOX" } else { "" });
            if inbox {
                for (ni, n) in p.nodes.iter().enumerate() {
                    let links: Vec<String> = p.adj[ni].iter().map(|&li| {
                        let l = &p.links[li as usize];
                        let other = if l.a as usize == ni { l.b as usize } else { l.a as usize };
                        format!("n{other}(len {:.1})", l.len)
                    }).collect();
                    eprintln!("  n{} ({:.2},{:.2},{:.2}) flags {:#x} {}", ni, n.pos[0], n.pos[1], n.pos[2], n.flags, links.join(" "));
                }
            }
        }
    }

    // 2-4. The disc side: who starts 06000762, and what carries map9's hashes.
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lb = LevelBin::parse(&data).expect("levelbin");
    let level = Level::parse(&data).expect("level");
    let main = level.main_map().expect("main map");

    eprintln!("=== placements whose params mention 0x06000762 or map9 hashes ===");
    let mut map9_hashes: std::collections::HashSet<u32> = Default::default();
    for o in level.maps.iter().flat_map(|m| m.objects.iter()) {
        if o.hashcode >> 16 == 0x0200 {
            // map9's objects: check name-scope later; collect all 0200xxxx hashes with owners
        }
    }
    for (m9i, m) in level.maps.iter().enumerate() {
        for o in m.objects.iter() {
            if o.hashcode >> 16 == 0x0200 { map9_hashes.insert(o.hashcode); }
        }
        let _ = m9i;
    }
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let mention_script = p.params.iter().any(|&(_, v)| v == 0x0600_0762);
            let mention_map9 = p.params.iter().any(|&(_, v)| map9_hashes.contains(&v));
            if !mention_script && !mention_map9 { continue; }
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let name = target.map(|(om, oi)| level.maps[om].objects[oi].name.trim().to_string()).unwrap_or("<unresolved>".into());
            eprintln!("map{mi} plc{pi} class {:#04x} \"{name}\" t ({:.2},{:.2},{:.2}) {}params {:?}", p.flags & 0xff, p.translation[0], p.translation[1], p.translation[2], if mention_script { "SCRIPT06000762 " } else { "" }, p.params);
        }
    }

    eprintln!("=== scripts whose streams mention 06000762 (Event/Anim ops) ===");
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        let Ok(nis) = nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)) else { continue };
        let mut rows: Vec<String> = Vec::new();
        for (si, s) in nis.streams.iter().enumerate() {
            for c in s.decode() {
                let text = format!("{c:?}");
                if text.contains("06000762") || text.contains("98314") {
                    rows.push(format!("  s{si}: {text}"));
                }
            }
        }
        // Also Event ops with any params — print all events of every script for the channel graph.
        if !rows.is_empty() {
            eprintln!("== script {:08x} ==", nis.hashcode);
            for r in rows { eprintln!("{r}"); }
        }
    }

    // 5. Every script's Event ops (the channel graph).
    eprintln!("=== all scripts: Event ops ===");
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        let Ok(nis) = nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)) else { continue };
        let mut rows: Vec<String> = Vec::new();
        for (si, s) in nis.streams.iter().enumerate() {
            for c in s.decode() {
                if let nf_formats::nis::Cmd::Event { kind, params } = c {
                    rows.push(format!("  s{si}: Event kind {kind:#04x} params {params:?}"));
                }
            }
        }
        if !rows.is_empty() {
            eprintln!("== script {:08x} ==", nis.hashcode);
            for r in rows { eprintln!("{r}"); }
        }
    }
    eprintln!("ok");
}
