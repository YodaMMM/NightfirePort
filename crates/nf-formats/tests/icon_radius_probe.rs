//! context-icon-arm-radius probe: the 0xFA third-icon placements' radius inputs on the disc.
//! Assertion-free dump (opt-in NF_ISO, run with --ignored): per icon the placement scale words,
//! the resolved object record's centre/radius (the glist bounds — Entity params +0x08/+0x14
//! == celglist+0x10/+0x1c [docs/fields/glist.md]), the port's current radius
//! (`wire::icons`: o.radius × ∏|placement scale|) and the game's runtime radius
//! (`Control_BuildWorldSph` 0x133070:48: glist+0x1c × |obj+0xe8|; ThirdIcon_Create 0x197070
//! never writes obj+0xe8 and control_init_object 0x134630:17 sets it to 1.0 → the scale does
//! NOT apply). Numbers only.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn dump(archive: &str) {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, archive);
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap();
    println!("== {archive} ==");
    for (i, p) in lv.maps[mm].placements.iter().enumerate() {
        if p.flags & 0xff != 0xfa { continue; }
        let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
        let Some((om, oi)) = target else {
            println!("#{i} UNRESOLVED hash {:08x} @({:?})", p.hashcode, p.translation);
            continue;
        };
        let o = &lv.maps[om].objects[oi];
        let ctx = p.params.iter().find(|(k, _)| *k == 0).map_or(0, |&(_, v)| v);
        let chan = p.params.iter().find(|(k, _)| *k == 1).map_or(0, |&(_, v)| v);
        let scales: Vec<f32> = [3usize, 4, 5, 6].iter().map(|&k| f32::from_bits(p.extra[k])).collect();
        let port_scale: f32 = [4usize, 5, 6].iter().map(|&k| f32::from_bits(p.extra[k]).max(1e-6).abs()).product();
        let t = p.translation;
        let c = o.centre;
        // The port's centre (wire::icons): yaw-only rotation of o.centre + translation.
        let (s, co) = (p.rotation[1].sin(), p.rotation[1].cos());
        let port_centre = [t[0] + c[0] * co + c[2] * s, t[1] + c[1], t[2] - c[0] * s + c[2] * co];
        println!("#{i} ctx {ctx} ch {chan} \"{}\" hash {:08x}", o.name.trim(), p.hashcode);
        println!("   trans ({:.4},{:.4},{:.4}) rot ({:.4},{:.4},{:.4}) extra_f32[3..7] = {:.4?}",
            t[0], t[1], t[2], p.rotation[0], p.rotation[1], p.rotation[2], &scales[1..]);
        println!("   obj centre ({:.4},{:.4},{:.4}) radius {:.4} min ({:.2},{:.2},{:.2}) max ({:.2},{:.2},{:.2})",
            c[0], c[1], c[2], o.radius, o.min[0], o.min[1], o.min[2], o.max[0], o.max[1], o.max[2]);
        println!("   port centre ({:.4},{:.4},{:.4})  port radius {:.4} = o.radius × ∏|scale[4..7]| {:.4}",
            port_centre[0], port_centre[1], port_centre[2], o.radius * port_scale, port_scale);
        println!("   GAME radius (obj+0xe8=1.0) = o.radius raw = {:.4}", o.radius);
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn dump_fa_icons_08_and_05() {
    dump("07000008.bin");
    dump("07000005.bin");
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn dump_fa_icons_all_maps_08() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000008.bin");
    let lv = Level::parse(&data).expect("parse");
    for (mi, m) in lv.maps.iter().enumerate() {
        let n = m.placements.iter().filter(|p| p.flags & 0xff == 0xfa).count();
        println!("map {mi}: {} placements, {} kind-0xFA", m.placements.len(), n);
    }
    for (mi, m) in lv.maps.iter().enumerate() {
        for (i, p) in m.placements.iter().enumerate() {
            if p.flags & 0xff != 0xfa { continue; }
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { println!("map {mi} #{i} UNRESOLVED"); continue };
            let o = &lv.maps[om].objects[oi];
            let ctx = p.params.iter().find(|(k, _)| *k == 0).map_or(0, |&(_, v)| v);
            let chan = p.params.iter().find(|(k, _)| *k == 1).map_or(0, |&(_, v)| v);
            let scales: Vec<f32> = [4usize, 5, 6].iter().map(|&k| f32::from_bits(p.extra[k])).collect();
            let t = p.translation;
            println!("map {mi} #{i} ctx {ctx} ch {chan} \"{}\" @({:.4},{:.4},{:.4}) scale {:.3?} r {:.4}",
                o.name.trim(), t[0], t[1], t[2], scales, o.radius);
        }
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn dump_wires_08() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000008.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap();
    for (i, p) in lv.maps[mm].placements.iter().enumerate() {
        if p.flags & 0xff != 0x3e { continue; }
        let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
        let Some((om, oi)) = target else { println!("#{i} wire UNRESOLVED"); continue };
        let o = &lv.maps[om].objects[oi];
        let t = p.translation;
        println!("#{i} wire \"{}\" @({:.4},{:.4},{:.4}) rot ({:.4},{:.4},{:.4}) centre ({:.4},{:.4},{:.4}) r {:.4} min ({:.2},{:.2},{:.2}) max ({:.2},{:.2},{:.2})",
            o.name.trim(), t[0], t[1], t[2], p.rotation[0], p.rotation[1], p.rotation[2],
            o.centre[0], o.centre[1], o.centre[2], o.radius, o.min[0], o.min[1], o.min[2], o.max[0], o.max[1], o.max[2]);
    }
}
