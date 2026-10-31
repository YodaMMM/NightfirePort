//! walkreal probe: the escort's second cover run arrival wedge on 07000001 — the AI point a level
//! below the stalled lead, the move_test legs, and the route the port builds. Diagnostic only.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn dump_cover_wedge() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    let sim = scene.sim.as_ref().unwrap();
    let nav = &sim.nav;
    eprintln!("=== cover wedge probe ===");
    eprintln!("nav: bounds={} walls={}", nav.bounds, nav.walls.len());

    // The AI points around the observed goal (2.95, 5.35, 9.94) and the stalled lead (3.5, 6.3, 7.9).
    let near = |p: [f32; 3], c: [f32; 3], r: f32| (p[0]-c[0]).powi(2) + (p[2]-c[2]).powi(2) < r*r;
    for (i, ap) in sim.ai_points.iter().enumerate() {
        if near(ap.pos, [2.95, 5.35, 9.94], 25.0) || near(ap.pos, [3.5, 6.3, 7.9], 25.0) {
            eprintln!("aipoint[{i}] group {} at ({:.2},{:.2},{:.2})", ap.group, ap.pos[0], ap.pos[1], ap.pos[2]);
        }
    }

    let a: [f32; 3] = [3.10, 6.34, 7.98];
    let b: [f32; 3] = [2.95, 5.35, 9.94];
    eprintln!("--- move legs a={a:?} (lead, cover run 1 end) -> b={b:?} (the AI point below) ---");
    let dy = b[1] - a[1];
    let h2 = (b[0]-a[0]).powi(2) + (b[2]-a[2]).powi(2);
    eprintln!("dxz {:.3} dy {:.3} h2+dy2 {:.3}", h2.sqrt(), dy, h2 + dy*dy);
    eprintln!("world LOS at +1: {:?}", sim.world.line_of_sight([a[0], a[1]+1.0, a[2]], [b[0], b[1]+1.0, b[2]], nf_game::world::LOS_GUARD_VISION));
    eprintln!("world seg at feet y: {:?}", sim.world.segment(a, b));
    let (mt, hit) = nav.move_test(&sim.world, a, b, true, false);
    eprintln!("move_test near=true: {mt} hit {hit:?}");
    let (mt0, _) = nav.move_test(&sim.world, a, b, false, false);
    eprintln!("move_test near=false: {mt0}");
    // Which boundary walls sit near the segment?
    for (i, w) in nav.walls.iter().enumerate() {
        let mid = [(w.a[0]+w.b[0])/2.0, (w.a[1]+w.b[1])/2.0, (w.a[2]+w.b[2])/2.0];
        if near(mid, a, 36.0) || near(mid, b, 36.0) {
            eprintln!("wall[{i}] a=({:.2},{:.2},{:.2}) b=({:.2},{:.2},{:.2}) ha {:.1} hb {:.1} flags {:#x}",
                w.a[0], w.a[1], w.a[2], w.b[0], w.b[1], w.b[2], w.ha, w.hb, w.flags);
        }
    }
    // The route the port builds for that pair.
    let path = nav.nav_path_for(a);
    eprintln!("nav_path_for(a) = {path:?}");
    let nodes = path.map(|p| nav.target_nodes(&sim.world, p, b, 0.0)).unwrap_or_default();
    eprintln!("target_nodes = {nodes:?}");
    match nf_game::nav::Route::to_position(nav, &sim.world, path, a, b, &nodes, &[], &|_, _| false) {
        Ok((code, r)) => {
            let d = r.distance(nav, a);
            eprintln!("to_position: code {code} nodes {:?} index {} dest ({:.2},{:.2},{:.2}) dist(a) {d:.3}",
                r.nodes, r.index, r.dest[0], r.dest[1], r.dest[2]);
            eprintln!("  -> move_to_goal would return {}", if d < 2.0 { 3 } else { 0 });
        }
        Err(e) => eprintln!("to_position: Err {e}"),
    }
    // And the reverse sanity: from the lead's wedge spot to the plate side (-30, 6, 13).
    let c: [f32; 3] = [-30.0, 6.0, 13.0];
    let pathc = nav.nav_path_for(c);
    let nodesc = pathc.map(|p| nav.target_nodes(&sim.world, p, b, 0.0)).unwrap_or_default();
    match nf_game::nav::Route::to_position(nav, &sim.world, pathc, c, b, &nodesc, &[], &|_, _| false) {
        Ok((code, r)) => {
            let d = r.distance(nav, c);
            eprintln!("c->b: code {code} nodes {:?} dist {d:.3}", r.nodes);
        }
        Err(e) => eprintln!("c->b: Err {e}"),
    }
}
