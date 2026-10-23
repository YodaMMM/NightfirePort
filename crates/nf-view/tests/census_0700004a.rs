//! level-4a (Tower-2 elevator) census: dump every placement by kind, with params, across ALL maps
//! (the 162 shaft doors may live on sub-maps), plus a mesh-resolution pass (every model draws:
//! no UNRESOLVED hash). Opt-in NF_ISO; prints only, no assertions — the pinned assertions live in
//! interact_0700004a_live.rs.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free census dump; run with --ignored"]
fn dump_0700004a_placements_by_kind() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "0700004a.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap();
    println!("maps {} main {mm} objects[mm] {}", lv.maps.len(), lv.maps[mm].objects.len());

    // All-maps per-kind counts first (the shape of the level at a glance).
    let mut all: Vec<(u32, usize)> = Vec::new();
    for m in &lv.maps {
        for p in &m.placements {
            let k = p.flags & 0xff;
            match all.iter_mut().find(|(kk, _)| *kk == k) {
                Some((_, n)) => *n += 1,
                None => all.push((k, 1)),
            }
        }
    }
    all.sort();
    println!("== ALL-MAPS kind counts ==");
    for (k, n) in &all {
        let per: Vec<String> = lv.maps.iter().enumerate()
            .map(|(mi, m)| (mi, m.placements.iter().filter(|p| p.flags & 0xff == *k).count()))
            .filter(|(_, n)| *n > 0)
            .map(|(mi, n)| format!("m{mi}:{n}"))
            .collect();
        println!("kind {k:#04x}: {n}  ({})", per.join(" "));
    }

    // Full dump, every map, kind-sorted.
    for (mi, m) in lv.maps.iter().enumerate() {
        let placements = &m.placements;
        if placements.is_empty() { continue; }
        println!("== MAP m{mi} (kind {} objects {}) placements {} ==",
            m.kind, m.objects.len(), placements.len());
        let mut by_kind: Vec<(u32, Vec<usize>)> = Vec::new();
        for (i, p) in placements.iter().enumerate() {
            let k = p.flags & 0xff;
            match by_kind.iter_mut().find(|(kk, _)| *kk == k) {
                Some((_, v)) => v.push(i),
                None => by_kind.push((k, vec![i])),
            }
        }
        by_kind.sort();
        for (k, idxs) in &by_kind {
            println!("-- KIND {k:#04x}: {} placements --", idxs.len());
            for &i in idxs {
                let p = &placements[i];
                let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
                let name = target.map(|(ma, o)| lv.maps[ma].objects[o].name.trim().to_string())
                    .unwrap_or_else(|| format!("UNRESOLVED hash {:08x}", p.hashcode));
                let t = p.translation;
                let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
                let cel = if p.creates_cel() { " CEL" } else { "" };
                println!("m{mi} #{i} kind {k:#04x}{cel} \"{}\" @({:.2},{:.2},{:.2}) params [{}] hash {:08x} flags {:#x}",
                    name, t[0], t[1], t[2], params.join(" "), p.hashcode, p.flags);
            }
        }
    }

    // Mesh-resolution pass: every placement must resolve to an object (no UNRESOLVED hash) for
    // its model to draw — the "every model draws" census line.
    println!("== MESH RESOLUTION (unresolved only) ==");
    let mut unresolved = 0usize;
    for (mi, m) in lv.maps.iter().enumerate() {
        for (i, p) in m.placements.iter().enumerate() {
            let ok = if p.by_hash() { lv.by_hash.contains_key(&p.hashcode) }
                else { (p.index as usize) < m.objects.len() };
            if !ok {
                unresolved += 1;
                let t = p.translation;
                println!("UNRESOLVED m{mi} #{i} kind {:#04x} hash {:08x} @({:.2},{:.2},{:.2})",
                    p.flags & 0xff, p.hashcode, t[0], t[1], t[2]);
            }
        }
    }
    println!("mesh resolution: {unresolved} unresolved placements");
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn dump_0700004a_midlanding_geometries() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700004a.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap_or(0);
    println!("== placements with translation y in -125..-90 (all maps) ==");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            if !(-125.0..=-90.0).contains(&p.translation[1]) { continue; }
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let name = target.map(|(m2, o)| lv.maps[m2].objects[o].name.trim().to_string())
                .unwrap_or_else(|| format!("UNRESOLVED {:08x}", p.hashcode));
            println!("m{mi} plc #{pi} kind {:#x} 0x8000 {} \"{}\" @({:.3},{:.3},{:.3})",
                p.flags & 0xffff, p.creates_cel(), name, p.translation[0], p.translation[1], p.translation[2]);
        }
    }
    println!("== instances with collision bbox meeting y -120..-90, x -8..12, z -14..2 ==");
    let (instances, _) = lv.instances(false);
    for (pi, inst) in instances.iter().enumerate() {
        let Some(c) = &lv.maps[inst.map].objects[inst.object].collision else { continue };
        let o = &lv.maps[inst.map].objects[inst.object];
        let m = nf_formats::placement::object_matrix(inst.placement.rotation, inst.placement.translation);
        let mut lo = [f32::MAX; 3]; let mut hi = [f32::MIN; 3]; let mut n = 0;
        for t in c.triangles() {
            for p in t { let w = nf_formats::placement::transform(&m, p);
                for k in 0..3 { lo[k] = lo[k].min(w[k]); hi[k] = hi[k].max(w[k]); } }
            n += 1;
        }
        if lo[1] <= -90.0 && hi[1] >= -120.0 && hi[0] >= -8.0 && lo[0] <= 12.0 && hi[2] >= -14.0 && lo[2] <= 2.0 {
            println!("plc #{pi} \"{}\" m{} o{} kind {:#x} 0x8000 {} glist {:#x} tris {n} bbox lo {:?} hi {:?} plc_t {:?}",
                o.name.trim(), inst.map, inst.object, inst.placement.flags & 0xffff,
                inst.placement.creates_cel(), o.flags, lo, hi, inst.placement.translation);
        }
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn dump_0700004a_lift_platform_mesh() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700004a.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = Level::parse(&data).expect("parse");
    // the lift entity 0x20005b4 (obj_type 31, the platform)
    for h in [0x2000_05b4u32] {
        match lv.by_hash.get(&h) {
            Some((m, o)) => {
                let obj = &lv.maps[*m].objects[*o];
                println!("entity {h:08x} -> m{m} o{o} \"{}\" flags {:#x} min {:?} max {:?}", obj.name.trim(), obj.flags, obj.min, obj.max);
                if let Some(c) = &obj.collision {
                    let mut lo = [f32::MAX; 3]; let mut hi = [f32::MIN; 3];
                    for t in c.triangles() { for p in t { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } } }
                    println!("   collision {} tris raw bbox lo {hi:?} / lo {lo:?}", c.tris.len());
                } else { println!("   no collision block"); }
            }
            None => println!("entity {h:08x} not in by_hash"),
        }
    }
    // all placements that resolve to the lift entity
    for (mi, m) in lv.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let tgt = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() } else { None };
            if tgt == Some((mi, lv.maps[mi].objects.len())) { continue; }
            if let Some((m2, o2)) = tgt {
                if lv.maps[m2].objects[o2].hashcode == 0x2000_05b4 {
                    println!("lift placement m{mi} plc #{pi} trans {:?} rot {:?} flags {:#x}", p.translation, p.rotation, p.flags);
                }
            }
        }
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn dump_0700004a_lift_sp_and_platform() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700004a.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = Level::parse(&data).expect("parse");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((m2, o2)) = target else { continue };
            let o = &lv.maps[m2].objects[o2];
            if !o.name.contains("SCRIPTPLAYER") { continue; }
            println!("m{mi} plc #{pi} \"{}\" kind {:#x} trans {:?} params {:?}", o.name.trim(), p.flags & 0xffff, p.translation, p.params);
        }
    }
    // the objects named like the platform
    for (mi, m) in lv.maps.iter().enumerate() {
        for (oi, o) in m.objects.iter().enumerate() {
            if o.name.to_lowercase().contains("lift") || o.hashcode == 0x2000_05b4 {
                let ctri = o.collision.as_ref().map(|c| {
                    let mut lo = [f32::MAX; 3]; let mut hi = [f32::MIN; 3];
                    for t in c.triangles() { for p in t { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } } }
                    format!("{} tris raw lo {lo:?} hi {hi:?}", c.tris.len())
                }).unwrap_or_else(|| "no collision".into());
                println!("obj m{mi} o{oi} \"{}\" hash {:08x} flags {:#x} min {:?} max {:?} {}", o.name.trim(), o.hashcode, o.flags, o.min, o.max, ctri);
            }
        }
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn dump_0700004a_allmap_geometries_near_handback() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700004a.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = Level::parse(&data).expect("parse");
    // every map, every placement with 0x8000 whose tris reach the handback column
    // (x 0..4, y -115..-100, z -9..-5) or carry ANY z < -6 geometry below y -90
    for (mi, m) in lv.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            if !p.creates_cel() { continue; }
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((m2, o2)) = target else { continue };
            let Some(c) = &lv.maps[m2].objects[o2].collision else { continue };
            let mat = nf_formats::placement::object_matrix(p.rotation, p.translation);
            let mut lo = [f32::MAX; 3]; let mut hi = [f32::MIN; 3]; let mut n = 0;
            for t in c.triangles() {
                for q in t { let w = nf_formats::placement::transform(&mat, q);
                    for k in 0..3 { lo[k] = lo[k].min(w[k]); hi[k] = hi[k].max(w[k]); } }
                n += 1;
            }
            let hits_column = hi[0] >= 0.0 && lo[0] <= 4.0 && hi[1] >= -115.0 && lo[1] <= -100.0 && hi[2] >= -9.0 && lo[2] <= -5.0;
            let south_of_bore_below_90 = lo[2] < -6.0 && hi[1] <= -90.0;
            if hits_column || south_of_bore_below_90 {
                println!("m{mi} plc #{pi} \"{}\" kind {:#x} 0x8000 {} glist {:#x} tris {} bbox lo {:?} hi {:?} t {:?}",
                    lv.maps[m2].objects[o2].name.trim(), p.flags & 0xffff, p.creates_cel(),
                    lv.maps[m2].objects[o2].flags, n, lo, hi, p.translation);
            }
        }
    }
    println!("maps: {}", lv.maps.len());
}
