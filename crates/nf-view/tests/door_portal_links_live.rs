//! door-portal-links (GAPS G-016 / G-035): the walk's per-portal `Door_IsClosed` gate against the
//! game's own numbers — 07000007 (the fix-perf level's start view) and 07000001.
//!
//! The gate [M `Vision_Recurse` 0x1e82e0: after the eye-side plane test (`fStack_f0 < 0.0`) the
//! walk reads `Door_IsClosed(portal+0xC)` (0x1367b0) and skips the portal when it reads closed —
//! no corner copy, no `ClipPortalWithPlane`, no destination-cel add]. The link [M
//! `build_LinkDoors2Portals` 0x1c8ac8: the straddle door's gfx sphere centre within 0.25 of the
//! quad's plane (`ABS(dot) < 0.25` @0x1c8c34) AND inside the quad (`vecutil_point_on_poly`
//! 0x1ec940); `Door_Link2Portal` 0x136788 = `obj+0xFD >> 3 & 1` = placement param 4 != 0].
//!
//! What these tests add over door_portal_live.rs: the 07000007 start view returns THE GAME'S cel
//! set (177 World draws [L fix-perf-level's live measurement, docs/atlas/cel.md]) — on that level
//! the game's own rule links NO door (every straddle door fails the plane/containment test, the
//! measured distances are pinned), so gated == legacy == all-open; on 07000001 the all-open walk
//! returns the legacy set per-cell (with it open, the old set), and the walk never ADDS cels in
//! any tested direction (the sister).
//!
//! Opt-in via `NF_ISO`.

use nf_view::scene;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn view_proj(pos: [f32; 3], yaw: f32, pitch: f32, aspect: f32) -> [[f32; 4]; 4] {
    use glam::{Mat4, Vec3};
    let (near, far, fov) = (0.05f32, 4000.0f32, 1.0471976f32); // 60°, the render path's GAME_FOV
    let f = 1.0 / (fov * 0.5).tan();
    let proj = Mat4::from_cols_array_2d(&[
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, far / (near - far), -1.0],
        [0.0, 0.0, near * far / (near - far), 0.0],
    ]);
    let cp = pitch.cos();
    let fw = Vec3::new(cp * yaw.sin(), pitch.sin(), cp * yaw.cos()).normalize();
    let s = fw.cross(Vec3::Y).normalize();
    let u = s.cross(fw);
    let view = Mat4::from_cols_array_2d(&[
        [s.x, u.x, -fw.x, 0.0],
        [s.y, u.y, -fw.y, 0.0],
        [s.z, u.z, -fw.z, 0.0],
        [-s.dot(Vec3::from(pos)), -u.dot(Vec3::from(pos)), fw.dot(Vec3::from(pos)), 1.0],
    ]);
    (proj * view).to_cols_array_2d()
}

fn depth0_game(vp: [[f32; 4]; 4], eye: [f32; 3]) -> [[f32; 4]; 5] {
    let planes = nf_view::gpu::extract_frustum(vp);
    let fw = [planes[4][0], planes[4][1], planes[4][2]];
    let fe = fw[0] * eye[0] + fw[1] * eye[1] + fw[2] * eye[2];
    [planes[0], planes[1], planes[2], planes[3], [-fw[0], -fw[1], -fw[2], fe + 200.0]]
}

fn quad_centre(c: &[[f32; 3]; 4]) -> [f32; 3] {
    [(c[0][0] + c[1][0] + c[2][0] + c[3][0]) * 0.25,
     (c[0][1] + c[1][1] + c[2][1] + c[3][1]) * 0.25,
     (c[0][2] + c[1][2] + c[2][2] + c[3][2]) * 0.25]
}

/// The submitted World draws for a room mask — gpu.rs's cel_visibility count headlessly
/// (the same rule fix_portal_live.rs's world_counts uses).
fn world_counts(scene: &scene::Scene, vp: [[f32; 4]; 4], vis: Option<&[bool]>) -> usize {
    let planes = nf_view::gpu::extract_frustum(vp);
    let mut submitted = 0usize;
    for m in &scene.materials {
        if m.layer != scene::Layer::World { continue; }
        if m.draw.pass != 4 && nf_view::gpu::sphere_outside(&planes, m.centre, m.radius) { continue; }
        match (vis, m.cel) {
            (Some(vis), Some(c)) => if !vis[c as usize] { continue; },
            _ => {}
        }
        submitted += 1;
    }
    submitted
}

fn room_vis(g: &scene::CelGraph, cell_vis: &[bool]) -> Vec<bool> {
    g.room_to_cell.iter().map(|m| m.map(|c| cell_vis[c]).unwrap_or(true)).collect()
}

/// The walk masks at one eye: (legacy = every portal open, gated = with the door snapshot,
/// all_open = the snapshot forced all-open).
fn masks(graph: &scene::CelGraph, cell: usize, eye: [f32; 3], vp: [[f32; 4]; 4], closed: &[bool], n_doors: usize)
    -> (Vec<bool>, Vec<bool>, Vec<bool>)
{
    let depth0 = depth0_game(vp, eye);
    let legacy = scene::portal_visible_cels_from_cell(graph, cell, eye, &depth0).expect("the legacy walk runs");
    let gated = scene::portal_visible_cels_from_cell_doors(graph, cell, eye, &depth0, Some(closed)).expect("the gated walk runs");
    let open = scene::portal_visible_cels_from_cell_doors(graph, cell, eye, &depth0, Some(&vec![false; n_doors])).expect("the all-open walk runs");
    (legacy, gated, open)
}

/// Cels the gated mask would ADD over legacy — the sister's subject; must always be empty
/// (the gate only ends portals, it can never open one [M Vision_Recurse 0x1e82e0]).
fn added_cels(legacy: &[bool], gated: &[bool]) -> Vec<usize> {
    (0..legacy.len()).filter(|&i| !legacy[i] && gated[i]).collect()
}

/// The nearest portal quad to `x` by plane distance: (distance, portal index).
fn nearest_portal_plane(g: &scene::CelGraph, x: [f32; 3]) -> (f32, usize) {
    let mut best = (f32::INFINITY, usize::MAX);
    for (pi, p) in g.portals.iter().enumerate() {
        let d = (p.n_fwd[0] * x[0] + p.n_fwd[1] * x[1] + p.n_fwd[2] * x[2] + p.d_fwd).abs();
        if d < best.0 { best = (d, pi); }
    }
    best
}

/// The game's link containment test (`vecutil_point_on_poly` 0x1ec940 over the quad's two
/// triangles) — the same barycentric rule link_doors_to_portals implements.
fn quad_contains(p: &scene::PortalEdge, x: [f32; 3]) -> bool {
    let c = &p.corners;
    let tri = |a: [f32; 3], b: [f32; 3], c: [f32; 3], x: [f32; 3]| {
        let dot = |p: [f32; 3], q: [f32; 3]| p[0] * q[0] + p[1] * q[1] + p[2] * q[2];
        let v0 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v1 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let v2 = [x[0] - a[0], x[1] - a[1], x[2] - a[2]];
        let (d00, d01, d11) = (dot(v0, v0), dot(v0, v1), dot(v1, v1));
        let (d20, d21) = (dot(v2, v0), dot(v2, v1));
        let den = d00 * d11 - d01 * d01;
        if den.abs() < 1e-12 { return false; }
        let u = (d11 * d20 - d01 * d21) / den;
        let v = (d00 * d21 - d01 * d20) / den;
        u >= -1e-4 && v >= -1e-4 && u + v <= 1.0 + 1e-4
    };
    tri(c[0], c[1], c[2], x) || tri(c[2], c[3], c[0], x)
}

/// An eye 2 units off a portal's centre along its normal, resolved to its endpoint cel:
/// (eye, yaw looking at the quad centre, the eye's cel).
fn eye_at_portal(graph: &scene::CelGraph, p: &scene::PortalEdge) -> Option<([f32; 3], f32, usize)> {
    let ctr = quad_centre(&p.corners);
    let eye = [ctr[0] + p.n_fwd[0] * 2.0, ctr[1] + p.n_fwd[1] * 2.0, ctr[2] + p.n_fwd[2] * 2.0];
    let mut st = scene::ViewerCel::default();
    let cell = scene::eye_cell(graph, &mut st, eye, None)?;
    if cell != p.cel_a && cell != p.cel_b { return None; }
    let dir = [ctr[0] - eye[0], ctr[1] - eye[1], ctr[2] - eye[2]];
    Some((eye, dir[0].atan2(dir[2]), cell))
}

/// 07000007 — the fix-perf level's start view: with the doors at create (CLOSED) the walk returns
/// THE GAME'S cel set — exactly the 177 World draws fix-perf-level measured from the game [L,
/// docs/atlas/cel.md "07000007: start view 4526 → 177"] — and the same set the legacy (every
/// portal open) walk returns: the game's own link rule leaves every one of the level's 7 straddle
/// doors unlinked (the measured plane/containment distances are pinned), so the gate is a no-op
/// here. Sisters: all-open == legacy per-cell, and the gate never ADDS cels over 16 yaws at the
/// start eye plus the two ground-truth spots.
#[test]
fn the_07000007_start_view_returns_the_games_cel_set() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    let g = &scene.cel_graph;
    let doors = &scene.sim.as_ref().unwrap().drones.doors;
    assert_eq!(g.portals.len(), 34, "the level's portal records [D the port's parse of the disc]");
    assert_eq!(doors.list.len(), 15, "15 doors [D docs/levels/07000007.md block 0xdb]");
    let straddlers: Vec<usize> = (0..doors.list.len()).filter(|&d| doors.list[d].straddle).collect();
    assert_eq!(straddlers, vec![0, 1, 2, 3, 4, 5, 6], "doors 0-6 carry placement param 4 (Door_Link2Portal [M 0x136788])");
    let closed = doors.closed_flags();
    let n_closed = closed.iter().filter(|c| **c).count();
    assert_eq!(n_closed, 14, "14/15 doors read closed at create [M Door_Create 0x136100: state 2]; door 7 starts OPEN (state 4)");
    assert!(!closed[7], "door 7 (state 4) does not read closed — Door_IsClosed is state == 2 EXACTLY [M 0x1367b0]");

    // THE LINK TABLE: every straddle door fails the game's own rule (plane < 0.25 AND contained
    // [M 0x1c8c34 + 0x1ec940]), measured on the port's disc parse — so nothing links.
    let links: Vec<usize> = g.portal_doors.iter().enumerate().filter_map(|(pi, d)| d.map(|_| pi)).collect();
    assert!(links.is_empty(), "no 07000007 door links a portal: got links at {:?}", links);
    let expected = [0.634f32, 0.637, 0.131, 0.636, 0.635, 0.637, 0.275];
    for (k, &di) in straddlers.iter().enumerate() {
        let x = doors.list[di].sph.0;
        let (dist, pi) = nearest_portal_plane(g, x);
        let within = dist < 0.25;
        let contained = within && quad_contains(&g.portals[pi], x);
        assert!(!contained, "door {di} must fail the game's link test (nearest plane dist {dist:.3})");
        assert!((dist - expected[k]).abs() < 2e-3,
            "door {di}'s nearest portal-plane distance is {dist:.3}, the parse measured {:.3}", expected[k]);
        if di == 2 {
            // The one door within 0.25 of a plane: the quad does not contain it (its centre is
            // (76.75, 45.63, -47.75), ~18 units from the door).
            assert!(!quad_contains(&g.portals[pi], x), "door 2's nearest quad (portal {pi}) must not contain it — vecutil_point_on_poly [M 0x1ec940] fails");
        }
    }

    // The start-view fixture (fix_portal_live's): eye = spawn + 1.6, the spawn yaw, 60°/16:9.
    let (pos, yaw) = scene.start.clone().unwrap();
    let eye = [pos[0], pos[1] + 1.6, pos[2]];
    let vp = view_proj(eye, yaw, 0.0, 16.0 / 9.0);
    let mut state = scene::ViewerCel::default();
    let cell = scene::eye_cell(g, &mut state, eye, None).expect("the eye cel resolves");
    eprintln!("[07] start eye ({:.2},{:.2},{:.2}) cell {cell}", eye[0], eye[1], eye[2]);
    let (legacy, gated, all_open) = masks(g, cell, eye, vp, &closed, doors.list.len());
    // With the doors CLOSED the walk returns the legacy set — the game's cel set (the gate is a
    // no-op on this level by its own rule).
    assert_eq!(gated, legacy, "gated (doors at create) == legacy (every portal open) on 07000007");
    // With it open, the old set.
    assert_eq!(all_open, legacy, "all-open == legacy on 07000007");
    // THE GAME'S NUMBER: 177 World draws [L fix-perf-level, docs/atlas/cel.md].
    let n_legacy = world_counts(&scene, vp, Some(&room_vis(g, &legacy)));
    let n_gated = world_counts(&scene, vp, Some(&room_vis(g, &gated)));
    eprintln!("[07] start view World draws: legacy {n_legacy}, gated {n_gated} — the game measured 177");
    assert_eq!(n_gated, 177, "the gated walk submits the game's 177 World draws at the 07000007 start view [L docs/atlas/cel.md]");
    assert_eq!(n_legacy, 177, "the legacy walk submits the same 177 (the gate removed nothing here)");
    // Sister: the gate never adds cels over 16 yaws at the start eye.
    for k in 0..16 {
        let y2 = yaw + (k as f32) * std::f32::consts::FRAC_PI_8;
        let vp2 = view_proj(eye, y2, 0.0, 16.0 / 9.0);
        let (l2, g2, _) = masks(g, cell, eye, vp2, &closed, doors.list.len());
        let adds = added_cels(&l2, &g2);
        assert!(adds.is_empty(), "yaw+{k}: the gated walk ADDED cels {:?} over the legacy walk — the gate can only end portals", adds);
    }
    // Sister at the two ground-truth spots (docs/live/2026-10-13-portal-ground-truth.md fixtures).
    for (name, e, y2, p2) in [
        ("window", [61.11f32, 44.12 + 0.7, -52.58], -2.541f32, -0.010f32 * std::f32::consts::FRAC_PI_2),
        ("clock hall", [34.25, 44.13 + 0.7, -70.71], 1.549, -0.070 * std::f32::consts::FRAC_PI_2),
    ] {
        let vp2 = view_proj(e, y2, p2, 16.0 / 9.0);
        let mut st2 = scene::ViewerCel::default();
        let c2 = scene::eye_cell(g, &mut st2, e, None).expect("the spot's cel resolves");
        let (l2, g2, o2) = masks(g, c2, e, vp2, &closed, doors.list.len());
        let adds = added_cels(&l2, &g2);
        assert!(adds.is_empty(), "{name}: the gated walk added cels {:?} over legacy", adds);
        assert_eq!(g2, l2, "{name}: gated == legacy (no links on this level)");
        assert_eq!(o2, l2, "{name}: all-open == legacy");
        eprintln!("[07 {name}] cell {c2}: legacy {} gated {} — adds 0",
            world_counts(&scene, vp2, Some(&room_vis(g, &l2))), world_counts(&scene, vp2, Some(&room_vis(g, &g2))));
    }
}

/// 07000001 — with the portalled doors OPEN the walk returns the old (every-portal-open) set,
/// per-cell, after the doors were driven through the REAL use path (`Player_Activate`
/// 0x1a5170's door arm → `Door_Activate` → `Door_Interp`): the game's `Door_IsClosed` reads false
/// on OPEN doors [M 0x1367b0] so `Vision_Recurse` crosses all three portalled edges again; the
/// two gallery_secretdoor leaves gate nothing (the game's rule links them to no quad
/// [M 0x1c8ac8's plane+containment test — the nearest record's centre is ~4 units away]).
/// Sisters: while SHUT the gate strictly shrinks the set (dest hidden, adds 0 over 16 yaws) and
/// the all-open snapshot IS the legacy walk at every tested direction.
#[test]
fn with_the_portalled_doors_open_the_07000001_walk_returns_the_old_set() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    // Snapshot everything the mutable half needs BEFORE any sim mutation.
    let p3 = scene.cel_graph.portals[3].clone();
    let linked: Vec<(usize, u32)> = scene.cel_graph.portal_doors.iter().enumerate()
        .filter_map(|(pi, d)| d.map(|di| (pi, di))).collect();
    assert_eq!(linked.len(), 3, "the three SlidingDoors hold the level's three links (door_portal_live pins the edges)");
    let edges: Vec<scene::PortalEdge> = linked.iter().map(|(pi, _)| scene.cel_graph.portals[*pi].clone()).collect();
    let n_doors = scene.sim.as_ref().unwrap().drones.doors.list.len();
    assert_eq!(n_doors, 5, "5 doors [D docs/atlas/door.md]");
    let closed = scene.sim.as_ref().unwrap().drones.doors.closed_flags();
    // The three PORTALLED SlidingDoors read closed at create (state 2, no group) [M Door_Create
    // 0x136100 + Door_IsClosed 0x1367b0]. The two gallery_secretdoor leaves carry a group, so
    // `Door_IsClosed` reads the GROUP state (DoorGroupsStates[g], 0 at create) — not their own
    // state — and does not read closed; they link no portal, so the walk never consults them.
    for di in 0..3 { assert!(closed[di], "door {di} (a portalled SlidingDoor) reads closed at create"); }
    eprintln!("[01] is_closed at create: {closed:?} (the group-carrying secretdoors read their group state, not their own)");

    // The door-0 fixture (door_portal_live's): the eye one standoff off portal 3, in cell 7.
    let ctr = quad_centre(&p3.corners);
    let (eye, yaw0, cell) = eye_at_portal(&scene.cel_graph, &p3).expect("the eye lands in an endpoint cel of portal 3");
    let dest = if cell == p3.cel_a { p3.cel_b } else { p3.cel_a };
    let vp = view_proj(eye, yaw0, 0.0, 16.0 / 9.0);
    let (legacy, gated, all_open) = masks(&scene.cel_graph, cell, eye, vp, &closed, n_doors);
    // SHUT: the gate strictly shrinks — dest leaves the set, nothing is added.
    assert!(legacy[dest], "cell {dest} is visible through portal 3 with every portal open (the pre-link walk)");
    assert!(!gated[dest], "cell {dest} is hidden while the door is shut [M Vision_Recurse 0x1e82e0's Door_IsClosed skip]");
    let adds = added_cels(&legacy, &gated);
    assert!(adds.is_empty(), "the gated walk added cels {:?} over legacy while shut", adds);
    let n_legacy = world_counts(&scene, vp, Some(&room_vis(&scene.cel_graph, &legacy)));
    let n_shut = world_counts(&scene, vp, Some(&room_vis(&scene.cel_graph, &gated)));
    eprintln!("[01] shut: legacy {n_legacy} draws, gated {n_shut} (dest {dest} hidden)");
    assert!(n_shut < n_legacy, "the shut door's gate must strictly reduce the World draws ({n_shut} !< {n_legacy})");
    // The all-open contract at the walk's input: every portal crossed again == the old set.
    assert_eq!(all_open, legacy, "all-open == legacy per-cell while the doors still stand shut (the snapshot is the walk's input)");
    // Sister: never adds in any of 16 directions while the doors stand shut.
    for k in 0..16 {
        let y2 = yaw0 + (k as f32) * std::f32::consts::FRAC_PI_8;
        let vp2 = view_proj(eye, y2, 0.0, 16.0 / 9.0);
        let (l2, g2, o2) = masks(&scene.cel_graph, cell, eye, vp2, &closed, n_doors);
        let adds = added_cels(&l2, &g2);
        assert!(adds.is_empty(), "yaw+{k} shut: the gated walk added cels {:?} over legacy", adds);
        assert_eq!(o2, l2, "yaw+{k}: all-open == legacy");
    }

    // OPEN through the real use path, door by door: each linked door's own side, head at the
    // door, looking at it (Player_Activate's door arm). The per-door eyes resolve first (read
    // pass), the uses and the ticks come after (write pass).
    let mut uses: Vec<([f32; 3], [f32; 3], u32)> = Vec::new();
    for (i, (_, di)) in linked.iter().enumerate() {
        let (eye_i, _, cell_i) = eye_at_portal(&scene.cel_graph, &edges[i]).expect("the door-side cel resolves");
        assert!(cell_i == edges[i].cel_a || cell_i == edges[i].cel_b, "the eye for door {di} lands in its portal's endpoint cel");
        let dpos = scene.sim.as_ref().unwrap().drones.doors.list[*di as usize].pos;
        let l = [dpos[0] - eye_i[0], 0.0, dpos[2] - eye_i[2]];
        let len = (l[0] * l[0] + l[2] * l[2]).sqrt().max(1e-4);
        let look = [l[0] / len, 0.0, l[2] / len];
        let head = [eye_i[0], dpos[1] + 1.1, eye_i[2]];
        uses.push((head, look, *di));
    }
    for (head, look, di) in &uses {
        let s = scene.sim.as_mut().unwrap();
        let (snd, hits) = s.drones.doors.player_use(*head, *look, &mut s.drones.channels, s.drones.frame, &s.world);
        assert!(hits >= 1, "door {di}: the use sphere meets the door");
        eprintln!("[01] door {di} use hits {hits} sounds {:?}", snd.iter().map(|s| s.0).collect::<Vec<_>>());
    }
    // The three portalled doors reach OPEN through the sim ticks.
    let mut opened = 0usize;
    for _ in 0..480 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        opened = scene.sim.as_ref().unwrap().drones.doors.list.iter().take(3).filter(|d| d.state == nf_game::door::OPEN).count();
        if opened == 3 { break; }
    }
    assert_eq!(opened, 3, "doors 0/1/2 reached OPEN through the use path");
    // The same fixture eye, the doors open: the walk returns THE OLD SET, per-cell.
    let states = scene.sim.as_ref().unwrap().drones.doors.closed_flags();
    assert!(states.iter().take(3).all(|c| !c), "doors 0/1/2 no longer read closed [M Door_IsClosed 0x1367b0]");
    let gated_open = scene::portal_visible_cels_from_cell_doors(&scene.cel_graph, cell, eye, &depth0_game(vp, eye), Some(&states)).expect("the gated walk runs");
    assert_eq!(gated_open, legacy, "with the portalled doors open the walk returns the old set per-cell (the secretdoors gate nothing — they link no quad)");
    let n_open = world_counts(&scene, vp, Some(&room_vis(&scene.cel_graph, &gated_open)));
    eprintln!("[01] open: gated {n_open} draws == legacy {n_legacy} (dest {dest} back)");
    assert_eq!(n_open, n_legacy, "the World draw count returns to the legacy count with the doors open");
    assert!(gated_open[dest], "dest {dest} is visible again once the door has opened through the use path");
    // Sister: even with the doors open, no direction adds cels (the gate never opens a portal).
    for k in 0..16 {
        let y2 = yaw0 + (k as f32) * std::f32::consts::FRAC_PI_8;
        let vp2 = view_proj(eye, y2, 0.0, 16.0 / 9.0);
        let (l2, g2, _) = masks(&scene.cel_graph, cell, eye, vp2, &states, n_doors);
        let adds = added_cels(&l2, &g2);
        assert!(adds.is_empty(), "yaw+{k} open: the gated walk added cels {:?} over legacy", adds);
    }
}

/// THE SISTER, standalone: the walk never ADDS cels in any direction — 16 yaws × 3 pitches on the
/// 07000007 start eye and the 07000001 door-0 fixture eye, with the doors at create (closed) and
/// with the snapshot forced all-open (which must equal legacy exactly in every direction).
#[test]
fn the_walk_never_adds_cels_in_any_direction() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    // 07000007: doors at create (14 closed, none linked).
    {
        let scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
        let g = &scene.cel_graph;
        let doors = &scene.sim.as_ref().unwrap().drones.doors;
        let closed = doors.closed_flags();
        let n_doors = doors.list.len();
        let (pos, yaw) = scene.start.clone().unwrap();
        let eye = [pos[0], pos[1] + 1.6, pos[2]];
        let mut state = scene::ViewerCel::default();
        let cell = scene::eye_cell(g, &mut state, eye, None).expect("the eye cel resolves");
        let mut checked = 0usize;
        for k in 0..16 {
            for &pitch in &[0.0f32, 0.35, -0.35] {
                let y2 = yaw + (k as f32) * std::f32::consts::FRAC_PI_8;
                let vp = view_proj(eye, y2, pitch, 16.0 / 9.0);
                let (l, gt, op) = masks(g, cell, eye, vp, &closed, n_doors);
                let adds = added_cels(&l, &gt);
                assert!(adds.is_empty(), "07000007 yaw+{k} pitch {pitch}: the gated walk added cels {:?} over legacy", adds);
                assert_eq!(op, l, "07000007 yaw+{k} pitch {pitch}: all-open != legacy — the all-open walk must BE the legacy walk");
                checked += 1;
            }
        }
        eprintln!("[07] sister: {checked} directions, 0 adds");
        assert!(checked >= 48, "the sweep must cover its directions");
    }
    // 07000001: the same sweep at the door-0 fixture.
    {
        let scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
        let g = &scene.cel_graph;
        let doors = &scene.sim.as_ref().unwrap().drones.doors;
        let closed = doors.closed_flags();
        let n_doors = doors.list.len();
        let p3 = g.portals[3].clone();
        let (eye, yaw0, _cell) = eye_at_portal(g, &p3).expect("the fixture eye resolves");
        let mut checked = 0usize;
        let mut shrunk_somewhere = false;
        for k in 0..16 {
            for &pitch in &[0.0f32, 0.35, -0.35] {
                let y2 = yaw0 + (k as f32) * std::f32::consts::FRAC_PI_8;
                let vp = view_proj(eye, y2, pitch, 16.0 / 9.0);
                let (l, gt, op) = masks(g, _cell, eye, vp, &closed, n_doors);
                let adds = added_cels(&l, &gt);
                assert!(adds.is_empty(), "07000001 yaw+{k} pitch {pitch}: the gated walk added cels {:?} over legacy", adds);
                assert_eq!(op, l, "07000001 yaw+{k} pitch {pitch}: all-open != legacy");
                if l.iter().zip(gt.iter()).any(|(a, b)| *a && !*b) { shrunk_somewhere = true; }
                checked += 1;
            }
        }
        eprintln!("[01] sister: {checked} directions, 0 adds, the gate shrank some set: {shrunk_somewhere}");
        assert!(checked >= 48, "the sweep must cover its directions");
        assert!(shrunk_somewhere, "the shut doors must hide something in at least one tested direction (the gate must DO something)");
    }
}
