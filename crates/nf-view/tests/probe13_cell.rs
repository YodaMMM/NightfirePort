//! level-13 cell-stickiness probe (716c): the reach-only teleport to the west-wall shutter panel
//! leaves the port's scoped collision on the wrong cel — every later walk leg falls (716b's
//! fall13_probe). Dumps bond.cel / y / the scope buckets at each step. Diagnostic — run with
//! `--ignored --nocapture`.
#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_cell_stickiness_after_panel_teleport() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = Walker::new(nf_view::load(&iso, "07000013.bin", false).expect("load 07000013"));
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[probe] spawn pos ({:.2},{:.2},{:.2}) cel {:?} grounded {}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.cel, sim.bond.grounded);
        let panel = [-19.49f32, -369.1, 23.9];
        let entrance = [-6.2f32, -368.0, 66.3];
        let world = &sim.world;
        eprintln!("[probe] find_cel(panel) {:?}  find_cel(spawn) {:?}  find_cel(entrance) {:?}",
            world.find_cel(panel), world.find_cel(sim.bond.pos), world.find_cel(entrance));
        eprintln!("[probe] buckets@panel {:?} buckets@spawn {:?}",
            world.debug_bucket_owners([-20.5, -371.0, 23.0], [-18.5, -368.0, 25.0]),
            world.debug_bucket_owners([1.0, -370.0, 10.0], [3.0, -367.0, 12.0]));
        // The scope graph around cells 21 and 28: bboxes, room flags, the portals touching
        // either, and the straddle set a capsule at the panel keeps from cell 21.
        let g = &sim.cel_graph;
        for c in [21usize, 28] {
            let n = &g.cells[c];
            eprintln!("[probe] cell {c}: bbox ({:.2},{:.2},{:.2})..({:.2},{:.2},{:.2}) flags {:#x} room {:?}",
                n.min[0], n.min[1], n.min[2], n.max[0], n.max[1], n.max[2], n.flags,
                g.room_to_cell.iter().position(|&r| r == Some(c)));
        }
        for (pi, p) in g.portals.iter().enumerate() {
            if p.cel_a == 21 || p.cel_b == 21 || p.cel_a == 28 || p.cel_b == 28 {
                eprintln!("[probe] portal {pi}: {}↔{} corners ({:.1},{:.1},{:.1})..({:.1},{:.1},{:.1})",
                    p.cel_a, p.cel_b,
                    p.corners[0][0], p.corners[0][1], p.corners[0][2],
                    p.corners[2][0], p.corners[2][1], p.corners[2][2]);
            }
        }
        eprintln!("[probe] straddle(21, panel band r0.55) {:?}  straddle(21, spawn band) {:?}",
            world.debug_straddle(21, [-19.49, -369.1, 23.9], [-19.49, -368.1, 23.9], 0.55),
            world.debug_straddle(21, [2.16, -369.1, 10.96], [2.16, -368.1, 10.96], 0.55));
        // The west lobby floor: who owns the geometry at several x along z 24?
        for x in [-16.0f32, -12.0, -8.0, -4.0, 0.0, 4.0] {
            eprintln!("[probe] buckets@({x},-369,24) {:?}",
                world.debug_bucket_owners([x - 1.0, -371.0, 23.0], [x + 1.0, -368.0, 25.0]));
        }
    }
    // The reach-only teleport (the gate13_live shape).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-19.49, -369.1, 23.9];
        sim.bond.yaw = 0.0;
        sim.bond.pitch = 0.0;
        sim.bond.vel = [0.0; 3];
    }
    w.tick(nf_game::player::Input::default(), false);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[probe] after teleport +1 tick: pos ({:.2},{:.2},{:.2}) cel {:?} grounded {}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.cel, sim.bond.grounded);
    }
    // E at the panel → ch 0x23 (the gate13_live press).
    let mut ch23 = false;
    for _ in 0..80 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x23) { ch23 = true; break; }
    }
    eprintln!("[probe] ch23 {ch23}");
    for _ in 0..400 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[probe] after the rise wait: pos ({:.2},{:.2},{:.2}) cel {:?} grounded {}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.cel, sim.bond.grounded);
    }
    // Walk east again (the legs an earlier walk saw falling).
    let legs = [(-16.0f32, -368.4, 25.0), (-12.0, -368.4, 28.0), (-6.0, -368.4, 35.0),
        (0.0, -368.4, 45.0), (3.0, -368.4, 50.0)];
    for (i, (x, y, z)) in legs.iter().enumerate() {
        let r = w.try_walk_to(Wp::new(*x, *y, *z).r(1.5), &format!("leg {i} ({x},{z})"));
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[probe] leg {i} ok {r}: pos ({:.2},{:.2},{:.2}) cel {:?} grounded {} falls {:?} stuck {}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.cel, sim.bond.grounded,
            w.report.falls, w.report.stuck_events.len());
        if sim.bond.pos[1] < -380.0 { eprintln!("[probe] FELL — stopping the probe"); break; }
    }
}

/// The k-th-pushed-instance ↔ k-th-cell alignment audit (716c): parse_cel_graph pushes one cell
/// per cel-creating placement ACROSS ALL MAPS (unresolvable ones become zero-bbox cells), while
/// `Level::instances(true)` yields MAIN-MAP cel placements only, and `build_scoped` pushes only
/// those whose object carries collision. Any (a) non-main-map cel placement, (b) unresolvable
/// cel placement or (c) collision-less cel placement desyncs the k-th instance from the k-th
/// cell and mis-attributes every later instance's scope bucket. Prints the counts per map.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_cell_instance_alignment() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let scene = nf_view::load(&iso, "07000013.bin", false).expect("load 07000013");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[align] graph cells {}  room_to_cell {:?}", sim.cel_graph.cells.len(), sim.cel_graph.room_to_cell);
    let level = {
        let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000013.bin")).expect("archive");
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        nf_formats::level::Level::parse(&data).expect("level")
    };
    let main = level.main_map();
    eprintln!("[align] main map {main:?} of {} maps", level.maps.len());
    let mut total_cel = 0usize;
    for (mi, map) in level.maps.iter().enumerate() {
        let mut cel = 0usize;
        let mut unres = 0usize;
        for p in &map.placements {
            if !p.creates_cel() { continue; }
            cel += 1;
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < map.objects.len() { Some((mi, p.index as usize)) } else { None };
            let has_col = target.map(|(m, o)| level.maps[m].objects[o].collision.is_some()).unwrap_or(false);
            if target.is_none() || !has_col { unres += 1; }
        }
        if cel > 0 {
            eprintln!("[align] map {mi}: cel placements {cel}, unresolvable-or-collision-less {unres}");
            total_cel += cel;
        }
    }
    eprintln!("[align] total cel-creating placements {total_cel}");
}

/// The southern-corridor walk probe (716c blocker 1): from the spawn, walk the east lane to the
/// lobby mid, then south-west along z ≈ 25-26.5 (south of the security-room glass band, which
/// spans z 27.5..35.3) to the west-wall panel, E it, and check the gate rises. Real input only.
/// Also dumps the floor ownership along the lane. Run with --ignored --nocapture.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_walk_south_corridor_to_panel() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = Walker::new(nf_view::load(&iso, "07000013.bin", false).expect("load 07000013"));
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (x, z) in [(-6.0f32, 26.0), (-10.0, 25.5), (-14.0, 25.5), (-17.0, 25.5), (-17.5, 26.0)] {
            eprintln!("[lane] buckets@({x},{z}) {:?}", sim.world
                .debug_bucket_owners([x - 0.8, -371.0, z - 0.8], [x + 0.8, -368.0, z + 0.8]));
        }
    }
    let legs = [
        (5.0, -368.4, 26.5), // the lobby east lane (the known-good opener)
        (-6.0, -368.4, 40.0), // swing south-west across the lobby mid
        (-6.0, -368.4, 28.0), // south along the west side
        (-8.0, -368.4, 25.8), // into the southern corridor
        (-12.0, -368.4, 25.8),
        (-16.5, -368.4, 25.5), // at the panel wall
    ];
    for (i, (x, y, z)) in legs.iter().enumerate() {
        let r = w.try_walk_to(Wp::new(*x, *y, *z).r(1.2), &format!("lane leg {i} ({x},{z})"));
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[lane] leg {i} ok {r}: pos ({:.2},{:.2},{:.2}) cel {:?} grounded {}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.cel, sim.bond.grounded);
        if sim.bond.pos[1] < -380.0 { eprintln!("[lane] FELL — stop"); return; }
    }
    // Face the panel (west) and press E.
    let mut ch23 = false;
    for t in 0..120 {
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (-18.08f32 - sim.bond.pos[0]).atan2(25.01 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input { turn: err.clamp(-0.05, 0.05), ..Default::default() };
        w.tick(input, t % 6 == 0);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x23) { ch23 = true; break; }
    }
    eprintln!("[lane] panel E → ch23 {ch23}");
    if ch23 {
        for _ in 0..400 { w.tick(nf_game::player::Input::default(), false); }
        let sim = w.scene.sim.as_ref().unwrap();
        let y = sim.drones.sss.list.iter().find(|s| s.channel == 0x23).map(|s| s.matrix[3][1]).unwrap_or(0.0);
        eprintln!("[lane] gate SS y {y:.2} (rise expected > 3)");
    }
}
/// Blocker-1 probe (716c): which placements' objects sit at the walker's stuck spots, and what
/// are their glist flags words (obj+0xF0 — the `Collide_PickObj` 0x1eae80:82 gate
/// `obj+0xf0 & uVar2 == 0 → skip`, uVar2 = 0x20 | query-flag bits). Prints per stuck spot the
/// objects whose transformed bbox holds it, name + flags + class byte if parsed.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_objects_at_stuck_spots() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000013.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = nf_formats::level::Level::parse(&data).expect("level");
    let m = level.main_map().expect("main map");
    let spots: [(&str, [f32; 2]); 6] = [
        ("stuck A", [-8.1, 21.6]), ("stuck B", [-12.8, 30.8]), ("stuck C", [-13.9, 31.9]),
        ("panel", [-18.08, 25.01]), ("door355 area", [-13.0, 31.0]), ("lobby mid", [-6.0, 45.0]),
    ];
    for (label, sxz) in spots {
        eprintln!("[obj] {label} ({:.1}, {:.1}):", sxz[0], sxz[1]);
        for (pi, p) in level.maps[m].placements.iter().enumerate() {
            let om = if p.by_hash() { level.by_hash.get(&p.hashcode).copied().map(|t| t.1) }
                else if (p.index as usize) < level.maps[m].objects.len() { Some(p.index as usize) } else { None };
            let Some(oi) = om else { continue };
            let o = &level.maps[m].objects[oi];
            let mtx = nf_formats::placement::object_matrix(p.rotation, p.translation);
            let mut lo = [f32::MAX; 3]; let mut hi = [f32::MIN; 3];
            for cx in [o.min[0], o.max[0]] { for cy in [o.min[1], o.max[1]] { for cz in [o.min[2], o.max[2]] {
                let w = nf_formats::placement::transform(&mtx, [cx, cy, cz]);
                for k in 0..3 { lo[k] = lo[k].min(w[k]); hi[k] = hi[k].max(w[k]); }
            } } }
            if lo[0] <= sxz[0] && sxz[0] <= hi[0] && lo[2] <= sxz[1] && sxz[1] <= hi[2]
                && lo[1] < -364.0 && hi[1] > -372.0 {
                eprintln!("[obj]   pl#{pi} obj#{oi} '{}'{:.40} flags {:#x} col {} bbox x {:.1}..{:.1} z {:.1}..{:.1}",
                    o.name, "", o.flags, o.collision.is_some(), lo[0], hi[0], lo[2], hi[2]);
            }
        }
    }
}

/// The security-room door route probe (716c blocker 1): dump the doors near the west end, walk
/// to the security-room door #355, E it, cross into the room, walk to the panel, E it. Run with
/// --ignored --nocapture.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_door355_route_to_panel() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = Walker::new(nf_view::load(&iso, "07000013.bin", false).expect("load 07000013"));
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.doors.list.iter().enumerate() {
            if d.pos[0] < -5.0 && d.pos[2] < 45.0 {
                eprintln!("[door] door[{i}] pos ({:.2},{:.2},{:.2}) kind {} state {:#x} ch_open {:#x} ch_lock {:#x} locked {} approach {}",
                    d.pos[0], d.pos[1], d.pos[2], d.kind, d.state, d.ch_open, d.ch_lock, d.locked_flag, d.approach);
            }
        }
    }
    // The east lane opener, then the west side to the door area (x −10.6 is the glass band's
    // east edge; the door sits in it).
    let legs = [
        (5.0, -368.4, 26.5),
        (-6.0, -368.4, 40.0),
        (-6.0, -368.4, 30.0),
        (-8.6, -368.4, 30.5), // approach the glass band's east edge from the lobby
    ];
    for (i, (x, y, z)) in legs.iter().enumerate() {
        let r = w.try_walk_to(Wp::new(*x, *y, *z).r(1.2), &format!("door route leg {i} ({x},{z})"));
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[door] leg {i} ok {r}: pos ({:.2},{:.2},{:.2}) cel {:?} grounded {}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.cel, sim.bond.grounded);
        if sim.bond.pos[1] < -380.0 { eprintln!("[door] FELL — stop"); return; }
    }
    // Find the nearest west door and E it.
    let (di, dpos) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let mut best: Option<(usize, [f32; 3])> = None;
        for (i, d) in sim.drones.doors.list.iter().enumerate() {
            if d.pos[0] < -5.0 && d.pos[2] < 45.0 {
                let dd = walker::dxz(d.pos, sim.bond.pos);
                if best.map_or(true, |(_, p)| dd < walker::dxz(p, sim.bond.pos)) { best = Some((i, d.pos)); }
            }
        }
        best.map(|(i, p)| (i, [p[0], p[1], p[2]])).expect("a west door")
    };
    eprintln!("[door] pressing E at door {di} ({:.2},{:.2},{:.2})", dpos[0], dpos[1], dpos[2]);
    let _ = w.walk_to(Wp::at(dpos[0], dpos[1], dpos[2], walker::Act::Use { use_ticks: 90 }).r(1.6), "door355 E");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[di];
        eprintln!("[door] after E: state {:#x} pos01 {:.2}", d.state, d.pos01);
    }
    // Through the door into the room, then south-west to the panel.
    let legs2 = [
        (-11.5, -368.4, 31.0), // through the doorway
        (-14.0, -368.4, 29.0),
        (-17.0, -368.4, 26.5), // inside the room, at the panel
    ];
    for (i, (x, y, z)) in legs2.iter().enumerate() {
        let r = w.try_walk_to(Wp::new(*x, *y, *z).r(1.2), &format!("room leg {i} ({x},{z})"));
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[door] room leg {i} ok {r}: pos ({:.2},{:.2},{:.2}) cel {:?} grounded {}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.cel, sim.bond.grounded);
        if sim.bond.pos[1] < -380.0 { eprintln!("[door] FELL — stop"); return; }
    }
    let mut ch23 = false;
    for t in 0..150 {
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (-18.08f32 - sim.bond.pos[0]).atan2(25.01 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input { turn: err.clamp(-0.05, 0.05), ..Default::default() };
        w.tick(input, t % 6 == 0);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x23) { ch23 = true; break; }
    }
    eprintln!("[door] panel E → ch23 {ch23}");
    if ch23 {
        for _ in 0..400 { w.tick(nf_game::player::Input::default(), false); }
        let sim = w.scene.sim.as_ref().unwrap();
        let y = sim.drones.sss.list.iter().find(|s| s.channel == 0x23).map(|s| s.matrix[3][1]).unwrap_or(0.0);
        eprintln!("[door] gate SS y {y:.2} (rise expected > 3)");
    }
}

/// The north-west door route probe (716c): door[1] at (−26.65, −368.36, 37.44) is the unlocked
/// E-openable door at portal 1 (21↔28). Route: the lobby's west side north of the glass band →
/// the door → into the security room → south to the panel → E. Run with --ignored --nocapture.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_northwest_door_route_to_panel() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = Walker::new(nf_view::load(&iso, "07000013.bin", false).expect("load 07000013"));
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.doors.list.iter().enumerate() {
            eprintln!("[nw] door[{i}] pos ({:.2},{:.2},{:.2}) kind {} state {:#x} ch_open {:#x} ch_lock {:#x} locked {}",
                d.pos[0], d.pos[1], d.pos[2], d.kind, d.state, d.ch_open, d.ch_lock, d.locked_flag);
        }
    }
    let legs = [
        (5.0, -368.4, 26.5),
        (-6.0, -368.4, 40.0),
        (-14.0, -368.4, 40.0),
        (-22.0, -368.4, 39.5),
        (-25.6, -368.4, 38.6),
    ];
    for (i, (x, y, z)) in legs.iter().enumerate() {
        let r = w.try_walk_to(Wp::new(*x, *y, *z).r(1.2), &format!("nw leg {i} ({x},{z})"));
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[nw] leg {i} ok {r}: pos ({:.2},{:.2},{:.2}) cel {:?} grounded {}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.cel, sim.bond.grounded);
        if sim.bond.pos[1] < -380.0 { eprintln!("[nw] FELL — stop"); return; }
    }
    // E the door (portal 1).
    let dpos = [-26.65f32, -368.36, 37.44];
    let _ = w.walk_to(Wp::at(dpos[0], dpos[1], dpos[2], walker::Act::Use { use_ticks: 120 }).r(1.4), "nw door E");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(d) = sim.drones.doors.list.iter().find(|d| walker::dxz(d.pos, dpos) < 2.0) {
            eprintln!("[nw] door after E: state {:#x} pos01 {:.2}", d.state, d.pos01);
        }
    }
    // Into the room, south to the panel. Wait for the sliding door to finish first.
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(d) = sim.drones.doors.list.iter().find(|d| walker::dxz(d.pos, dpos) < 2.0) {
            eprintln!("[nw] door after the wait: state {:#x} pos01 {:.2}", d.state, d.pos01);
        }
    }
    let legs2 = [
        (-27.0, -368.4, 37.0),
        (-27.0, -368.4, 35.0),
        (-25.5, -368.4, 32.5),
        (-22.0, -368.4, 29.0),
        (-18.5, -368.4, 26.2),
    ];
    for (i, (x, y, z)) in legs2.iter().enumerate() {
        let r = w.try_walk_to(Wp::new(*x, *y, *z).r(1.1), &format!("room leg {i} ({x},{z})"));
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[nw] room leg {i} ok {r}: pos ({:.2},{:.2},{:.2}) cel {:?} grounded {}",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.cel, sim.bond.grounded);
        if sim.bond.pos[1] < -380.0 { eprintln!("[nw] FELL — stop"); return; }
    }
    let mut ch23 = false;
    for t in 0..150 {
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (-18.08f32 - sim.bond.pos[0]).atan2(25.01 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input { turn: err.clamp(-0.05, 0.05), ..Default::default() };
        w.tick(input, t % 6 == 0);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x23) { ch23 = true; break; }
    }
    eprintln!("[nw] panel E → ch23 {ch23}");
    if ch23 {
        for _ in 0..400 { w.tick(nf_game::player::Input::default(), false); }
        let sim = w.scene.sim.as_ref().unwrap();
        let y = sim.drones.sss.list.iter().find(|s| s.channel == 0x23).map(|s| s.matrix[3][1]).unwrap_or(0.0);
        eprintln!("[nw] gate SS y {y:.2} (rise expected > 3)");
    }
}
