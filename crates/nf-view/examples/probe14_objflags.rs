//! Scratch probe (level-14-lid-and-climb): the map objects behind the lid slabs and the ride
//! platforms — their glist flags words. The game's static pick skips `cel+0x94 & 0x20`
//! [M Collide_Pick 0x1eb320:140]; a created script entity attaches the SAME glist, so its
//! cel+0x94 carries the object's own word [M 0x1d1998:102-103]. Assertion-free.
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

    let hashes: [u32; 6] = [
        0x0200_0750, // the lid slab the vent SPs create (kind 0x31)
        0x0200_0751, // the ladder art (kind 0x19)
        0x0200_0aa1, // a real fusebox wire entity (created by 0x06000A1B)
        0x0200_0367, // the paternoster cab (the ride platform — solid in the game)
        0x0200_00db, // a Door_Slide (never collides as a static)
        0x0200_0020, // a Breakable window
    ];
    for h in hashes {
        match level.by_hash.get(&h) {
            Some(&(m, o)) => {
                let obj = &level.maps[m].objects[o];
                println!(
                    "{h:08x}: map {m} object {o} flags {:#010x} (0x20 never-pick {}, 0x40 bullet-pass {}, 0x1000 cage {}) collision {} tris",
                    obj.flags,
                    obj.flags & 0x20 != 0,
                    obj.flags & 0x40 != 0,
                    obj.flags & 0x1000 != 0,
                    obj.collision.as_ref().map_or(0, |c| c.triangles().len()),
                );
            }
            None => println!("{h:08x}: not in 07000014's maps"),
        }
    }

    println!("-- every object whose flags carry 0x20, with collision --");
    for (m, map) in level.maps.iter().enumerate() {
        for (o, obj) in map.objects.iter().enumerate() {
            if obj.flags & 0x20 != 0 && obj.collision.is_some() {
                println!("  map {m} object {o} hash {:08x} flags {:#010x}", obj.hashcode, obj.flags);
            }
        }
    }
}
