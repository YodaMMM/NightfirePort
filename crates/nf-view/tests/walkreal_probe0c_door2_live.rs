//! walkreal probe #2 (level-0c-door2turbine-2): the SE walkway's west parapet and the shelf
//! connection — the parapet's top height and x/z extent, the west-face height profile, who
//! owns the parapet tris (placement attribution, y window widened past the old scan's 13.0
//! cap), the shelf → corridor junction fine map, the dynamic-object check at the balance
//! spot, and the slot-jump map onto the gate band. Everything [D] from the disc's collision
//! tris + the port's built world. Diagnostic only — run with --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn down(sim: &nf_view::scene::Sim, x: f32, y0: f32, y1: f32, z: f32) -> Option<f32> {
    sim.world.segment_hit([x, y0, z], [x, y1, z]).map(|(t, _)| y0 - (y0 - y1) * t)
}

#[test]
fn probe_walkway_shelf_connection() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);

    // 1. The parapet top profile: down rays over the walkway's west edge and east of it.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== parapet top profile (down rays y 15 -> 11) ===");
        let xs = [39.85f32, 39.9, 39.95, 40.0, 40.05, 40.1, 40.2, 40.3, 40.45, 40.6];
        let zs = [-218.0f32, -217.0, -216.0, -215.0, -214.5, -214.2, -214.0, -213.8, -213.6];
        for z in zs {
            let mut row = format!("z {z:7.1}: ");
            for x in xs {
                match down(sim, x, 15.0, 11.0, z) {
                    Some(y) => row.push_str(&format!("{x}:{y:5.2}  ")),
                    None => row.push_str(&format!("{x}:  --  ")),
                }
            }
            eprintln!("{row}");
        }
        eprintln!("(x cols: 39.85 39.9 39.95 40.0 40.05 40.1 40.2 40.3 40.45 40.6)");
    }

    // 2. The west-face height profile: how high does the wall at x 39.99 go, per z — and is
    //    there a z-gap anywhere along the walkway's west edge?
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== west-face height profile (rays x 40.6 -> 38.0 at each y) ===");
        let ys = [12.1f32, 12.3, 12.5, 12.7, 12.9, 13.05, 13.2, 13.4, 13.6, 13.9, 14.25];
        let zs = [-218.0f32, -217.5, -217.0, -216.5, -216.0, -215.5, -215.0, -214.5, -214.2, -214.0, -213.8, -213.6];
        for z in zs {
            let mut row = format!("z {z:7.1}: ");
            for y in ys {
                match sim.world.segment_hit([40.6, y, z], [38.0, y, z]) {
                    Some((t, _)) => { let x = 40.6 - 2.6 * t; row.push_str(&format!("y{y}:x{x:5.2}  ")); }
                    None => row.push_str(&format!("y{y}:  --  ")),
                }
            }
            eprintln!("{row}");
        }
        eprintln!("(cols: y 12.1 12.3 12.5 12.7 12.9 13.05 13.2 13.4 13.6 13.9 14.25)");
    }

    // 3. Attribution: EVERY placement whose transformed collision owns tris over the parapet
    //    block (x 39.0..41.0, y 11.5..15.0, z −219.5..−213.0) — the old scan's y window
    //    capped at 13.0, which a 13.05 surface would slip.
    {
        use nf_formats::{disc, iso9660::IsoImage, level::Level, placement::view_rot_trans_matrix};
        let img = IsoImage::open(&iso).unwrap();
        let (entries, bin) = disc::files_bin_entries(&img).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).unwrap();
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let m = level.main_map().unwrap();
        let in_block = |v: &[f32; 3]| v[0] >= 39.0 && v[0] <= 41.0 && v[1] >= 11.5 && v[1] <= 15.0 && v[2] >= -219.5 && v[2] <= -213.0;
        eprintln!("=== placement attribution, parapet block (x 39..41, y 11.5..15, z −219.5..−213) ===");
        for (pi, p) in level.maps[m].placements.iter().enumerate() {
            let obj = if p.by_hash() {
                match level.by_hash.get(&p.hashcode) { Some(&(mm, oo)) => &level.maps[mm].objects[oo], None => continue }
            } else {
                match level.maps[m].objects.get(p.index as usize) { Some(o) => o, None => continue }
            };
            let Some(coll) = obj.collision.as_ref() else { continue };
            let tris = coll.triangles();
            if tris.is_empty() { continue; }
            let mat = view_rot_trans_matrix(p.rotation, p.translation);
            let (mut n, mut ymin, mut ymax) = (0usize, f32::MAX, f32::MIN);
            let (mut xmin, mut xmax, mut zmin, mut zmax) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
            for tri in &tris {
                let wv = tri.map(|v| [mat[0][0] * v[0] + mat[1][0] * v[1] + mat[2][0] * v[2] + mat[3][0],
                                      mat[0][1] * v[0] + mat[1][1] * v[1] + mat[2][1] * v[2] + mat[3][1],
                                      mat[0][2] * v[0] + mat[1][2] * v[1] + mat[2][2] * v[2] + mat[3][2]]);
                if wv.iter().all(|v| in_block(v)) {
                    n += 1;
                    for v in &wv {
                        ymin = ymin.min(v[1]); ymax = ymax.max(v[1]);
                        xmin = xmin.min(v[0]); xmax = xmax.max(v[0]);
                        zmin = zmin.min(v[2]); zmax = zmax.max(v[2]);
                    }
                }
            }
            if n > 0 {
                eprintln!("plc[{pi}] \"{}\" at=({:.2},{:.2},{:.2}) flags={:#x}: {n} tris in block, x {xmin:.2}..{xmax:.2} y {ymin:.2}..{ymax:.2} z {zmin:.2}..{zmax:.2}",
                    obj.name.trim(), p.translation[0], p.translation[1], p.translation[2], p.flags);
            }
        }
        eprintln!("=== scan done ===");
    }

    // 4. The shelf → corridor junction fine map (the 0.5-gap at z −213.0..−212.5) and the
    //    walkway→shelf columns at the junction rows.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== shelf/corridor fine map (down rays y 13 -> 9) ===");
        let xs = [37.8f32, 38.05, 38.3, 38.55, 38.8, 39.05, 39.3, 39.55, 39.8, 40.05];
        let zs = [-214.5f32, -214.0, -213.5, -213.25, -213.0, -212.75, -212.5, -212.25, -212.0];
        for z in zs {
            let mut row = format!("z {z:7.2}: ");
            for x in xs {
                match down(sim, x, 13.0, 9.0, z) {
                    Some(y) => row.push_str(&format!("{y:5.2} ")),
                    None => row.push_str("   -- "),
                }
            }
            eprintln!("{row}");
        }
        eprintln!("(x cols: 37.8 38.05 38.3 38.55 38.8 39.05 39.3 39.55 39.8 40.05)");
    }

    // 5. The dynamic-object check: stand Bond mid-walkway, walk north on real input, and at
    //    each sample dump every drone within r 2.5 (the "drone under Bond" hypothesis) plus
    //    the floor directly beneath.
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [40.2, 12.6, -216.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(20);
        for sample in 0..6u32 {
            for _ in 0..30 {
                let turn = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let b = &sim.bond;
                    let desired = (40.2f32 - b.pos[0]).atan2(-211.5f32 - b.pos[2]);
                    walker::wrap_pi(desired - b.yaw).clamp(-0.08, 0.08)
                };
                w.tick(nf_game::player::Input { turn, forward: 1.0, ..Default::default() }, false);
            }
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let under = down(sim, b.pos[0], b.pos[1], b.pos[1] - 3.0, b.pos[2]);
            let mut near = String::new();
            for d in sim.drones.list.iter() {
                let dx = d.pos[0] - b.pos[0]; let dy = d.pos[1] - b.pos[1]; let dz = d.pos[2] - b.pos[2];
                if dx * dx + dy * dy + dz * dz < 6.25 {
                    near.push_str(&format!(" [skin={:#x} at=({:.2},{:.2},{:.2})]", d.skin, d.pos[0], d.pos[1], d.pos[2]));
                }
            }
            eprintln!("sample {sample}: bond=({:.2},{:.2},{:.2}) grounded={} under={under:?} near:{near}",
                b.pos[0], b.pos[1], b.pos[2], b.grounded);
        }
    }

    // 6. The slot-jump map: the ring SE corner's east edge, the x 41..43 slot, the band resume.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== slot map (down rays y 13 -> 9) ===");
        let xs = [40.2f32, 40.5, 40.8, 41.0, 41.4, 41.8, 42.2, 42.6, 43.0, 43.4, 43.8, 44.2, 44.6];
        let zs = [-210.0f32, -209.5, -209.0, -208.5, -208.0, -207.5];
        for z in zs {
            let mut row = format!("z {z:7.1}: ");
            for x in xs {
                match down(sim, x, 13.0, 9.0, z) {
                    Some(y) => row.push_str(&format!("{y:5.2} ")),
                    None => row.push_str("   -- "),
                }
            }
            eprintln!("{row}");
        }
        eprintln!("(x cols: 40.2 40.5 40.8 41.0 41.4 41.8 42.2 42.6 43.0 43.4 43.8 44.2 44.6)");
    }

    // 7. The south-west channel map: the strip WEST of the wall (x 37.8..39.7) and SOUTH of the
    //    shelf (z −222..−214.5) — #504's sound trigger sits at (39.15, 13.46, −218.74) [D audit],
    //    so a floor may exist there that the old maps never probed.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== south-west channel map (down rays y 15 -> 9) ===");
        let xs = [37.8f32, 38.05, 38.3, 38.55, 38.8, 39.05, 39.3, 39.55, 39.8, 40.05, 40.3];
        let zs = [-222.0f32, -220.5, -219.5, -219.0, -218.5, -218.0, -217.5, -217.0, -216.5, -216.0, -215.5, -215.0, -214.5];
        for z in zs {
            let mut row = format!("z {z:7.1}: ");
            for x in xs {
                match down(sim, x, 15.0, 9.0, z) {
                    Some(y) => row.push_str(&format!("{y:5.2} ")),
                    None => row.push_str("   -- "),
                }
            }
            eprintln!("{row}");
        }
        eprintln!("(x cols: 37.8 38.05 38.3 38.55 38.8 39.05 39.3 39.55 39.8 40.05 40.3)");
    }

    // 8. The east-wall profile: does the slot open EAST anywhere along its length?
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== east-wall profile (rays x 40.0 -> 41.6 at each y) ===");
        let ys = [12.1f32, 12.5, 13.0, 13.5, 14.0, 14.4];
        let zs = [-218.5f32, -218.0, -217.0, -216.0, -215.0, -214.5, -214.0, -213.8];
        for z in zs {
            let mut row = format!("z {z:7.1}: ");
            for y in ys {
                match sim.world.segment_hit([40.0, y, z], [41.6, y, z]) {
                    Some((t, _)) => { let x = 40.0 + 1.6 * t; row.push_str(&format!("y{y}:x{x:5.2}  ")); }
                    None => row.push_str(&format!("y{y}:  --  ")),
                }
            }
            eprintln!("{row}");
        }
        eprintln!("(cols: y 12.1 12.5 13.0 13.5 14.0 14.4)");
        // And north of the end wall at the slot's x: is anything standing in z −213.5..−211?
        eprintln!("=== north rays at x 40.3 (z −214.5 -> −211.5) ===");
        for y in [12.1f32, 12.5, 13.0, 13.5, 14.0, 14.4] {
            match sim.world.segment_hit([40.3, y, -214.5], [40.3, y, -211.5]) {
                Some((t, _)) => eprintln!("y{y}: hit at z={:.2}", -214.5 + 3.0 * t),
                None => eprintln!("y{y}: free flight to z −211.5"),
            }
        }
    }

    // 9. The decisive tri dump: every tri of room_turbine01 (plc 8) and room_ex10 (plc 116)
    //    with any vertex in the district (x 37..42, y 10..16, z −220..−207), bbox per tri.
    {
        use nf_formats::{disc, iso9660::IsoImage, level::Level, placement::view_rot_trans_matrix};
        let img = IsoImage::open(&iso).unwrap();
        let (entries, bin) = disc::files_bin_entries(&img).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).unwrap();
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let m = level.main_map().unwrap();
        let district = |v: &[f32; 3]| v[0] >= 37.0 && v[0] <= 42.0 && v[1] >= 10.0 && v[1] <= 16.0 && v[2] >= -220.0 && v[2] <= -207.0;
        for (pi, p) in level.maps[m].placements.iter().enumerate() {
            if pi != 8 && pi != 116 { continue; }
            let obj = if p.by_hash() {
                match level.by_hash.get(&p.hashcode) { Some(&(mm, oo)) => &level.maps[mm].objects[oo], None => continue }
            } else {
                match level.maps[m].objects.get(p.index as usize) { Some(o) => o, None => continue }
            };
            let Some(coll) = obj.collision.as_ref() else { continue };
            let tris = coll.triangles();
            let mat = view_rot_trans_matrix(p.rotation, p.translation);
            eprintln!("=== plc[{pi}] \"{}\" district tris ===", obj.name.trim());
            for (ti, tri) in tris.iter().enumerate() {
                let wv = tri.map(|v| [mat[0][0] * v[0] + mat[1][0] * v[1] + mat[2][0] * v[2] + mat[3][0],
                                      mat[0][1] * v[0] + mat[1][1] * v[1] + mat[2][1] * v[2] + mat[3][1],
                                      mat[0][2] * v[0] + mat[1][2] * v[1] + mat[2][2] * v[2] + mat[3][2]]);
                if wv.iter().any(district) {
                    let (mut x0, mut x1) = (f32::MAX, f32::MIN);
                    let (mut y0, mut y1) = (f32::MAX, f32::MIN);
                    let (mut z0, mut z1) = (f32::MAX, f32::MIN);
                    for v in &wv {
                        x0 = x0.min(v[0]); x1 = x1.max(v[0]);
                        y0 = y0.min(v[1]); y1 = y1.max(v[1]);
                        z0 = z0.min(v[2]); z1 = z1.max(v[2]);
                    }
                    eprintln!("tri[{ti}] x {x0:7.2}..{x1:7.2}  y {y0:6.2}..{y1:6.2}  z {z0:8.2}..{z1:8.2}");
                }
            }
        }
    }
    // 10. The band west section (x 44..70): where does the floor actually run? Bond fell at
    //     x ~60-63 walking z −208.4 eastward.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== band west map (down rays y 12 -> 8) ===");
        let xs = [44.0f32, 45.0, 46.0, 47.5, 49.0, 51.0, 53.0, 55.0, 57.0, 59.0, 61.0, 63.0, 65.0, 67.0, 69.0];
        let zs = [-209.5f32, -209.0, -208.5, -208.0, -207.5, -207.0, -206.5, -206.0];
        for z in zs {
            let mut row = format!("z {z:7.1}: ");
            for x in xs {
                match down(sim, x, 12.0, 8.0, z) {
                    Some(y) => row.push_str(&format!("{y:5.2} ")),
                    None => row.push_str("   -- "),
                }
            }
            eprintln!("{row}");
        }
        eprintln!("(x cols: 44 45 46 47.5 49 51 53 55 57 59 61 63 65 67 69)");
    }
    eprintln!("ok");
}
