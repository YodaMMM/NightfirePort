//! level-14-empty-panels probe: what does the port draw and collide at the two "empty" spots —
//! the vent/hatch opening at (54.21, 1.03, 46.15) and the wall panel at (−88.58, −1.10, 46.15)?
//! Dumps, per spot: the placements near it (every map, with the glist word, the 0x8000 cel bit,
//! the collision block, the cel-graph node and its game-mask membership), the port's Materials
//! near it attributed to their nearest placement, and two collision rays (down; along the view
//! yaw). Assertion-free; run with:
//!   NF_ISO=<iso> cargo run -p nf-view --release --example dump_07000014_panels

use nf_formats::level::Level;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = nf_formats::iso9660::IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

const SPOTS: &[(&str, [f32; 3], f32)] = &[
    ("VENT", [54.21, 1.03, 46.15], -8.344),
    ("PANEL", [-88.58, -1.10, 46.15], -0.333),
];

fn main() { portal_trace(); overlaps(); cell_owners(); fusebox_script();
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap_or(0);
    let sc = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let g = &sc.cel_graph;
    let world = sc.sim.as_ref().map(|s| &s.world);

    for (tag, spot, yaw) in SPOTS {
        println!("\n===== {tag} spot {spot:?} yaw {yaw} =====");
        let mut state = nf_view::scene::ViewerCel::default();
        let eye = nf_view::scene::eye_cell(g, &mut state, *spot, world);
        let big = [1.0f32, 0.0, 0.0, 1.0e9];
        let sides = [big, [-1.0, 0.0, 0.0, 1.0e9], [0.0, 1.0, 0.0, 1.0e9], [0.0, 0.0, 1.0, 1.0e9]];
        let depth0 = [big, [-1.0, 0.0, 0.0, 1.0e9], [0.0, 1.0, 0.0, 1.0e9], [0.0, 0.0, 1.0, 1.0e9], [0.0, 0.0, 0.0, 1.0e9]];
        let mut mask = eye.and_then(|e| nf_view::scene::portal_visible_cels_game_from_cell_doors(g, e, *spot, &depth0, &sides, None));
        if let (Some(m), Some(w)) = (mask.as_mut(), world) {
            nf_view::scene::near_los_rescue(g, m, *spot, &sides, w, 30.0);
        }
        let mut wide = mask.clone().unwrap_or_else(|| vec![false; g.cells.len()]);
        for pe in &g.portals {
            if pe.cel_a < wide.len() && pe.cel_b < wide.len() {
                if wide[pe.cel_a] { wide[pe.cel_b] = true; }
                if wide[pe.cel_b] { wide[pe.cel_a] = true; }
            }
        }
        let room_vis: Vec<bool> = g.room_to_cell.iter().map(|m| m.map(|c| wide[c]).unwrap_or(true)).collect();
        println!("  eye cell {eye:?} (cells {}), game mask {}",
            g.cells.len(), mask.as_ref().map(|m| format!("{}/{} visible", m.iter().filter(|v| **v).count(), m.len())).unwrap_or("none".into()));

        // 1. Placements near the spot (any map).
        struct Near { label: String, kind: u32, glist: u32, cel: bool, coll: bool, centre: [f32; 3], node: Option<usize>, boxes: usize, leaves: usize }
        let mut nears: Vec<Near> = Vec::new();
        for (mi, m) in lv.maps.iter().enumerate() {
            for (pi, p) in m.placements.iter().enumerate() {
                let t = p.translation;
                let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
                let Some((omap, oidx)) = target else { continue };
                let o = &lv.maps[omap].objects[oidx];
                let wc = [t[0] + o.centre[0], t[1] + o.centre[1], t[2] + o.centre[2]];
                let d_t = ((t[0] - spot[0]).powi(2) + (t[1] - spot[1]).powi(2) + (t[2] - spot[2]).powi(2)).sqrt();
                let d_c = ((wc[0] - spot[0]).powi(2) + (wc[1] - spot[1]).powi(2) + (wc[2] - spot[2]).powi(2)).sqrt();
                if d_t > 15.0 && d_c > 15.0 { continue; }
                let node = if mi == mm && (pi as usize) < g.own_cells_main.len() { g.own_cells_main[pi].map(|n| n as usize) } else { None };
                let vis = node.and_then(|n| mask.as_ref().map(|m| m[n]));
                println!("m{mi} #{pi} kind {:#04x}{} \"{}\" @({:.2},{:.2},{:.2}) d_t {d_t:.1} d_c {d_c:.1} glist {:#x} boxes {} leaves {} coll {} node {node:?} node_vis {vis:?}",
                    p.flags & 0xff, if p.creates_cel() { " CEL" } else { "" }, o.name.trim(),
                    t[0], t[1], t[2], o.flags, o.gfx.boxes.len(), o.gfx.leaves.len(), o.collision.is_some());
                nears.push(Near { label: format!("m{mi}#{pi} \"{}\"", o.name.trim()), kind: p.flags & 0xff, glist: o.flags, cel: p.creates_cel(), coll: o.collision.is_some(), centre: wc, node, boxes: o.gfx.boxes.len(), leaves: o.gfx.leaves.len() });
            }
        }

        // 2. Materials near the spot, attributed to the nearest near-placement.
        println!("  -- port materials within 12:");
        let mut by_mat: std::collections::BTreeMap<String, u32> = Default::default();
        for mat in sc.materials.iter() {
            let c = mat.centre;
            let d = ((c[0] - spot[0]).powi(2) + (c[1] - spot[1]).powi(2) + (c[2] - spot[2]).powi(2)).sqrt();
            if d > 12.0 { continue; }
            let (mut best, mut bd) = (String::new(), f32::MAX);
            for n in &nears {
                let dn = ((c[0] - n.centre[0]).powi(2) + (c[1] - n.centre[1]).powi(2) + (c[2] - n.centre[2]).powi(2)).sqrt();
                if dn < bd { bd = dn; best = n.label.clone(); }
            }
            // The exact render-cull verdict (gpu.rs:1031-1035): orphan nodes force-draw; node-MAX
            // draws fall to their containing room's visibility (room_vis over scene.cels).
            let orphan = {
                let mut reached = vec![false; g.cells.len()];
                for p in &g.portals { if p.cel_a < g.cells.len() { reached[p.cel_a] = true; } if p.cel_b < g.cells.len() { reached[p.cel_b] = true; } }
                for ch in &g.chains { for &c in ch { if (c as usize) < g.cells.len() { reached[c as usize] = true; } } }
                reached.iter().map(|r| !r).collect::<Vec<bool>>()
            };
            let hidden = if mat.node != u32::MAX {
                !wide[mat.node as usize] && !orphan[mat.node as usize]
            } else {
                mat.cel.is_some_and(|c| !room_vis[c as usize])
            };
            let tag2 = if hidden { "HIDDEN" } else { "drawn" };
            *by_mat.entry(format!("{tag2} \"{best}\" d {bd:.1} | node {} chain {} cel {:?} lflags {:#06x} layer {:?}",
                mat.node, mat.chain, mat.cel, mat.lflags, mat.layer)).or_default() += 1;
        }
        for (k, n) in &by_mat { println!("    {n:4} × {k}"); }

        // 3. Collision rays: straight down, and along the view yaw (horizontal).
        if let (Some(w), Some(cell)) = (world, eye) {
            for (rname, from, to) in [
                ("down", [spot[0], spot[1] + 3.0, spot[2]], [spot[0], spot[1] - 8.0, spot[2]]),
                ("view", *spot, [spot[0] + 6.0 * yaw.sin(), spot[1], spot[2] + 6.0 * yaw.cos()]),
            ] {
                let hit = w.segment_hit_scoped(Some(cell), from, to);
                println!("  -- ray {rname} {from:?} → {to:?}: {hit:?}");
            }
        }
    }
}

// (appended section) — the exact cel_visibility pipeline + the room probes around the openings.
fn portal_trace() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    let sc = nf_view::load(&iso, "07000014.bin", false).expect("load");
    let g = &sc.cel_graph;
    let world = sc.sim.as_ref().map(|s| &s.world);
    println!("\n===== rooms ({}) / graph cells ({}) / portals {} =====", sc.cels.len(), g.cells.len(), g.portals.len());

    // Which room (scene.cels index) contains a point — smallest-volume containing bbox, find_cel's rule.
    let probe = |p: [f32; 3]| -> Option<usize> {
        let mut best = None; let mut bv = f32::INFINITY;
        for (i, c) in sc.cels.iter().enumerate() {
            let inside = c.min[0] <= p[0] && p[0] <= c.max[0] && c.min[1] <= p[1] && p[1] <= c.max[1] && c.min[2] <= p[2] && p[2] <= c.max[2];
            if !inside { continue; }
            let vol = (c.max[0] - c.min[0]) * (c.max[1] - c.min[1]) * (c.max[2] - c.min[2]);
            if vol < bv { bv = vol; best = Some(i); }
        }
        best
    };
    let points: Vec<(&str, [f32; 3])> = vec![
        ("vent eye", [54.21, 1.03, 46.15]),
        ("vent below", [54.21, -1.0, 46.15]),
        ("vent below deep", [54.21, -2.0, 46.15]),
        ("panel eye", [-88.58, -1.10, 46.15]),
        ("panel behind wall", [-92.0, -1.10, 48.0]),
        ("generator pit", [-82.5, -5.1, 43.2]),
    ];
    for (n, p) in &points {
        let r = probe(*p);
        let cell = r.and_then(|r| g.room_to_cell[r]);
        println!("  point {n} {p:?} → room {r:?} → graph cell {cell:?}");
    }

    // The exact visibility pipeline per the render (gpu.rs cel_visibility, no door gating):
    for (tag, spot) in [("VENT", [54.21f32, 1.03, 46.15]), ("PANEL", [-88.58, -1.10, 46.15])] {
        let mut state = nf_view::scene::ViewerCel::default();
        let eye = nf_view::scene::eye_cell(g, &mut state, spot, world);
        let Some(eye) = eye else { println!("  {tag}: no eye cell"); continue };
        let big = [1.0f32, 0.0, 0.0, 1.0e9];
        let sides = [big, [-1.0, 0.0, 0.0, 1.0e9], [0.0, 1.0, 0.0, 1.0e9], [0.0, 0.0, 1.0, 1.0e9]];
        let depth0 = [big, [-1.0, 0.0, 0.0, 1.0e9], [0.0, 1.0, 0.0, 1.0e9], [0.0, 0.0, 1.0, 1.0e9], [0.0, 0.0, 0.0, 1.0e9]];
        let mut game = nf_view::scene::portal_visible_cels_game_from_cell_doors(g, eye, spot, &depth0, &sides, None).expect("mask");
        if let Some(w) = world {
            nf_view::scene::near_los_rescue(g, &mut game, spot, &sides, w, 30.0);
        }
        // widen: one portal beyond.
        let mut wide = game.clone();
        for pe in &g.portals {
            if pe.cel_a < game.len() && pe.cel_b < game.len() {
                if game[pe.cel_a] { wide[pe.cel_b] = true; }
                if game[pe.cel_b] { wide[pe.cel_a] = true; }
            }
        }
        let vis: Vec<bool> = g.room_to_cell.iter().map(|m| m.map(|c| wide[c]).unwrap_or(true)).collect();
        let room_of = |i: usize| -> String {
            let c = g.room_to_cell[i];
            format!("cell {c:?} vis_wide {}", c.map(|cc| wide[cc]).map(|v| v.to_string()).unwrap_or("n/a".into()))
        };
        println!("  {tag}: eye cell {eye} raw {} rescued+widen {} rooms-visible {}/{}",
            game.iter().filter(|v| **v).count(), wide.iter().filter(|v| **v).count(), vis.iter().filter(|v| **v).count(), vis.len());
        for (n, p) in &points {
            if let Some(r) = probe(*p) {
                println!("    {tag} point {n} → room {r} ({})", room_of(r));
            }
        }
        // The pits and the intact generator's room membership under the REAL pipeline:
        for cell in [222usize, 224, 227, 226, 225, 223] {
            if cell < g.cells.len() {
                println!("    {tag} graph cell {cell} class {:#06x} vis_raw {} vis_wide {}", g.cells[cell].flags & 0xffff, game[cell], wide[cell]);
            }
        }
    }
}

// (overlaps section) — every placement whose object's world AABB (translation + glist min/max)
// overlaps the spaces behind/below the two openings.
fn overlaps() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    let data = { let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("archive");
        img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read") };
    let lv = Level::parse(&data).expect("parse");
    let regions: Vec<(&str, [f32; 3], [f32; 3])> = vec![
        ("VENT-below", [51.0, -4.0, 42.0], [58.0, 1.3, 50.0]),
        ("PANEL-behind", [-97.0, -4.0, 43.0], [-88.5, 2.5, 53.0]),
        ("GEN-pit", [-88.0, -10.0, 38.0], [-77.0, -0.4, 48.0]),
    ];
    for (tag, lo, hi) in regions {
        println!("\n--- region {tag} {lo:?}..{hi:?} ---");
        for (mi, m) in lv.maps.iter().enumerate() {
            for (pi, p) in m.placements.iter().enumerate() {
                let t = p.translation;
                let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
                let Some((omap, oidx)) = target else { continue };
                let o = &lv.maps[omap].objects[oidx];
                let (wmin, wmax) = (o.min, o.max);
                let mn = [t[0] + wmin[0].min(wmax[0]), t[1] + wmin[1].min(wmax[1]), t[2] + wmin[2].min(wmax[2])];
                let mx = [t[0] + wmin[0].max(wmax[0]), t[1] + wmin[1].max(wmax[1]), t[2] + wmin[2].max(wmax[2])];
                let overlap = mn[0] <= hi[0] && mx[0] >= lo[0] && mn[1] <= hi[1] && mx[1] >= lo[1] && mn[2] <= hi[2] && mx[2] >= lo[2];
                if !overlap { continue; }
                if o.gfx.boxes.is_empty() && o.gfx.leaves.is_empty() { continue; }
                println!("m{mi} #{pi} kind {:#04x}{} \"{}\" t({:.2},{:.2},{:.2}) aabb ({:.1},{:.1},{:.1})..({:.1},{:.1},{:.1}) glist {:#x} boxes {} leaves {} coll {}",
                    p.flags & 0xff, if p.creates_cel() { " CEL" } else { "" }, o.name.trim(),
                    t[0], t[1], t[2], mn[0], mn[1], mn[2], mx[0], mx[1], mx[2], o.flags, o.gfx.boxes.len(), o.gfx.leaves.len(), o.collision.is_some());
            }
        }
    }
}

// (cell-owners section) — which placement created each cel-graph cell (parse_cel_graph's order:
// sub-maps first, then the main map in placement order; one cell per creates_cel placement).
fn cell_owners() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    let data = { let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("archive");
        img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read") };
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap_or(0);
    let mut owners: Vec<Option<(usize, usize)>> = vec![None; 400];
    let mut idx = 0usize;
    'outer: for pass_main in [false, true] {
        for (mi, m) in lv.maps.iter().enumerate() {
            let is_main = mi == mm;
            if is_main != pass_main { continue; }
            for (pi, p) in m.placements.iter().enumerate() {
                if !p.creates_cel() { continue; }
                if idx >= owners.len() { break 'outer; }
                owners[idx] = Some((mi, pi));
                idx += 1;
            }
        }
    }
    let sc = nf_view::load(&iso, "07000014.bin", false).expect("load");
    let g = &sc.cel_graph;
    for cell in [222usize, 223, 224, 225, 226, 227, 43] {
        let Some(Some((mi, pi))) = owners.get(cell).copied() else { println!("cell {cell}: no owner"); continue };
        let m = &lv.maps[mi];
        let p = &m.placements[pi];
        let o = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied().and_then(|(om, oi)| lv.maps.get(om).and_then(|mm2| mm2.objects.get(oi))) }
            else { m.objects.get(p.index as usize) };
        let (name, aabb) = o.map(|o| (o.name.trim().to_string(), format!("({:.1},{:.1},{:.1})..({:.1},{:.1},{:.1})", p.translation[0] + o.min[0], p.translation[1] + o.min[1], p.translation[2] + o.min[2], p.translation[0] + o.max[0], p.translation[1] + o.max[1], p.translation[2] + o.max[2])))
            .unwrap_or(("?".into(), "?".into()));
        println!("cell {cell} ← m{mi} #{pi} kind {:#04x} \"{}\" t({:.2},{:.2},{:.2}) aabb {} glist {:#x} coll {} | node flags {:#06x} sphere ({:.1},{:.1},{:.1}) r {:.1}",
            p.flags & 0xff, name, p.translation[0], p.translation[1], p.translation[2], aabb, o.map(|o| o.flags).unwrap_or(0), o.map(|o| o.collision.is_some()).unwrap_or(false),
            g.cells[cell].flags & 0xffff, g.cells[cell].sphere.0[0], g.cells[cell].sphere.0[1], g.cells[cell].sphere.0[2], g.cells[cell].sphere.1);
    }
}

// (fusebox-script section) — decode script 0x06000A1B (the real fuseboxes' script [D params]) —
// what it spawns and when.
fn fusebox_script() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let Some(nis) = sim.drones.scripts.lib.get(&0x0600_0a1b) else { println!("0x06000A1B not in lib"); return };
    println!("\n== script 0x06000A1B: {} streams ==", nis.streams.len());
    for (si, st) in nis.streams.iter().enumerate() {
        println!(" stream {si}: keys {}..{} cmds {}", st.first_key, st.last_key, st.commands.len());
        for (t, c) in st.decode().iter().enumerate().take(20) {
            println!("   t{t}: {c:?}");
        }
    }
}
