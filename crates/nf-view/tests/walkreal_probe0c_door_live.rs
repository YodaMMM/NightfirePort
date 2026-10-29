//! walkreal probe (level-0c-door2turbine): door2turbine #281's E-path — the door's parsed
//! fields, the use-sphere test from the SE-corner stance, who writes ch 5 (the door's
//! ch_open), and every E pulse's effect on the door state. Also plate #274's cell blockers.
//! Diagnostic only — run with --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_door2turbine_e_path() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== door census (all doors) ===");
        for (i, d) in sim.drones.doors.list.iter().enumerate() {
            eprintln!("door[{i}] placement={} at=({:.2},{:.2},{:.2}) kind={} ch_open={:#x} ch_lock={:#x} lock_flag={} approach={} state={:#x} flags={:#x} glist={:#x} sph=({:.2},{:.2},{:.2}) r={:.2}",
                d.placement, d.pos[0], d.pos[1], d.pos[2], d.kind, d.ch_open, d.ch_lock, d.locked_flag, d.approach, d.state, d.flags, d.glist,
                d.sph.0[0], d.sph.0[1], d.sph.0[2], d.sph.1);
        }
        eprintln!("=== who writes ch 5 ===");
        for (i, t) in sim.drones.triggers.iter().enumerate() {
            if t.out == 5 || t.list.contains(&5) || t.list.contains(&0x5) {
                let c = t.centre();
                eprintln!("trig[{i}] placement={} kind={:#x} out={:#04x} gate={} key={} mask={:#04x} at=({:.2},{:.2},{:.2})",
                    t.placement, t.kind, t.out, t.gate, t.key_gate, t.mask, c[0], c[1], c[2]);
            }
        }
        for (i, s) in sim.drones.sss.list.iter().enumerate() {
            if s.channel == 5 {
                eprintln!("ss[{i}] placement={} ch={:#04x} state={:#x} mask={:#x} at=({:.2},{:.2},{:.2})",
                    s.placement, s.channel, s.state, s.mask, s.matrix[3][0], s.matrix[3][1], s.matrix[3][2]);
            }
        }
    }
    // Probe-only teleport to the SE-corner stance (the walkreal leg 9's walk_to destination),
    // then the REAL E press through the walker's Use act.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [40.5, 11.5, -211.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = &sim.bond;
        eprintln!("=== at the door: bond=({:.2},{:.2},{:.2}) yaw={:.3} grounded={}", b.pos[0], b.pos[1], b.pos[2], b.yaw, b.grounded);
        let door = sim.drones.doors.list.iter().enumerate()
            .find(|(_, d)| (d.pos[0] - 40.0).abs() < 0.5 && (d.pos[2] + 210.4).abs() < 0.5);
        match door {
            Some((i, d)) => {
                eprintln!("door[{i}] kind={} ch_open={:#x} ch_lock={:#x} lock_flag={} approach={} state={:#x} flags={:#x} glist={:#x} pos01={:.3} timer={}",
                    d.kind, d.ch_open, d.ch_lock, d.locked_flag, d.approach, d.state, d.flags, d.glist, d.pos01, d.timer);
                // The use-sphere test exactly as player_use runs it: use_point = head + look
                // (head = pos.y + eye), the door's live world sphere.
                let head = [b.pos[0], b.pos[1] + b.eye - b.crouch_drop, b.pos[2]];
                let look = [b.yaw.sin(), 0.0, b.yaw.cos()]; // [S] unverified: the port's forward convention
                let up = [head[0] + look[0], head[1] + look[1], head[2] + look[2]];
                let (c, r) = d.live_sph();
                let sx = up[0] - c[0]; let sy = up[1] - c[1]; let sz = up[2] - c[2];
                let reach = 1.0 + r;
                eprintln!("use_point=({:.2},{:.2},{:.2}) live_sph=({:.2},{:.2},{:.2}) r={:.2} sqdist={:.3} reach^2={:.3} HIT={}",
                    up[0], up[1], up[2], c[0], c[1], c[2], r,
                    sx * sx + sy * sy + sz * sz, reach * reach,
                    sx * sx + sy * sy + sz * sz < reach * reach);
                eprintln!("ch5 on? {}", sim.drones.channels.on(5));
            }
            None => eprintln!("door2turbine NOT FOUND at (40,*,−210.4)"),
        }
    }
    // Manual Use-act replica with per-pulse diagnostics: settle, face the door, pulse E
    // every 6 ticks, print the sphere test each pulse, stop when the door moves.
    for _ in 0..100 { w.tick(nf_game::player::Input::default(), false); }
    let mut opened = false;
    for t in 0..66u32 {
        let press = t % 6 == 0;
        if press {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let head = [b.pos[0], b.pos[1] + b.eye - b.crouch_drop, b.pos[2]];
            let look = b.look();
            let up = [head[0] + look[0], head[1] + look[1], head[2] + look[2]];
            let d = sim.drones.doors.list.iter().find(|d| (d.pos[0] - 40.0).abs() < 0.5 && (d.pos[2] + 210.4).abs() < 0.5).unwrap();
            let (c, r) = d.live_sph();
            let sx = up[0] - c[0]; let sy = up[1] - c[1]; let sz = up[2] - c[2];
            let sq = sx * sx + sy * sy + sz * sz;
            let reach2 = (1.0 + r) * (1.0 + r);
            eprintln!("t={t} yaw={:.3} grounded={} pos=({:.2},{:.2},{:.2}) use=({:.2},{:.2},{:.2}) sq={sq:.3} reach2={reach2:.3} hit={}",
                b.yaw, b.grounded, b.pos[0], b.pos[1], b.pos[2], up[0], up[1], up[2], sq < reach2);
        }
        let turn = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let desired = (40.0f32 - b.pos[0]).atan2(-210.4f32 - b.pos[2]);
            walker::wrap_pi(desired - b.yaw).clamp(-0.08, 0.08)
        };
        w.tick(nf_game::player::Input { turn, ..Default::default() }, press);
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(d) = sim.drones.doors.list.iter().find(|d| (d.pos[0] - 40.0).abs() < 0.5 && (d.pos[2] + 210.4).abs() < 0.5) {
            if d.state != 2 { opened = true; eprintln!("DOOR MOVED at t={t}: state={:#x} pos01={:.3} flags={:#x}", d.state, d.pos01, d.flags); break; }
        }
    }
    if !opened { eprintln!("the door never moved across 66 ticks"); }
    // The floor map around door #281: where does the port's collision actually put floors?
    // The [D] route: the SE walkway 12.0 (z −214..−222), the SE corner 11.1 (36-40, −208..−210),
    // the band 10.3 (x 46..96). Probe every 2 units.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== floor map around door #281 (ray y 14 -> y 8) ===");
        for z in [-222.0f32, -220.0, -218.0, -216.0, -214.0, -213.0, -212.0, -211.0, -210.0, -209.0, -208.0, -207.0] {
            let mut row = format!("z {z:7.1}: ");
            for x in [36.0f32, 38.0, 40.0, 42.0, 44.0, 46.0] {
                let hit = sim.world.segment_hit([x, 14.0, z], [x, 8.0, z]);
                match hit {
                    Some((t, _)) => { let y = 14.0 - 6.0 * t; row.push_str(&format!("x{x}={y:5.2}  ")); }
                    None => row.push_str(&format!("x{x}=  --  ", )),
                }
            }
            eprintln!("{row}");
        }
        eprintln!("=== door slab region fine map (x 39..42, z −212..−208 step 1) ===");
        for z in [-212.0f32, -211.0, -210.0, -209.0, -208.0] {
            let mut row = format!("z {z:7.1}: ");
            for x in [39.0f32, 40.0, 40.5, 41.0, 42.0] {
                let hit = sim.world.segment_hit([x, 14.0, z], [x, 8.0, z]);
                match hit {
                    Some((t, _)) => { let y = 14.0 - 6.0 * t; row.push_str(&format!("x{x}={y:5.2}  ")); }
                    None => row.push_str(&format!("x{x}=  --  ")),
                }
            }
            eprintln!("{row}");
        }
    }
    // The raw placement census around the door strip: who owns the threshold floor?
    {
        use nf_formats::{disc, iso9660::IsoImage, level::Level, placement::view_rot_trans_matrix};
        let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
        let img = IsoImage::open(&iso_path).unwrap();
        let (entries, bin) = disc::files_bin_entries(&img).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).unwrap();
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let m = level.main_map().unwrap();
        eprintln!("=== raw placements near (40, *, −212) r 9 ===");
        for (pi, p) in level.maps[m].placements.iter().enumerate() {
            let t = p.translation;
            if (t[0] - 40.0).abs() > 9.0 || (t[2] + 212.0).abs() > 9.0 { continue; }
            let obj = if p.by_hash() {
                match level.by_hash.get(&p.hashcode).map(|&(mm, oo)| &level.maps[mm].objects[oo]) {
                    Some(o) => o, None => { eprintln!("plc[{pi}] at=({:.2},{:.2},{:.2}) flags={:#x} hash={:#x} UNRESOLVED", t[0], t[1], t[2], p.flags, p.hashcode); continue; }
                }
            } else {
                match level.maps[m].objects.get(p.index as usize) {
                    Some(o) => o,
                    None => { eprintln!("plc[{pi}] at=({:.2},{:.2},{:.2}) flags={:#x} idx={} UNRESOLVED", t[0], t[1], t[2], p.flags, p.index); continue; }
                }
            };
            let tris = obj.collision.as_ref().map(|c| c.triangles()).unwrap_or_default();
            eprintln!("plc[{pi}] \"{}\" at=({:.2},{:.2},{:.2}) flags={:#x} hash={:#x} coll_tris={}", obj.name.trim(), t[0], t[1], t[2], p.flags, p.hashcode, tris.len());
            // Any collision tri inside the threshold strip (x 37..43, z −214..−210.3, y 9..13)?
            let mat = view_rot_trans_matrix(p.rotation, p.translation);
            let mut strip = 0usize;
            let mut ymin = f32::MAX;
            for tri in &tris {
                let w = tri.map(|v| [mat[0][0] * v[0] + mat[1][0] * v[1] + mat[2][0] * v[2] + mat[3][0],
                                     mat[0][1] * v[0] + mat[1][1] * v[1] + mat[2][1] * v[2] + mat[3][1],
                                     mat[0][2] * v[0] + mat[1][2] * v[1] + mat[2][2] * v[2] + mat[3][2]]);
                if w.iter().all(|v| v[0] >= 37.0 && v[0] <= 43.0 && v[2] >= -214.0 && v[2] <= -210.3 && v[1] >= 9.0 && v[1] <= 13.0) {
                    strip += 1;
                    ymin = ymin.min(w.iter().map(|v| v[1]).fold(f32::MAX, f32::min));
                }
            }
            if strip > 0 { eprintln!("       ^ {} collision tris INSIDE the threshold strip (min y {ymin:.2})", strip); }
        }
    }
    // The FULL placement scan: every placement whose transformed collision has tris in
    // (a) the door threshold strip, (b) the plate #274 cell. This settles who owns the floors.
    {
        use nf_formats::{disc, iso9660::IsoImage, level::Level, placement::view_rot_trans_matrix};
        let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
        let img = IsoImage::open(&iso_path).unwrap();
        let (entries, bin) = disc::files_bin_entries(&img).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).unwrap();
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let m = level.main_map().unwrap();
        let in_strip = |v: &[f32; 3]| v[0] >= 37.0 && v[0] <= 43.0 && v[2] >= -214.5 && v[2] <= -210.3 && v[1] >= 9.0 && v[1] <= 13.0;
        let in_plate = |v: &[f32; 3]| v[0] >= 4.0 && v[0] <= 8.0 && v[2] >= -112.5 && v[2] <= -109.5 && v[1] >= 12.0 && v[1] <= 15.5;
        let near_door_any = |v: &[f32; 3]| v[0] >= 36.0 && v[0] <= 46.0 && v[2] >= -216.0 && v[2] <= -206.0 && v[1] >= 8.0 && v[1] <= 15.0;
        eprintln!("=== FULL placement scan: threshold strip + plate #274 cell owners ===");
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
            let (mut s_tris, mut p_tris) = (0usize, 0usize);
            let mut sy = f32::MAX; let mut pymin = f32::MAX; let mut pymax = f32::MIN;
            for tri in &tris {
                let w = tri.map(|v| [mat[0][0] * v[0] + mat[1][0] * v[1] + mat[2][0] * v[2] + mat[3][0],
                                     mat[0][1] * v[0] + mat[1][1] * v[1] + mat[2][1] * v[2] + mat[3][1],
                                     mat[0][2] * v[0] + mat[1][2] * v[1] + mat[2][2] * v[2] + mat[3][2]]);
                if w.iter().all(|v| in_strip(v)) { s_tris += 1; sy = sy.min(w.iter().map(|v| v[1]).fold(f32::MAX, f32::min)); }
                if w.iter().all(|v| in_plate(v)) { p_tris += 1; pymin = pymin.min(w.iter().map(|v| v[1]).fold(f32::MAX, f32::min)); pymax = pymax.max(w.iter().map(|v| v[1]).fold(f32::MIN, f32::max)); }
            }
            if s_tris > 0 || p_tris > 0 {
                eprintln!("plc[{pi}] \"{}\" at=({:.2},{:.2},{:.2}) flags={:#x} hash={:#x}: strip={s_tris} (min y {sy:.2}) plate_cell={p_tris} (y {pymin:.2}..{pymax:.2})",
                    obj.name.trim(), p.translation[0], p.translation[1], p.translation[2], p.flags, p.hashcode);
            }
            // plc 8 (room_turbine01) + plc 281 (the door): any-vert tris near the door region —
            // the big-floor catch the all-verts strip test missed.
            if pi == 281 || pi == 8 {
                let mut any_cnt = 0usize;
                for tri in tris.iter() {
                    let w = tri.map(|v| [mat[0][0] * v[0] + mat[1][0] * v[1] + mat[2][0] * v[2] + mat[3][0],
                                         mat[0][1] * v[0] + mat[1][1] * v[1] + mat[2][1] * v[2] + mat[3][1],
                                         mat[0][2] * v[0] + mat[1][2] * v[1] + mat[2][2] * v[2] + mat[3][2]]);
                    if !w.iter().any(|v| near_door_any(v)) { continue; }
                    any_cnt += 1;
                    let min = [w[0][0].min(w[1][0]).min(w[2][0]), w[0][1].min(w[1][1]).min(w[2][1]), w[0][2].min(w[1][2]).min(w[2][2])];
                    let max = [w[0][0].max(w[1][0]).max(w[2][0]), w[0][1].max(w[1][1]).max(w[2][1]), w[0][2].max(w[1][2]).max(w[2][2])];
                    eprintln!("plc{pi} tri x {:.2}..{:.2}  y {:.2}..{:.2}  z {:.2}..{:.2}", min[0], max[0], min[1], max[1], min[2], max[2]);
                }
                eprintln!("plc[{pi}] tris with any vert near the door region: {any_cnt} of {}", tris.len());
            }
        }
        eprintln!("=== scan done ===");
    }
    // The FINE ledge map: x 40.2..41.7 step 0.3, z −214.5..−209.5 step 0.5, ray y 13 -> 6.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== fine ledge map ===");
        for z in [-214.5f32, -214.0, -213.5, -213.0, -212.5, -212.0, -211.5, -211.0, -210.5, -210.0, -209.5] {
            let mut row = format!("z {z:7.1}: ");
            let mut x = 40.2f32;
            while x <= 41.71 {
                let hit = sim.world.segment_hit([x, 13.0, z], [x, 6.0, z]);
                match hit {
                    Some((t, _)) => { let y = 13.0 - 7.0 * t; row.push_str(&format!("{y:5.2} ")); }
                    None => row.push_str("  --  "),
                }
                x += 0.3;
            }
            eprintln!("{row}");
        }
        eprintln!("(x cols: 40.2 40.5 40.8 41.1 41.4 41.7)");
    }
    // The approach gap: x 36.8..40.3, z −214.5..−210.5 — how the corner connects south.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== approach gap map (x 36.8..40.3) ===");
        for z in [-214.5f32, -214.0, -213.5, -213.0, -212.5, -212.0, -211.5, -211.0, -210.5] {
            let mut row = format!("z {z:7.1}: ");
            let mut x = 36.8f32;
            while x <= 40.31 {
                let hit = sim.world.segment_hit([x, 13.0, z], [x, 6.0, z]);
                match hit {
                    Some((t, _)) => { let y = 13.0 - 7.0 * t; row.push_str(&format!("{y:5.2} ")); }
                    None => row.push_str("  --  "),
                }
                x += 0.5;
            }
            eprintln!("{row}");
        }
        eprintln!("(x cols: 36.8 37.3 37.8 38.3 38.8 39.3 39.8 40.3)");
    }
    // The parapet's west face: east->west rays at y 12.5, find the gaps.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== parapet west face rays (y 12.5, x 40.6 -> 38.0) ===");
        let mut z = -218.0f32;
        while z <= -213.9 {
            let hit = sim.world.segment_hit([40.6, 12.5, z], [38.0, 12.5, z]);
            let s = match hit { Some((t, _)) => format!("hit t={t:.2} x={:.2}", 40.6 - 2.6 * t), None => "clear".to_string() };
            eprintln!("z {z:7.1}: {s}");
            z += 0.5;
        }
        eprintln!("=== parapet top map (x 40.2..41.4, z −218..−213.5, ray 14->11) ===");
        for z in [-218.0f32, -217.0, -216.0, -215.0, -214.5, -214.0] {
            let mut row = format!("z {z:7.1}: ");
            let mut x = 40.2f32;
            while x <= 41.41 {
                let hit = sim.world.segment_hit([x, 14.0, z], [x, 11.0, z]);
                match hit {
                    Some((t, _)) => { let y = 14.0 - 3.0 * t; row.push_str(&format!("{y:5.2} ")); }
                    None => row.push_str("  --  "),
                }
                x += 0.3;
            }
            eprintln!("{row}");
        }
        eprintln!("(x cols: 40.2 40.5 40.8 41.1 41.4)");
    }
    // The plate #274 blockers: what stands in the cell (5.79, 14.1, -111.1)?
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== plate #274 cell ===");
        for (label, a, b_pt) in [
            ("down", [5.79f32, 15.5, -111.1], [5.79, 12.0, -111.1]),
            ("east->", [8.0, 14.1, -111.1], [5.0, 14.1, -111.1]),
            ("west->", [3.0, 14.1, -111.1], [6.0, 14.1, -111.1]),
            ("north->", [5.79, 14.1, -114.0], [5.79, 14.1, -110.0]),
            ("south->", [5.79, 14.1, -108.0], [5.79, 14.1, -112.0]),
        ] {
            let hit = sim.world.segment_hit(a, b_pt);
            eprintln!("{} {:?} -> {:?} {:?}", label, a, b_pt, hit.map(|(t, n)| (t, n)));
        }
    }
}
