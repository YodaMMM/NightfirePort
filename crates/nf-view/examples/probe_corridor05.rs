//! The corridor + the route south of the crawl exit (level-05-every-objective-3): fine floor
//! maps around the crawl exit (−34.38, 37.00, −36.07) → the roofed corridor (x −36, z −38..−44)
//! → the second crack (z −46..−50) → the south gallery (z −52..−60) → the SW loop → the SWITCH
//! plate #269 (−22.76, 37.15, −72.36) → the moat inlet + pulley #281 (−3.40, 42.94, −71.77) →
//! the east battlement → the stairs → door #112 (37.25, 49.00, −69.00). Scratch probe.

use nf_formats::level::Level;

fn main() {
    let iso = nf_formats::iso9660::IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO")).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).expect("07000005.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse 07000005");
    let (instances, _missing) = level.instances(true);
    let tx = |m: &[[f32; 4]; 4], p: [f32; 3]| -> [f32; 3] { nf_formats::placement::transform(m, p) };

    struct W { v: [[f32; 3]; 3] }
    let mut world: Vec<W> = Vec::new();
    for inst in instances.iter() {
        let om = &level.maps[inst.map].objects[inst.object];
        let Some(c) = &om.collision else { continue };
        if om.flags & 0x20 != 0 { continue; }
        for (t, _) in c.triangles_with_material() {
            world.push(W { v: [tx(&inst.matrix, t[0]), tx(&inst.matrix, t[1]), tx(&inst.matrix, t[2])] });
        }
    }
    eprintln!("tris {}", world.len());

    let ray_hit = |w: &W, x: f32, z: f32, y0: f32| -> Option<f32> {
        let (a, b, c) = (w.v[0], w.v[1], w.v[2]);
        let (e1, e2) = ([b[0]-a[0], b[1]-a[1], b[2]-a[2]], [c[0]-a[0], c[1]-a[1], c[2]-a[2]]);
        let pv = [-e2[2], 0.0, e2[0]];
        let det = e1[0]*pv[0] + e1[1]*pv[1] + e1[2]*pv[2];
        if det.abs() < 1e-9 { return None; }
        let inv = 1.0/det;
        let s = [x-a[0], y0-a[1], z-a[2]];
        let u = (s[0]*pv[0] + s[1]*pv[1] + s[2]*pv[2]) * inv;
        if !(0.0..=1.0).contains(&u) { return None; }
        let qv = [s[1]*e1[2]-s[2]*e1[1], s[2]*e1[0]-s[0]*e1[2], s[0]*e1[1]-s[1]*e1[0]];
        let vv = -qv[1]*inv;
        if vv < 0.0 || u+vv > 1.0 { return None; }
        let t = (e2[0]*qv[0] + e2[1]*qv[1] + e2[2]*qv[2]) * inv;
        let h = y0 - t;
        (t >= 0.0 && h >= 0.0).then_some(h)
    };
    let top_below = |x: f32, z: f32, y0: f32| -> Option<(f32, String)> {
        let mut best: Option<(f32, String)> = None;
        for (i, w) in world.iter().enumerate() {
            if let Some(h) = ray_hit(w, x, z, y0) {
                let better = best.as_ref().map(|(bh, _)| h > *bh).unwrap_or(true);
                if better { best = Some((h, format!("tri{i}"))); }
            }
        }
        best
    };

    // ---- MAP 1: the corridor mouth, fine (x −40..−30 step 0.5, z −35..−52 step 0.5), y0 42 ----
    println!("=== MAP 1 corridor mouth (y0 42; . = void) ===");
    let mut h = String::from("       ");
    let mut hx = -40.0f32;
    while hx <= -29.5 { h.push_str(&format!("{hx:>5.0}")); hx += 1.0; }
    println!("{h}");
    let mut z = -35.0f32;
    while z >= -52.0 {
        let mut row = format!("z {z:6.1}: ");
        let mut x = -40.0f32;
        while x <= -29.5 {
            match top_below(x, z, 42.0) {
                Some((f, _)) if f > 18.0 => row.push_str(&format!("{f:>5.1}")),
                _ => row.push_str("    ."),
            }
            x += 1.0;
        }
        println!("{row}");
        z -= 0.5;
    }

    // ---- MAP 1b: same band at 0.5 x-resolution around the mouth x −39..−33, z −35..−47 ----
    println!("=== MAP 1b mouth fine x (step 0.5; y0 42) ===");
    let mut h = String::from("       ");
    let mut hx = -39.0f32;
    while hx <= -32.5 { h.push_str(&format!("{hx:>5.1}")); hx += 0.5; }
    println!("{h}");
    let mut z = -35.5f32;
    while z >= -47.5 {
        let mut row = format!("z {z:6.1}: ");
        let mut x = -39.0f32;
        while x <= -32.5 {
            match top_below(x, z, 42.0) {
                Some((f, _)) if f > 18.0 => row.push_str(&format!("{f:>5.1}")),
                _ => row.push_str("    ."),
            }
            x += 0.5;
        }
        println!("{row}");
        z -= 0.5;
    }

    // ---- MAP 2: gallery + SW loop (x −44..−14 step 1, z −50..−80 step 1; y0 44) ----
    println!("=== MAP 2 gallery/SW loop (y0 44) ===");
    let mut h = String::from("        ");
    let mut hx = -44.0f32;
    while hx <= -13.5 { h.push_str(&format!("{hx:>4.0}")); hx += 1.0; }
    println!("{h}");
    let mut z = -50.0f32;
    while z >= -80.0 {
        let mut row = format!("z {z:6.1}: ");
        let mut x = -44.0f32;
        while x <= -13.5 {
            match top_below(x, z, 44.0) {
                Some((f, _)) if f > 17.0 => row.push_str(&format!("{f:>4.0}")),
                _ => row.push_str("   ."),
            }
            x += 1.0;
        }
        println!("{row}");
        z -= 1.0;
    }

    // ---- MAP 3: battlement + moat + east side (x −20..40 step 1, z −62..−80 step 1; y0 52) ----
    println!("=== MAP 3 battlement/moat/east (y0 52) ===");
    let mut h = String::from("        ");
    let mut hx = -20.0f32;
    while hx <= 40.0 { h.push_str(&format!("{hx:>4.0}")); hx += 1.0; }
    println!("{h}");
    let mut z = -62.0f32;
    while z >= -80.0 {
        let mut row = format!("z {z:6.1}: ");
        let mut x = -20.0f32;
        while x <= 40.0 {
            match top_below(x, z, 52.0) {
                Some((f, _)) if f > 17.0 => row.push_str(&format!("{f:>4.0}")),
                _ => row.push_str("   ."),
            }
            x += 1.0;
        }
        println!("{row}");
        z -= 1.0;
    }

    // ---- MAP 4: the door band + stairs fine (x 4..40 step 1, z −60..−72 step 1; y0 55) ----
    println!("=== MAP 4 stairs/door band (y0 55) ===");
    let mut h = String::from("        ");
    let mut hx = 4.0f32;
    while hx <= 40.0 { h.push_str(&format!("{hx:>4.0}")); hx += 1.0; }
    println!("{h}");
    let mut z = -60.0f32;
    while z >= -72.0 {
        let mut row = format!("z {z:6.1}: ");
        let mut x = 4.0f32;
        while x <= 40.0 {
            match top_below(x, z, 55.0) {
                Some((f, _)) if f > 17.0 => row.push_str(&format!("{f:>4.0}")),
                _ => row.push_str("   ."),
            }
            x += 1.0;
        }
        println!("{row}");
        z -= 1.0;
    }

    // ---- MAP 5: the crack spine fine (x −36.5..−33.0 step 0.25, z −44..−53 step 0.5, y0 42) ----
    println!("=== MAP 5 crack spine fine (y0 42, x step 0.25 from −36.5) ===");
    {
        let mut h = String::from("        ");
        let mut hx = -36.5f32;
        while hx <= -33.0 { h.push_str(&format!("{hx:>5.2}")); hx += 0.25; }
        println!("{h}");
        let mut z = -44.0f32;
        while z >= -53.0 {
            let mut row = format!("z {z:6.1}: ");
            let mut x = -36.5f32;
            while x <= -33.0 {
                match top_below(x, z, 42.0) {
                    Some((f, _)) if f > 18.0 => row.push_str(&format!("{f:>5.1}")),
                    _ => row.push_str("    ."),
                }
                x += 0.25;
            }
            println!("{row}");
            z -= 0.5;
        }
    }

    // ---- MAP 6: the SW loop fine (x −42..−33 step 0.5, z −64..−79 step 0.5, y0 44) ----
    println!("=== MAP 6 SW loop fine (y0 44, x step 0.5 from −42) ===");
    {
        let mut h = String::from("        ");
        let mut hx = -42.0f32;
        while hx <= -33.0 { h.push_str(&format!("{hx:>5.1}")); hx += 0.5; }
        println!("{h}");
        let mut z = -64.0f32;
        while z >= -79.0 {
            let mut row = format!("z {z:6.1}: ");
            let mut x = -42.0f32;
            while x <= -33.0 {
                match top_below(x, z, 44.0) {
                    Some((f, _)) if f > 17.0 => row.push_str(&format!("{f:>5.1}")),
                    _ => row.push_str("    ."),
                }
                x += 0.5;
            }
            println!("{row}");
            z -= 0.5;
        }
    }

    // ---- MAP 7: the moat landing fine (x −7..2 step 0.5, z −70..−75 step 0.5, y0 44) ----
    println!("=== MAP 7 moat landing fine (y0 44, x step 0.5 from −7) ===");
    {
        let mut h = String::from("        ");
        let mut hx = -7.0f32;
        while hx <= 2.0 { h.push_str(&format!("{hx:>5.1}")); hx += 0.5; }
        println!("{h}");
        let mut z = -70.0f32;
        while z >= -75.0 {
            let mut row = format!("z {z:6.1}: ");
            let mut x = -7.0f32;
            while x <= 2.0 {
                match top_below(x, z, 44.0) {
                    Some((f, _)) if f > 17.0 => row.push_str(&format!("{f:>5.1}")),
                    _ => row.push_str("    ."),
                }
                x += 0.5;
            }
            println!("{row}");
            z -= 0.5;
        }
    }

    // ---- MAP 8: the door approach fine (x 26..40 step 0.5, z −75..−62 step 0.5, y0 55) ----
    println!("=== MAP 8 door approach fine (y0 55, x step 0.5 from 26) ===");
    {
        let mut h = String::from("        ");
        let mut hx = 26.0f32;
        while hx <= 40.0 { h.push_str(&format!("{hx:>5.1}")); hx += 0.5; }
        println!("{h}");
        let mut z = -75.0f32;
        while z <= -62.0 {
            let mut row = format!("z {z:6.1}: ");
            let mut x = 26.0f32;
            while x <= 40.0 {
                match top_below(x, z, 55.0) {
                    Some((f, _)) if f > 17.0 => row.push_str(&format!("{f:>5.1}")),
                    _ => row.push_str("    ."),
                }
                x += 0.5;
            }
            println!("{row}");
            z += 0.5;
        }
    }

    // ---- MAP 9: east of the stairs (x 34..50 step 0.5, z −79..−62 step 0.5, y0 55) ----
    println!("=== MAP 9 east stairs wrap (y0 55, x step 0.5 from 34) ===");
    {
        let mut h = String::from("        ");
        let mut hx = 34.0f32;
        while hx <= 50.0 { h.push_str(&format!("{hx:>5.1}")); hx += 0.5; }
        println!("{h}");
        let mut z = -79.0f32;
        while z <= -62.0 {
            let mut row = format!("z {z:6.1}: ");
            let mut x = 34.0f32;
            while x <= 50.0 {
                match top_below(x, z, 55.0) {
                    Some((f, _)) if f > 17.0 => row.push_str(&format!("{f:>5.1}")),
                    _ => row.push_str("    ."),
                }
                x += 0.5;
            }
            println!("{row}");
            z += 0.5;
        }
    }

    // ---- MAP 10: the band's west end + the x-14 void (x 12..24 step 0.25, z −76..−64 step 0.5, y0 55) ----
    println!("=== MAP 10 band west end (y0 55, x step 0.25 from 12) ===");
    {
        let mut h = String::from("        ");
        let mut hx = 12.0f32;
        while hx <= 24.0 { h.push_str(&format!("{hx:>5.2}")); hx += 0.25; }
        println!("{h}");
        let mut z = -76.0f32;
        while z <= -64.0 {
            let mut row = format!("z {z:6.1}: ");
            let mut x = 12.0f32;
            while x <= 24.0 {
                match top_below(x, z, 55.0) {
                    Some((f, _)) if f > 17.0 => row.push_str(&format!("{f:>5.1}")),
                    _ => row.push_str("    ."),
                }
                x += 0.25;
            }
            println!("{row}");
            z += 0.5;
        }
    }

    // ---- MAP 11: surfaces 40..50 around the building (x 34..50 step 0.5, z −80..−62 step 0.5, y0 50) ----
    println!("=== MAP 11 building flank y0 50 (x step 0.5 from 34) ===");
    {
        let mut h = String::from("        ");
        let mut hx = 34.0f32;
        while hx <= 50.0 { h.push_str(&format!("{hx:>5.1}")); hx += 0.5; }
        println!("{h}");
        let mut z = -80.0f32;
        while z <= -62.0 {
            let mut row = format!("z {z:6.1}: ");
            let mut x = 34.0f32;
            while x <= 50.0 {
                match top_below(x, z, 50.0) {
                    Some((f, _)) if f > 17.0 => row.push_str(&format!("{f:>5.1}")),
                    _ => row.push_str("    ."),
                }
                x += 0.5;
            }
            println!("{row}");
            z += 0.5;
        }
    }


    // ---- MAP 12: the band south edge fine z (x 14..38 step 0.25, z −72.0..−70.8 step 0.1, y0 55 then y0 47) ----
    for &y0 in &[55.0f32, 47.0] {
        println!("=== MAP 12 band south edge y0 {y0} (x step 0.25 from 14, z step 0.1) ===");
        let mut h = String::from("        ");
        let mut hx = 14.0f32;
        while hx <= 38.0 { h.push_str(&format!("{hx:>5.1}")); hx += 0.25; }
        println!("{h}");
        let mut z = -72.0f32;
        while z <= -67.0 {
            let mut row = format!("z {z:6.2}: ");
            let mut x = 14.0f32;
            while x <= 38.0 {
                match top_below(x, z, y0) {
                    Some((f, _)) if f > 17.0 => row.push_str(&format!("{f:>5.1}")),
                    _ => row.push_str("    ."),
                }
                x += 0.25;
            }
            println!("{row}");
            z += 0.1;
        }
    }

    // ---- placements in the route region (z < −35, any x) ----
    println!("=== placements z < −35 ===");
    let mm = level.main_map().expect("main map");
    for (i, p) in level.maps[mm].placements.iter().enumerate() {
        let (x, y, z) = (p.translation[0], p.translation[1], p.translation[2]);
        if z < -35.0 {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < level.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
            let name = target.map(|(m, o)| level.maps[m].objects[o].name.trim().to_string())
                .unwrap_or_else(|| format!("UNRESOLVED {:08x}", p.hashcode));
            let mut ps = String::new();
            for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
            println!("#{i} kind {:#04x} \"{}\" @({:.2},{:.2},{:.2}) rot({:.3},{:.3},{:.3}){ps}",
                p.flags & 0xff, name, x, y, z, p.rotation[0], p.rotation[1], p.rotation[2]);
        }
    }
}
