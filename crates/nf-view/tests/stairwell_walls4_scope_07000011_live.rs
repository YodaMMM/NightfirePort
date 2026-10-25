//! stairwell-walls4-scope-11: the second stairwell spot's decal planes vs the game's bullet pick.
//!
//! From Bond (22.40, 30.09, 1.42, yaw -1.505, pitch -0.035) frame 2944 the port's flat
//! `segment_bullet` stops the round mid-well on map43#175 'exterior_cell5_back_right' (glist 0x0,
//! mat 0x0a — a diagonal plane under the stair flight and an upright plane across the well). The
//! game's flying-bullet pick is NOT flat: `Collide_Pick` 0x1eb320:125 takes its candidate nodes
//! ONLY from `Collide_StraddleCels` 0x1eac98 — the BFS from the bullet's cel (HITTEST+0x80, which
//! `Bullet_update` re-feeds every tick [M disasm 0x12f0cc-0x12f0d8]) through the portals the
//! segment crosses (`Intersect_Portal` 0x1ea7b8) — plus the ForcedList (dynamic objects; the
//! walk runs because the bullet's HITTEST+0x98 = 0 [M 0x12f040] clears the bit-0x10 gate).
//!
//! This assertion-free probe reports, for the reported spot's rays: Bond's cell (bbox
//! containment [S — the game's build_FindCel adds collision-ray tie-breaks]), the straddle set
//! the game's BFS would collect, and which cell each blocking plane's instance lives in. Run
//! with --ignored --nocapture.
//! Evidence tags: [M] decompile address, [D] disc bytes, [L] live measurement, [S] port choice.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn seg_tri_hit(from: [f32; 3], to: [f32; 3], t: &[[f32; 3]; 3]) -> bool {
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let d = sub(to, from);
    let e1 = sub(t[1], t[0]);
    let e2 = sub(t[2], t[0]);
    if dot(cross(e1, e2), d) >= 0.0 { return false; }
    let p = cross(d, e2);
    let det = dot(e1, p);
    if det.abs() < 1e-12 { return false; }
    let inv = 1.0 / det;
    let s = sub(from, t[0]);
    let u = dot(s, p) * inv;
    if !(0.0..=1.0).contains(&u) { return false; }
    let q = cross(s, e1);
    let v = dot(d, q) * inv;
    if v < 0.0 || u + v > 1.0 { return false; }
    let tt = dot(e2, q) * inv;
    (0.0..=1.0).contains(&tt)
}

#[test]
#[ignore = "assertion-free scope dump; run with --ignored --nocapture"]
fn dump_the_stairwell_bullet_straddle_set() {
    // The exact parse path Scene::build uses (headless): level → instances → cel graph → scope.
    let img = nf_formats::iso9660::IsoImage::open(&iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000011.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = nf_formats::level::Level::parse(&data).expect("parse 07000011");
    let instances = level.instances(true).0;
    let (_cels, cel_graph) = nf_view::scene::parse_cel_graph(&level);
    let sg = nf_view::scene::scope_graph(&cel_graph);
    let world = nf_game::world::CollisionWorld::build_scoped(&level, &instances, Some(sg.clone()));
    // Instance → graph cell: a map43 (main) placement resolves through `own_cells_main` (placement
    // ordinal → cells index [M parse_cel_graph]); match the instance's placement by identity.
    let main = level.main_map().expect("main map");
    let own = &cel_graph.own_cells_main;
    let cell_of_inst = |inst: &nf_formats::level::Instance| -> Option<usize> {
        for (ord, p) in level.maps[main].placements.iter().enumerate() {
            if p.hashcode == inst.placement.hashcode && p.index == inst.placement.index
                && p.translation == inst.placement.translation {
                return own.get(ord).copied().flatten().map(|c| c as usize);
            }
        }
        None
    };
    let id_of = |k: usize| -> String {
        let inst = &instances[k];
        let ob = &level.maps[inst.map].objects[inst.object];
        let cell = cell_of_inst(inst).map(|c| format!(" → graphcell{c}")).unwrap_or_default();
        format!("inst{k} map{}#{} '{}' glist {:#x}{}", inst.map, inst.object, ob.name, ob.flags, cell)
    };
    let cell_of = |m: usize, o: usize| -> Vec<usize> {
        instances.iter().enumerate()
            .filter(|(_, i)| i.map == m && i.object == o)
            .filter_map(|(k, i)| cell_of_inst(i).map(|c| (k, c)))
            .map(|(_, c)| c).collect()
    };
    let s = [22.40f32, 30.09, 1.42];
    // Bond's cell: bbox containment (report every candidate; the tie-break is the probe's read).
    let mut bond_cells: Vec<usize> = Vec::new();
    for (k, c) in sg.cells.iter().enumerate() {
        if (0..3).all(|ax| c.0[ax] <= s[ax] && s[ax] <= c.1[ax]) { bond_cells.push(k); }
    }
    eprintln!("[walls4s] spot cells (bbox containment): {:?} — {} cells total", bond_cells, sg.cells.len());
    // Graph cell → the instance that created it (own_cells_main back-map).
    let cell_owner: std::collections::HashMap<usize, usize> = instances.iter().enumerate()
        .filter_map(|(k, i)| cell_of_inst(i).map(|c| (c, k))).collect();
    for k in &bond_cells {
        eprintln!("[walls4s]   candidate graphcell{k}: {}", cell_owner.get(k).map(|&ik| id_of(ik)).unwrap_or_else(|| "(no instance/cel placement)".into()));
    }
    // The planes' instances and their graph cells.
    let want: [(&str, usize, usize); 3] = [("fudge126", 43, 126), ("back_right175", 43, 175), ("room127", 43, 127)];
    let mut want_insts: Vec<(usize, &'static str, usize)> = Vec::new(); // (instance ordinal, name)
    for (nm, m, o) in want {
        let cs: Vec<usize> = instances.iter().enumerate()
            .filter(|(_, i)| i.map == m && i.object == o).map(|(k, _)| k).collect();
        for k in &cs {
            eprintln!("[walls4s] {nm}: {} ", id_of(*k));
        }
        let gcells = cell_of(m, o);
        eprintln!("[walls4s] {nm}: graph cells {:?}", gcells);
        for k in cs { want_insts.push((k, nm, 0)); }
    }
    // The game's straddle BFS [M Collide_StraddleCels 0x1eac98]: from the start cell, through
    // every portal the segment crosses (either triangle — the ray walk's crossing test), both
    // halves walked per cell (the forward/reverse nodes).
    let straddle = |start: usize, from: [f32; 3], to: [f32; 3]| -> Vec<usize> {
        let mut seen = vec![false; sg.cells.len()];
        let mut out = Vec::new();
        let mut queue = vec![start];
        seen[start] = true;
        while let Some(c) = queue.pop() {
            out.push(c);
            for p in &sg.portals {
                let (other, hit) = if p.a == c { (p.b, true) } else if p.b == c { (p.a, true) } else { continue };
                let crosses = p.tris.iter().any(|t| seg_tri_hit(from, to, t));
                if hit && crosses && !seen[other] {
                    seen[other] = true;
                    queue.push(other);
                }
            }
        }
        out
    };
    // The centre ray and the near-centre fan, same lanes as the world.rs census.
    let p = (-0.035f32) * (std::f32::consts::PI / 2.0);
    let mut rays: Vec<(&str, [f32; 3])> = Vec::new();
    for (nm, yo, po) in [("centre", 0.0f32, 0.0), ("yaw-0.075", -0.075, 0.0), ("yaw+0.075", 0.075, 0.0), ("pitch-0.11", 0.0, -0.11), ("pitch+0.11", 0.0, 0.11)] {
        let a = yo + (-1.505f32);
        let q = (po * (std::f32::consts::PI / 2.0)) + p;
        let (sq, cq) = (q.sin(), q.cos());
        let (sa, ca) = (a.sin(), a.cos());
        rays.push((nm, [cq * sa, sq, cq * ca]));
    }
    let far = 70.0f32;
    for (nm, dir) in &rays {
        let to = [s[0] + dir[0] * far, s[1] + dir[1] * far, s[2] + dir[2] * far];
        for &start in &bond_cells {
            let cells = straddle(start, s, to);
            eprintln!("[walls4s] ray {nm} from cell{start}: {} straddle cells {:?}",
                cells.len(), cells.iter().map(|k| format!("cell{k}")).collect::<Vec<_>>());
            // Which of the planes' cells made it?
            for (nm2, m, o) in want {
                let cs = cell_of(m, o);
                let got: Vec<usize> = cs.iter().filter(|k| cells.contains(k)).copied().collect();
                eprintln!("[walls4s]   {nm2} straddles: {} of {:?}", got.len(), cs);
            }
            // The bucket CONTENT: every owner instance the scoped pick tests from this cell.
            if let Some(runs) = world.scoped_run_owners(start, s, to) {
                let mut owners: std::collections::BTreeMap<u32, usize> = Default::default();
                for (_, ow) in &runs { *owners.entry(*ow).or_insert(0) += 1; }
                eprintln!("[walls4s]   bucket: {} runs, {} owner instances", runs.len(), owners.len());
                for (ik, nm2, _) in &want_insts {
                    if let Some(n) = owners.get(&(*ik as u32)) {
                        eprintln!("[walls4s]     IN bucket: {} ({} runs)", id_of(*ik), n);
                    }
                }
                let mut names: Vec<&str> = Vec::new();
                for (ik, nm2, _) in &want_insts {
                    if owners.contains_key(&(*ik as u32)) { names.push(nm2); }
                }
                eprintln!("[walls4s]   planes in bucket: {}", if names.is_empty() { "NONE".to_string() } else { names.join(", ") });
            } else { eprintln!("[walls4s]   bucket: (no scope)"); }
            // The scoped and flat traces on this ray.
            eprintln!("[walls4s]   flat segment_bullet={:?} scoped_hit={:?}", world.segment_bullet(s, to),
                world.segment_hit_scoped(Some(start), s, to).map(|(t, _)| t));
            break; // one start cell per ray until the candidate set is understood
        }
    }
}
