// level-14-vent-grates scratch 4: the sim's resolved entity matrices for the grate script's
// nine copies vs the placements' own matrices — and the resulting panel corners in world
// space. Assertion-free.
fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = nf_formats::level::Level::parse(&data).expect("parse");

    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..70 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }

    let mut grate_placements: Vec<(usize, [f32; 3], [f32; 3])> = Vec::new();
    if let Some(m) = level.main_map() {
        for (p, pl) in level.maps[m].placements.iter().enumerate() {
            if pl.hashcode == 0x0200_0ab4 {
                grate_placements.push((p, pl.translation, pl.rotation));
            }
        }
    }
    println!("grate placements: {}", grate_placements.len());

    // The sim's SP players: placement index, scripts, modes, state, clock.
    {
        let sps = &sim.drones.scripts;
        println!("== the SP players on the grate script ==");
        for p in sps.players.iter() {
            if p.scripts.contains(&0x0600_0a3b) {
                println!("pl {} scripts {:08x?} modes {:?} mask {} state {} clock? cur {} activated {}",
                    p.placement, p.scripts, p.modes, p.mask, p.state, p.cur, p.activated);
            }
        }
        println!("== the entities in the Objects list ==");
        for (src, stream, hash, m, _a) in sps.entities() {
            if hash == 0x0200_0ab4 || hash == 0x0200_0ab5 || hash == 0x0200_0750 {
                println!("src {:08x} s{} hash {:08x} origin ({:.2}, {:.2}, {:.2})",
                    src, stream, hash, m[3][0], m[3][1], m[3][2]);
            }
        }
    }

    // The sim's entity matrices per copy k.
    for (k, (pi, t, r)) in grate_placements.iter().enumerate() {
        let m = sim.script_entity_k((0x0600_0a3b, 0), 0x0200_0ab4, k);
        match m {
            Some(m) => {
                // The panel's world corners: object-space (x, y, z) over {±0.74, ±0.025, −1.44..0}
                // — the four corners of the panel's FACE at y = +0.025 and the origin.
                let corner = |x: f32, y: f32, z: f32| nf_formats::placement::transform(&m, [x, y, z]);
                let c00 = corner(-0.74, 0.025, 0.0);
                let c10 = corner(0.74, 0.025, 0.0);
                let c01 = corner(-0.74, 0.025, -1.44);
                let c11 = corner(0.74, 0.025, -1.44);
                println!("pl {pi} t({:.2},{:.2},{:.2}) r({:.3},{:.3},{:.3}): entity origin ({:.2}, {:.2}, {:.2})",
                    t[0], t[1], t[2], r[0], r[1], r[2], m[3][0], m[3][1], m[3][2]);
                println!("   face corners: ({:.2}, {:.2}, {:.2}) ({:.2}, {:.2}, {:.2}) | deep: ({:.2}, {:.2}, {:.2}) ({:.2}, {:.2}, {:.2})",
                    c00[0], c00[1], c00[2], c10[0], c10[1], c10[2], c01[0], c01[1], c01[2], c11[0], c11[1], c11[2]);
            }
            None => println!("pl {pi}: NO entity matrix for copy {k}"),
        }
    }

// The placement-matrix panel corners for the two driver-visible grates.
println!("== the placement-matrix corners ==");
for (pi, t, r) in grate_placements.iter() {
    if *pi != 848 && *pi != 849 && *pi != 266 { continue; }
    let m = nf_formats::placement::view_rot_trans_matrix(*r, *t); // the [M 0x1e5168] SP builder
    let corner = |x: f32, y: f32, z: f32| nf_formats::placement::transform(&m, [x, y, z]);
    let c00 = corner(-0.74, 0.025, 0.0);
    let c10 = corner(0.74, 0.025, 0.0);
    let c01 = corner(-0.74, 0.025, -1.44);
    let c11 = corner(0.74, 0.025, -1.44);
    let ctr = nf_formats::placement::transform(&m, [0.0, 0.0, -0.7175]);
    println!("pl {pi} t({:.2},{:.2},{:.2}) r({:.3},{:.3},{:.3})",
        t[0], t[1], t[2], r[0], r[1], r[2]);
    println!("   origin face: ({:.2}, {:.2}, {:.2}) ({:.2}, {:.2}, {:.2})",
        c00[0], c00[1], c00[2], c10[0], c10[1], c10[2]);
    println!("   deep   face: ({:.2}, {:.2}, {:.2}) ({:.2}, {:.2}, {:.2})",
        c01[0], c01[1], c01[2], c11[0], c11[1], c11[2]);
    println!("   bounds centre: ({:.2}, {:.2}, {:.2})", ctr[0], ctr[1], ctr[2]);
}
}
