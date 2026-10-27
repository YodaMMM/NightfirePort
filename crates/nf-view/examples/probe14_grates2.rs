// level-14-vent-grates scratch 2: the full placement records for the grate family — flags
// (CREATE_CEL?), params (the class/creator words), extra/scale — plus the object's gfx/kind
// fields and how the port's session draw classifies them. Assertion-free.
use nf_formats::level::Level;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = nf_formats::iso9660::IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let level = Level::parse(&data).expect("parse 07000014");

    for (m, map) in level.maps.iter().enumerate() {
        for (p, pl) in map.placements.iter().enumerate() {
            if pl.hashcode != 0x0200_0ab4 && pl.hashcode != 0x0200_0750 { continue; }
            let Some(&(om, oo)) = level.by_hash.get(&pl.hashcode) else { continue };
            let obj = &level.maps[om].objects[oo];
            println!(
                "map {m} pl {p}: hash {:08x} flags {:#08x} (cel {} neverpick {} bulletpass {}) trans ({:.2}, {:.2}, {:.2}) rot ({:.3}, {:.3}, {:.3})",
                pl.hashcode, pl.flags, pl.creates_cel(), obj.flags & 0x20 != 0, obj.flags & 0x40 != 0,
                pl.translation[0], pl.translation[1], pl.translation[2],
                pl.rotation[0], pl.rotation[1], pl.rotation[2]
            );
            println!("  params: {:?}", pl.params.iter().map(|(a, b)| format!("{a:#x}={b:#x}")).collect::<Vec<_>>());
            println!("  extra: {:?}", pl.extra.iter().map(|w| format!("{w:#x}")).collect::<Vec<_>>());
            println!("  scale: {:?}", pl.scale());
            println!("  obj: flags {:#x} centre {:?} radius {:.2} min {:?} max {:?} collision {}",
                obj.flags, obj.centre, obj.radius, obj.min, obj.max,
                obj.collision.as_ref().map_or(0, |c| c.triangles().len()));
        }
    }
    // The kind bytes: how does the port's draw classify these? Search the session's object list.
    // Also: the two unnamed "?" placements near the mouths (276/277/302/334/336/337/781/852) —
    // their hashcodes are ffffffff, meaning cel-attached art, not placements.
    println!("== the gfx kind of 02000ab4 across maps ==");
    for (m, map) in level.maps.iter().enumerate() {
        for obj in map.objects.iter() {
            if obj.hashcode == 0x0200_0ab4 {
                println!("map {m}: name {:?} flags {:#x} collision {}", obj.name, obj.flags,
                    obj.collision.as_ref().map_or(0, |c| c.triangles().len()));
            }
        }
    }
}
