//! level-13-lobby-rifles probe: name the 07000012 instances the world trace stops the lane's
//! rays on. The walkreal geom dump (NF_RF_GEOM) prints "stop … inside run R owner N bounds …";
//! this probe resolves owner ordinals against the sim's own instance list (the loader's
//! cels_only build + the two retains) and prints each one's object, placement and world bounds.
//! Opt-in NF_ISO; prints only.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement::transform;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

/// The loader's exact PUSHED-instance list for the SIM's collision world: scene.rs feeds
/// `level.instances(true).0` into `CollisionWorld::build_scoped`, whose push filter keeps an
/// instance only when its object carries collision and its flags lack the 0x20 shadow bit and
/// the 0x1000 missile-cage bit — `stat.owners` holds THAT pushed ordinal.
fn sim_instances(lv: &Level) -> Vec<nf_formats::level::Instance> {
    lv.instances(true).0.into_iter()
        .filter(|i| lv.maps[i.map].objects[i.object].collision.is_some()
            && i.placement.flags & 0x20 == 0 && i.placement.flags & 0x1000 == 0)
        .collect()
}

fn dump_instance(lv: &Level, instances: &[nf_formats::level::Instance], want: usize) {
    let Some(inst) = instances.get(want) else { eprintln!("instance {want}: out of range"); return };
    let obj = &lv.maps[inst.map].objects[inst.object];
    let Some(c) = &obj.collision else {
        eprintln!("instance {want}: map{} obj{} \"{}\" NO collision", inst.map, inst.object, obj.name.trim());
        return;
    };
    let mut lo = [f32::INFINITY; 3];
    let mut hi = [f32::NEG_INFINITY; 3];
    for (t, _) in c.triangles_with_material() {
        let t = t.map(|p| transform(&inst.matrix, p));
        for p in &t { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } }
    }
    eprintln!("instance {want}: map{} obj{} \"{}\" kind {:#04x} plc @({:.2},{:.2},{:.2}) flags {:#x} hash {:08x} tris {} bounds lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2})",
        inst.map, inst.object, obj.name.trim(), inst.placement.flags & 0xff,
        inst.placement.translation[0], inst.placement.translation[1], inst.placement.translation[2],
        inst.placement.flags, inst.placement.hashcode, c.triangles().len(),
        lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
}

#[test]
#[ignore = "assertion-free instance dump; run with --ignored"]
fn dump_07000012_lane_stop_instances() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000012.bin");
    let lv = Level::parse(&data).expect("parse");
    let instances = sim_instances(&lv);
    eprintln!("sim instances: {}", instances.len());
    // The walkreal geom dump's owners on the 07000012 south-east-bend lane (the runs holding the
    // centre-ray stop points): the small object at (15.2..15.7, 0.9..1.6, -2.1..-1.3) is 318, the
    // big east mesh is 302.
    for want in [302usize, 303, 318] {
        dump_instance(&lv, &instances, want);
    }
}
