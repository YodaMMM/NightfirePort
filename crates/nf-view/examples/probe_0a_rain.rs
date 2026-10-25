//! Scratch probe (level-0a-rain-indoors): the 0700000a weather data [D] — the 0xDF Env placement,
//! every 0x32 RainBox placement with its resolved object's glist bounds + translation, and the
//! port's boxes/alive drops after load + ticks. Run: NF_ISO=... cargo run --release -p nf-view --example probe_0a_rain
fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    let iso = nf_formats::iso9660::IsoImage::open(&path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000a.bin")).expect("0700000a.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = nf_formats::level::Level::parse(&data).unwrap();
    let main = level.main_map().expect("main map");
    let ps = &level.maps[main].placements;
    println!("main map {} placements {}", main, ps.len());
    println!("== 0xDF Env placements ==");
    for (i, p) in ps.iter().enumerate() {
        if p.flags & 0xff == 0xdf {
            println!("  #{i} idx{} hash {:?} flags {:#x} (0xa000 bits {}) t {:?} rot {:?} params {:?}",
                p.index, p.hashcode, p.flags, p.flags & 0xa000, p.translation, p.rotation, p.params);
        }
    }
    println!("== 0x32 RainBox placements ==");
    for (i, p) in ps.iter().enumerate() {
        if p.flags & 0xff == 0x32 {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < level.maps[main].objects.len() { Some((main, p.index as usize)) } else { None };
            match target {
                Some((m, o)) => {
                    let ob = &level.maps[m].objects[o];
                    let t = p.translation;
                    let lo = [ob.min[0] + t[0], ob.min[1] + t[1], ob.min[2] + t[2]];
                    let hi = [ob.max[0] + t[0], ob.max[1] + t[1], ob.max[2] + t[2]];
                    println!("  #{i} idx{} hash {:?} flags {:#x} (0xa000 {}) obj map{}#{} '{}' gmin {:?} gmax {:?} t {:?} -> box lo {:?} hi {:?}",
                        p.index, p.hashcode, p.flags, p.flags & 0xa000, m, o, ob.name, ob.min, ob.max, t, lo, hi);
                }
                None => println!("  #{i} idx{} hash {:?} flags {:#x} UNRESOLVED t {:?}",
                    p.index, p.hashcode, p.flags, p.translation),
            }
        }
    }
    // The office-lounge spot from the user's report.
    let spot = [-37.50f32, 1.11, -16.95];
    println!("== containment for the office spot {:?} ==", spot);
    for (i, p) in ps.iter().enumerate() {
        if p.flags & 0xff != 0x32 { continue; }
        let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < level.maps[main].objects.len() { Some((main, p.index as usize)) } else { None };
        let Some((m, o)) = target else { continue };
        let ob = &level.maps[m].objects[o];
        let t = p.translation;
        let lo = [ob.min[0] + t[0], ob.min[1] + t[1], ob.min[2] + t[2]];
        let hi = [ob.max[0] + t[0], ob.max[1] + t[1], ob.max[2] + t[2]];
        let xz = lo[0] < spot[0] && spot[0] < hi[0] && lo[2] < spot[2] && spot[2] < hi[2];
        let y = lo[1] < spot[1] && spot[1] < hi[1];
        println!("  #{i}: xz {} y {} (lo {:?} hi {:?})", xz, y, lo, hi);
    }
    // The port's sim: kind + boxes + alive drops after ticks.
    let mut scene = nf_view::load(&path, "0700000a.bin", false).expect("load scene");
    let sim = scene.sim.as_mut().expect("sim");
    let input = nf_game::player::Input::default();
    for _ in 0..120 { sim.step_bond(input, false, false, false); }
    println!("== sim env (after 120 ticks) ==");
    println!("  bond pos {:?} eye {:?}", sim.bond.pos, sim.bond.eye);
    let env_info = sim.drones.env.as_ref().map(|env| {
        (env.kind, env.boxes.clone(), env.init_bodge, env.drops.len(), env.centre,
            env.drops.clone(), env.alive.clone())
    });
    match env_info {
        None => println!("  NO env (None)"),
        Some((kind, boxes, bodge, n, centre, drops, alive)) => {
            println!("  kind {} boxes {} init_bodge {} max {} centre {:?}", kind, boxes.len(), bodge, n, centre);
            for b in &boxes { println!("    box lo {:?} hi {:?}", b.min, b.max); }
            let alive_n = alive.iter().filter(|&&a| a).count();
            let mut lo = [f32::MAX; 3]; let mut hi = [f32::MIN; 3];
            for (p, &a) in drops.iter().zip(&alive) {
                if !a { continue; }
                for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); }
            }
            println!("  after ticks: alive {}/{} drop bbox lo {:?} hi {:?}", alive_n, n, lo, hi);
            // Drops within 5 m of the office spot.
            let spot = [-37.50f32, 1.11, -16.95];
            let near = drops.iter().zip(&alive).filter(|(p, &a)| a &&
                (p[0]-spot[0]).powi(2) + (p[1]-spot[1]).powi(2) + (p[2]-spot[2]).powi(2) < 25.0).count();
            println!("  alive drops within 5 m of the office spot: {}", near);
        }
    }
}
