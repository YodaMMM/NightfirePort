
#[path = "../tests/walkreal_07000005_live.rs"]
mod t05;
use nf_formats::level::Level;
fn main() {
    let iso_path = std::env::var("NF_ISO").unwrap();
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let (instances, _) = level.instances(true);
    for (idx, inst) in instances.iter().enumerate() {
        let om = &level.maps[inst.map].objects[inst.object];
        // kink region: name contains Outerwall or Road, or flags bit 0x200
        if om.flags & 0x200 != 0 || om.name.contains("Outerwall") || om.name.contains("Road") {
            let c = om.collision.as_ref().map_or(0, |c| c.triangles_with_material().len());
            println!("inst {:3} {} flags {:#x} tris {}", idx, om.name, om.flags, c);
        }
    }
}
