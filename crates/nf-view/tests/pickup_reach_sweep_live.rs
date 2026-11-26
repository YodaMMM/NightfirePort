//! pickup-reach-sweep: every placed pickup on every campaign on-foot level, walked to with the
//! player's own movement from the nearest nav node that sees it. For each pickup: stand Bond on
//! the node (the one placement hop), then steer with `Input` only (turn + forward, a jump and a
//! strafe when the steer stops making ground) until the game's take test passes —
//! `Pickup::touches` with the drone pass's own arguments (Bond's cel, position, world-sphere
//! centre [M Pickup_Update 0x18e148:203-214]) — or the pickup is taken (gone / respawn state 2
//! [M Pickup_Handler 0x18e650:338-348]). A take the inventory refuses still counts as reached:
//! the sweep measures reach, not the take rules.
//!
//! The levels: `nf_view::dev::LEVELS` entries with a type-0x24 campaign start (the MP arenas carry
//! none [D 0700004b survey]). `NF_REACH_LEVELS=07000014,...` narrows the list. Each level runs as
//! its own test so the binary spreads across test threads.
//!
//! A pickup the straight walk misses gets three more tries, each still real input after one
//! placement: a floor-grid route (cells 0.5 u, rise <= 1.0 taken with a hop, drops <= 3, knee and
//! chest lanes clear) walked waypoint by waypoint; a ladder climb (walk into the ladder, hold
//! forward to the top dismount); and a start at a ladder's foot. Closed doors and E-operated
//! SimpleScripts on the way are opened with the use key; low overhangs are crouched under.
//!
//! The sweep prints one line per unreached pickup (`[reach] MISS ...`) with Bond's end state, the
//! first static object each lane meets (name + placement flags, from the disc), and the goal
//! side's read when no grid route exists (its size, the lowest climb into it, the moving objects
//! on its rim). It asserts that every unreached pickup is on `MISS_OK`, each entry with its cause.

#[path = "common/walker_b2.rs"]
mod walker;

use nf_game::player::Input;
use walker::{dxz, wrap_pi, Walker};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// One sweep result.
#[derive(Debug, Clone)]
pub struct Miss {
    pub k: usize,
    pub kind: u16,
    pub weapon: u16,
    pub at: [f32; 3],
    pub from: [f32; 3],
    pub end: [f32; 3],
    pub best: f32,
    pub why: String,
}

/// Every nav node of every path.
fn nav_nodes(sim: &nf_view::scene::Sim) -> Vec<[f32; 3]> {
    sim.nav.paths.iter().flat_map(|p| p.nodes.iter().map(|n| n.pos)).collect()
}

/// The start nodes for a pickup, best first. Same floor first: nodes whose height sits at the
/// pickup's floor + 0.4 (the nav nodes ride 0.4 over their floor [D the 07000014 booth nodes
/// 120-123 at 1.2 over the 0.8 deck]) within 0.7; else within 2.5 u of the pickup, else 6 u.
/// Within a set: the ones whose line to the pickup is clear of static geometry, nearest (XZ)
/// first, then the blocked ones.
fn start_nodes(sim: &nf_view::scene::Sim, at: [f32; 3]) -> Vec<([f32; 3], bool)> {
    let floor = sim.world.floor_below([at[0], at[1] + 0.5, at[2]], 4.0);
    let pick = |f: &dyn Fn(&[f32; 3]) -> bool| -> Vec<[f32; 3]> {
        let mut v: Vec<[f32; 3]> = nav_nodes(sim).into_iter().filter(|n| f(n) && dxz(*n, at) < 40.0).collect();
        v.sort_by(|a, b| dxz(*a, at).total_cmp(&dxz(*b, at)));
        v
    };
    let mut nodes = floor.map(|fl| pick(&|n: &[f32; 3]| (n[1] - 0.4 - fl).abs() < 0.7)).unwrap_or_default();
    if nodes.is_empty() { nodes = pick(&|n: &[f32; 3]| (n[1] - at[1]).abs() < 2.5); }
    if nodes.is_empty() { nodes = pick(&|n: &[f32; 3]| (n[1] - at[1]).abs() < 6.0); }
    let clear = |n: &[f32; 3]| {
        let a = [n[0], n[1] + 0.3, n[2]];
        let b = [at[0], at[1] + 0.3, at[2]];
        sim.world.segment(a, b).is_none()
    };
    let mut out: Vec<([f32; 3], bool)> = nodes.iter().filter(|n| clear(n)).map(|n| (*n, true)).collect();
    out.extend(nodes.iter().filter(|n| !clear(n)).map(|n| (*n, false)));
    out
}

/// Stand Bond on `p` (the floor under it + 1.1, the game's standing height [M Bond::stand_at]),
/// facing `face`. The sweep's one placement per pickup.
fn place(w: &mut Walker, p: [f32; 3], face: [f32; 3]) {
    let sim = w.scene.sim.as_mut().unwrap();
    let y = sim.world.floor_below([p[0], p[1] + 1.0, p[2]], 4.0).map_or(p[1], |f| f + 1.1);
    let at = [p[0], y, p[2]];
    sim.bond.pos = at;
    sim.bond.prev_pos = at;
    sim.bond.vel = [0.0; 3];
    sim.bond.yaw = (face[0] - p[0]).atan2(face[2] - p[2]);
    sim.bond.pitch = 0.0;
    sim.bond.cel = sim.world.find_cel(at);
    for _ in 0..8 { w.tick(Input::default(), false); }
}

/// Whether pickup `k` is reached this tick (the game's take test, or already taken).
fn reached(sim: &nf_view::scene::Sim, k: usize) -> bool {
    let p = &sim.drones.pickups[k];
    if p.gone || p.state == 2 { return true; }
    p.touches(&sim.world, sim.bond.cel, sim.bond.pos, sim.bond.sphere_centre())
}

/// Steer toward `goal` with real input until `done` (or the pickup is reached), a closed door in
/// the way is opened with the use key (the player's E [M Door_PlayerUse via `player_use`]), and
/// a stalled steer tries a jump and both strafes. Returns (reached pickup, best xz, ticks).
fn steer_to(w: &mut Walker, k: Option<usize>, goal: [f32; 3], arrive: f32, cap: u32, used: &mut Vec<usize>) -> (bool, f32, u32) {
    let mut best = f32::MAX;
    let mut since = 0u32;
    let mut escape = 0u32;
    let mut esc_ticks = 0u32;
    let mut use_ticks = 0u32;
    let mut use_door: Option<[f32; 3]> = None;
    for t in 0..cap {
        let sim = w.scene.sim.as_ref().unwrap();
        if k.is_some_and(|k| reached(sim, k)) { return (true, best.min(dxz(sim.bond.pos, goal)), t); }
        if sim.disabled { w.tick(Input::default(), false); continue; }
        let p = sim.bond.pos;
        let d = dxz(p, goal);
        if d < arrive { return (false, d, t); }
        if d < best - 0.05 { best = d; since = 0; } else { since += 1; }
        let aim = use_door.unwrap_or(goal);
        let desired = (aim[0] - p[0]).atan2(aim[2] - p[2]);
        let err = wrap_pi(desired - sim.bond.yaw);
        let mut input = Input {
            turn: err.clamp(-0.15, 0.15),
            forward: if err.abs() > 0.8 || use_ticks > 0 { 0.0 } else if d < 0.6 { 0.4 } else { 1.0 },
            ..Default::default()
        };
        // A ledge up ahead: hop onto it (the player's jump, every 30 ticks while grounded).
        if goal[1] - p[1] > 0.35 && d < 1.5 && sim.bond.grounded && t % 30 == 0 { input.jump = true; }
        if use_ticks > 0 {
            use_ticks -= 1;
            if use_ticks == 0 { use_door = None; }
            let press = err.abs() < 0.2 && use_ticks % 20 == 0;
            w.tick(input, press);
            continue;
        }
        if esc_ticks > 0 {
            esc_ticks -= 1;
            match escape % 4 {
                0 => { input.jump = esc_ticks % 20 == 0; }
                1 => { input.strafe = 1.0; }
                2 => { input.strafe = -1.0; }
                // Duck under a low overhang (the crouch press [M Player_HandleJump 0x1916b0:89-95]).
                _ => { input.crouch = esc_ticks == 39 && !sim.bond.crouched; }
            }
        } else if since > 90 {
            // A closed door within reach on Bond's floor: open it first.
            let door = sim.drones.doors.list.iter().enumerate()
                .filter(|(i, dr)| !used.contains(i) && dr.pos01 < 0.5 && dxz(dr.pos, p) < 3.0 && (dr.pos[1] - p[1]).abs() < 2.5)
                .min_by(|a, b| dxz(a.1.pos, p).total_cmp(&dxz(b.1.pos, p)))
                .map(|(i, dr)| (i, dr.sph.0));
            // An E-operated SimpleScript (a grill or hatch: activate mask bit 2, armed state 0x40
            // [M SS_Activate 0x1a9ea8]) within reach: operate it.
            let door = door.or_else(|| sim.drones.sss.list.iter().enumerate()
                .filter(|(i, x)| !used.contains(&(10_000 + i)) && x.mask & 4 != 0 && x.state == 0x40
                    && dxz([x.matrix[3][0], x.matrix[3][1], x.matrix[3][2]], p) < 3.0)
                .map(|(i, x)| (10_000 + i, [x.matrix[3][0], x.matrix[3][1], x.matrix[3][2]])).next());
            if let Some((i, c)) = door {
                used.push(i);
                use_door = Some(c);
                use_ticks = 120;
                since = 0;
                w.tick(input, false);
                continue;
            }
            if escape >= 8 { return (false, best, t); }
            escape += 1;
            esc_ticks = 40;
            since = 0;
        }
        w.tick(input, false);
    }
    (false, best, cap)
}

/// Walk to pickup `k` from where Bond stands: straight when the start node sees it, else through
/// the nav route's nodes first.
fn steer(w: &mut Walker, k: usize, los: bool, cap: u32) -> (bool, f32, u32) {
    let target = w.scene.sim.as_ref().unwrap().drones.pickups[k].centre();
    let mut used = Vec::new();
    let mut spent = 0u32;
    if !los {
        let route = {
            let sim = w.scene.sim.as_ref().unwrap();
            walker::nav_route(sim, sim.bond.pos, target).unwrap_or_default()
        };
        for n in route {
            let (ok, _, t) = steer_to(w, Some(k), n, 0.6, 900, &mut used);
            spent += t;
            if ok { return (true, 0.0, spent); }
        }
    }
    let (ok, best, t) = steer_to(w, Some(k), target, 0.0, cap, &mut used);
    (ok, best, spent + t)
}

/// The take spots: floor points within the take radius of pickup `k` where Bond, placed and idled
/// 8 ticks, stays put (the capsule fits) and the game's take test passes. Returns the first spot
/// found (scanned on a 0.25 grid, nearest first) — `None` = no place on any floor near the pickup
/// takes it, i.e. the pickup cannot be taken by walking anywhere.
fn take_spot(w: &mut Walker, k: usize) -> Option<[f32; 3]> {
    let (c, r) = { let p = &w.scene.sim.as_ref().unwrap().drones.pickups[k]; (p.centre(), p.bounds.radius + 1.1) };
    let mut pts: Vec<[f32; 3]> = Vec::new();
    let n = (r / 0.25).ceil() as i32;
    for ix in -n..=n { for iz in -n..=n {
        let (x, z) = (c[0] + ix as f32 * 0.25, c[2] + iz as f32 * 0.25);
        if ((x - c[0]).powi(2) + (z - c[2]).powi(2)).sqrt() > r { continue; }
        // Every floor from 1.6 over the pickup down to 2.7 under it (Bond's pos sits 1.1 over his floor).
        let mut top = c[1] + 1.6;
        for _ in 0..3 {
            let Some(f) = w.scene.sim.as_ref().unwrap().world.floor_below([x, top, z], top - (c[1] - 2.7)) else { break };
            pts.push([x, f, z]);
            top = f - 0.05;
        }
    } }
    pts.sort_by(|a, b| dxz(*a, c).total_cmp(&dxz(*b, c)));
    for f in pts {
        let at = [f[0], f[1] + 1.1, f[2]];
        if ((at[0] - c[0]).powi(2) + (at[1] - c[1]).powi(2) + (at[2] - c[2]).powi(2)).sqrt() > r { continue; }
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = at; sim.bond.prev_pos = at; sim.bond.vel = [0.0; 3];
            sim.bond.cel = sim.world.find_cel(at);
        }
        let mut hit = false;
        for _ in 0..8 {
            w.tick(Input::default(), false);
            if reached(w.scene.sim.as_ref().unwrap(), k) { hit = true; break; }
        }
        let moved = dxz(w.scene.sim.as_ref().unwrap().bond.pos, at);
        if hit && moved < 0.3 { return Some(f); }
    }
    None
}

/// A floor-grid route to pickup `k` (the sweep's planner, a port-side search, not a game rule):
/// cells 0.5 u apart over ±25 u, each cell's floor the first floor under the pickup's height +
/// 1.6; a move to a neighbour cell needs the rise ≤ 0.45 (any drop ≤ 3 is taken) and a clear
/// line at knee and chest height in both directions (the walls the floor ray cannot see). Seeds:
/// nav nodes standing on their cell's floor. Goal: a cell whose stand point (floor + 1.1) is
/// inside the take radius. Returns the cell centres (floor + 1.1) from a seed to the goal.

/// The grid search: the route (as above), and when there is none, the goal side's component
/// read: walking BACK from the goal cells under the same rules, its size, whether any nav node
/// stands in it, and the cheapest way in — the lowest rise over its rim from a seed-side-free cell
/// (a climb), and how many rim edges only a moving object (door/prop) blocks.
fn grid_search(sim: &nf_view::scene::Sim, k: usize, seed: Option<[f32; 3]>) -> (Option<Vec<[f32; 3]>>, String) {
    let p = &sim.drones.pickups[k];
    let (c, r) = (p.centre(), p.bounds.radius + 1.1);
    const N: i32 = 50;
    const S: f32 = 0.5;
    // Stairs sample up to ~0.6 between cells 0.5 apart and a ledge to 1.0 is a hop; the real
    // walk proves every route.
    const RISE: f32 = 1.0;
    let w = 2 * N + 1;
    let idx = |ix: i32, iz: i32| ((iz + N) * w + (ix + N)) as usize;
    let at = |ix: i32, iz: i32| [c[0] + ix as f32 * S, c[2] + iz as f32 * S];
    let mut h: Vec<Option<f32>> = vec![None; (w * w) as usize];
    for iz in -N..=N { for ix in -N..=N {
        let [x, z] = at(ix, iz);
        h[idx(ix, iz)] = sim.world.floor_below([x, c[1] + 1.6, z], 8.0);
    } }
    let clear = |a: [f32; 3], b: [f32; 3]| sim.world.segment(a, b).is_none() && sim.world.segment(b, a).is_none();
    // The capsule's room: a cell whose stand point has a static wall within 0.3 at knee or chest
    // height is no stand (Bond's capsule radius is 0.55 [player::RADIUS]; 0.3 keeps the cells of
    // narrow doorways and crawl gaps, the real walk decides those).
    let roomy = |x: f32, f: f32, z: f32| (0..8).all(|q| {
        let a = q as f32 * std::f32::consts::FRAC_PI_4;
        [0.6f32, 1.2].iter().all(|&y| {
            let p0 = [x, f + y, z];
            let p1 = [x + a.sin() * 0.3, f + y, z + a.cos() * 0.3];
            !sim.world.segment_blocked_static(p0, p1)
        })
    });
    for iz in -N..=N { for ix in -N..=N {
        let [x, z] = at(ix, iz);
        if let Some(f) = h[idx(ix, iz)] { if !roomy(x, f, z) { h[idx(ix, iz)] = None; } }
    } }
    let clear_static = |a: [f32; 3], b: [f32; 3]| !sim.world.segment_blocked_static(a, b) && !sim.world.segment_blocked_static(b, a);
    let lanes = |ix: i32, iz: i32, fa: f32, jx: i32, jz: i32, fb: f32, f: &dyn Fn([f32; 3], [f32; 3]) -> bool| {
        let [ax, az] = at(ix, iz);
        let [bx, bz] = at(jx, jz);
        let top = fa.max(fb);
        f([ax, top + 0.6, az], [bx, top + 0.6, bz]) && f([ax, top + 1.2, az], [bx, top + 1.2, bz])
    };
    let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];
    let goal = |ix: i32, iz: i32, f: f32| {
        let [x, z] = at(ix, iz);
        ((x - c[0]).powi(2) + (f + 1.1 - c[1]).powi(2) + (z - c[2]).powi(2)).sqrt() < r - 0.05
    };
    let mut node_cell = vec![false; h.len()];
    let seeds: Vec<[f32; 3]> = match seed { Some(b) => vec![[b[0], b[1] - 0.7, b[2]]], None => nav_nodes(sim) };
    for n in seeds {
        let ix = ((n[0] - c[0]) / S).round() as i32;
        let iz = ((n[2] - c[2]) / S).round() as i32;
        if ix.abs() > N || iz.abs() > N { continue; }
        if let Some(f) = h[idx(ix, iz)] { if (n[1] - 0.4 - f).abs() < 0.5 { node_cell[idx(ix, iz)] = true; } }
    }
    let mut prev: Vec<i32> = vec![-2; h.len()];
    let mut q = std::collections::VecDeque::new();
    for i in 0..h.len() { if node_cell[i] { prev[i] = -1; q.push_back(((i as i32) % w - N, (i as i32) / w - N)); } }
    while let Some((ix, iz)) = q.pop_front() {
        let fa = h[idx(ix, iz)].unwrap();
        if goal(ix, iz, fa) {
            if std::env::var_os("NF_REACH_DEBUG").is_some() { eprintln!("[reach-dbg] goal cell {:?} floor {fa} centre {c:?} r {r}", at(ix, iz)); }
            let mut out = Vec::new();
            let mut cur = idx(ix, iz) as i32;
            while cur >= 0 {
                let (cx, cz) = (cur % w - N, cur / w - N);
                let [x, z] = at(cx, cz);
                out.push([x, h[cur as usize].unwrap() + 1.1, z]);
                cur = prev[cur as usize];
            }
            out.reverse();
            return (Some(out), String::new());
        }
        for (dx, dz) in dirs {
            let (jx, jz) = (ix + dx, iz + dz);
            if jx.abs() > N || jz.abs() > N || prev[idx(jx, jz)] != -2 { continue; }
            let Some(fb) = h[idx(jx, jz)] else { continue };
            if fb - fa > RISE || fa - fb > 3.0 { continue; }
            if !lanes(ix, iz, fa, jx, jz, fb, &clear) { continue; }
            prev[idx(jx, jz)] = idx(ix, iz) as i32;
            q.push_back((jx, jz));
        }
    }
    // No route: the goal side's component (reverse moves).
    let mut comp = vec![false; h.len()];
    let mut q = std::collections::VecDeque::new();
    for iz in -N..=N { for ix in -N..=N {
        if let Some(f) = h[idx(ix, iz)] { if goal(ix, iz, f) { comp[idx(ix, iz)] = true; q.push_back((ix, iz)); } }
    } }
    let goals = q.len();
    let (mut size, mut nodes, mut edge) = (0usize, 0usize, false);
    let mut climb = f32::MAX;
    let mut dyn_rim = 0usize;
    let mut dyn_hits: Vec<String> = Vec::new();
    let (mut fmin, mut fmax) = (f32::MAX, f32::MIN);
    while let Some((jx, jz)) = q.pop_front() {
        size += 1;
        let fb = h[idx(jx, jz)].unwrap();
        fmin = fmin.min(fb); fmax = fmax.max(fb);
        if node_cell[idx(jx, jz)] { nodes += 1; }
        for (dx, dz) in dirs {
            let (ix, iz) = (jx + dx, jz + dz);
            if ix.abs() > N || iz.abs() > N { edge = true; continue; }
            if comp[idx(ix, iz)] { continue; }
            let Some(fa) = h[idx(ix, iz)] else { continue };
            // a → b forward, entering the component.
            if fa - fb > 3.0 { continue; }
            let static_ok = lanes(ix, iz, fa, jx, jz, fb, &clear_static);
            if fb - fa > RISE {
                if static_ok { climb = climb.min(fb - fa); }
                continue;
            }
            if !lanes(ix, iz, fa, jx, jz, fb, &clear) {
                if static_ok {
                    dyn_rim += 1;
                    if dyn_hits.len() < 3 {
                        let ([ax, az], [bx, bz]) = (at(ix, iz), at(jx, jz));
                        let top = fa.max(fb);
                        for y in [top + 0.6, top + 1.2] {
                            let (a, b) = ([ax, y, az], [bx, y, bz]);
                            let hit = sim.world.segment_surface_host(a, b, false).map(|h| (a, b, h))
                                .or_else(|| sim.world.segment_surface_host(b, a, false).map(|h| (b, a, h)));
                            if let Some((a, b, (t, _, _, _, host))) = hit {
                                let pt = [0, 1, 2].map(|q| a[q] + t * (b[q] - a[q]));
                                let id = match host { nf_game::world::HitHost::Object(id, _) => format!("{id:#x}"), nf_game::world::HitHost::Cel(c) => format!("cel{c:?}") };
                                dyn_hits.push(format!("({:.1},{:.1},{:.1}) {id}", pt[0], pt[1], pt[2]));
                                break;
                            }
                        }
                    }
                }
                continue;
            }
            comp[idx(ix, iz)] = true;
            q.push_back((ix, iz));
        }
    }
    let climb = if climb == f32::MAX { "none".to_string() } else { format!("{climb:.2}") };
    (None, format!(" goal-side: {goals} goal cells, {size} cells, floors {fmin:.2}..{fmax:.2}, {nodes} nav cells, window edge {edge}, lowest climb in {climb}, door/prop-only rim edges {dyn_rim} {dyn_hits:?}"))
}

/// Walk the grid route with real input: stand on its seed node, steer through every third
/// cell, then onto the pickup.
fn walk_grid(w: &mut Walker, k: usize, route: &[[f32; 3]]) -> bool {
    let target = w.scene.sim.as_ref().unwrap().drones.pickups[k].centre();
    place(w, route[0], route.get(1).copied().unwrap_or(target));
    let mut used = Vec::new();
    let dbg = std::env::var_os("NF_REACH_DEBUG").is_some();
    if dbg { eprintln!("[reach-dbg] #{k} route {:?}", route.iter().map(|p| p.map(|v| (v * 100.0).round() / 100.0)).collect::<Vec<_>>()); }
    for wp in route.iter().skip(1).step_by(3) {
        let (ok, d, t) = steer_to(w, Some(k), *wp, 0.35, 600, &mut used);
        if dbg { let sim = w.scene.sim.as_ref().unwrap(); eprintln!("[reach-dbg]   wp {:?} d {d:.2} t {t} bond {:?} crouched {}", wp, sim.bond.pos, sim.bond.crouched); }
        if ok { return true; }
    }
    steer_to(w, Some(k), target, 0.0, 600, &mut used).0
}

/// The ladder leg for a miss: every ladder within 20 u whose top reaches the pickup's floor,
/// approached from both faces (stand on the floor at the ladder's foot 1 u out, walk into it until
/// the climb state [M Player_CollLadder], hold forward to the top dismount), then the straight steer
/// and the grid route from where Bond stands. Returns the ladder centre that worked.
fn ladder_leg(w: &mut Walker, k: usize) -> Option<[f32; 3]> {
    let c = w.scene.sim.as_ref().unwrap().drones.pickups[k].centre();
    let ladders: Vec<([f32; 3], f32, f32, f32)> = w.scene.sim.as_ref().unwrap().ladders.iter()
        .filter(|l| dxz(l.centre, c) < 20.0 && l.top() > c[1] - 1.5 && l.bottom() < c[1] + 6.0)
        .map(|l| (l.centre, l.yaw, l.bottom(), l.top())).collect();
    for (lc, yaw, bottom, top) in ladders {
        for side in [1.0f32, -1.0] {
            let b = [lc[0] + side * yaw.sin() * 1.0, bottom + 0.5, lc[2] + side * yaw.cos() * 1.0];
            let Some(f) = w.scene.sim.as_ref().unwrap().world.floor_below(b, 2.0) else { continue };
            place(w, [b[0], f + 0.4, b[2]], lc);
            let mut climbing = false;
            for _ in 0..300 {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.bond.substate == nf_game::player::SS_CLIMB { climbing = true; break; }
                let p = sim.bond.pos;
                let err = wrap_pi((lc[0] - p[0]).atan2(lc[2] - p[2]) - sim.bond.yaw);
                w.tick(Input { turn: err.clamp(-0.15, 0.15), forward: if err.abs() > 0.3 { 0.2 } else { 1.0 }, ..Default::default() }, false);
            }
            if !climbing {
                // The foot itself as a start: a ladder's lower end is where its climb down lands
                // (the sweep does not drive the descent; the foot stand is the one placement).
                place(w, [b[0], f + 0.4, b[2]], c);
                if walk_from_here(w, k).is_some() { return Some(lc); }
                continue;
            }
            for _ in 0..1500 {
                if w.scene.sim.as_ref().unwrap().bond.substate != nf_game::player::SS_CLIMB { break; }
                w.tick(Input { forward: 1.0, ..Default::default() }, false);
            }
            for _ in 0..30 { w.tick(Input { forward: 1.0, ..Default::default() }, false); }
            if walk_from_here(w, k).is_some() { return Some(lc); }
            // And the foot as a start (the ladder may lead down to the pickup's floor).
            place(w, [b[0], f + 0.4, b[2]], c);
            if walk_from_here(w, k).is_some() { return Some(lc); }
        }
    }
    None
}

/// From where Bond stands: the straight steer, then the grid route seeded at his stand.
fn walk_from_here(w: &mut Walker, k: usize) -> Option<()> {
    let c = w.scene.sim.as_ref().unwrap().drones.pickups[k].centre();
    if reached(w.scene.sim.as_ref().unwrap(), k) { return Some(()); }
    let mut used = Vec::new();
    if steer_to(w, Some(k), c, 0.0, 900, &mut used).0 { return Some(()); }
    let pos = w.scene.sim.as_ref().unwrap().bond.pos;
    if let (Some(rt), _) = grid_search(w.scene.sim.as_ref().unwrap(), k, Some(pos)) {
        let mut used = Vec::new();
        for wp in rt.iter().skip(1).step_by(3) {
            if steer_to(w, Some(k), *wp, 0.35, 600, &mut used).0 { return Some(()); }
        }
        if steer_to(w, Some(k), c, 0.0, 600, &mut used).0 { return Some(()); }
    }
    None
}

/// Triage aid for a MISS with a take spot: walk BACK from the spot to the nearest nav nodes (any
/// floor within 3 u), then forward again from the node that was reached to the spot. `back`: a
/// node Bond walks to from the spot (none = the spot is cut off from the nav web on foot — a
/// crate top, a sealed room, a ledge); `fwd`: the walk from that node returns to the spot.
fn spot_link(w: &mut Walker, spot: [f32; 3]) -> String {
    let mut nodes: Vec<[f32; 3]> = nav_nodes(w.scene.sim.as_ref().unwrap()).into_iter()
        .filter(|n| (n[1] - 0.4 - spot[1]).abs() < 3.0 && dxz(*n, spot) < 30.0).collect();
    nodes.sort_by(|a, b| dxz(*a, spot).total_cmp(&dxz(*b, spot)));
    for n in nodes.iter().take(4) {
        place(w, spot, *n);
        let mut used = Vec::new();
        let (_, d, _) = steer_to(w, None, *n, 0.6, 1500, &mut used);
        let sim = w.scene.sim.as_ref().unwrap();
        if d < 0.6 && (sim.bond.pos[1] - 1.1 - (n[1] - 0.4)).abs() < 1.0 {
            let mut used = Vec::new();
            let (_, d2, _) = steer_to(w, None, spot, 0.3, 1500, &mut used);
            let up = w.scene.sim.as_ref().unwrap().bond.pos[1] - 1.1;
            return format!(" back->({:.1},{:.1},{:.1}) fwd {}", n[0], n[1], n[2], d2 < 0.3 && (up - spot[1]).abs() < 0.5);
        }
    }
    " back none".to_string()
}

/// Triage aid for a MISS: the first static instance each ray from Bond's end stand toward the
/// pickup meets (knee, waist, chest, head lanes), named from the disc's own records — object
/// name, placement kind and flags, and whether it is CEL geometry (placement 0x8000 [M
/// parsemap_block_map_data_static 0x1d1998:92]).
fn name_hits(level: &nf_formats::level::Level, inst: &[nf_formats::level::Instance], from: [f32; 3], to: [f32; 3]) -> String {
    let mut out = String::new();
    for dy in [-0.8f32, -0.4, 0.0, 0.4] {
        let a = [from[0], from[1] + dy, from[2]];
        let b = [to[0], to[1] + dy.max(-0.3), to[2]];
        let mut best: Option<(f32, usize)> = None;
        for (k, i) in inst.iter().enumerate() {
            let ob = &level.maps[i.map].objects[i.object];
            let Some(c) = &ob.collision else { continue };
            for t in c.triangles() {
                let wt = t.map(|p| nf_formats::placement::transform(&i.matrix, p));
                if let Some(f) = nf_game::world::seg_tri_t(a, b, &wt) {
                    if best.map_or(true, |(bf, _)| f < bf) { best = Some((f, k)); }
                }
            }
        }
        match best {
            Some((f, k)) => {
                let i = &inst[k];
                let ob = &level.maps[i.map].objects[i.object];
                out.push_str(&format!(" | dy{dy:+.1}: {:.2} u '{}' plc {:#x} glist {:#x}", f * dxz(a, b), ob.name, i.placement.flags, ob.flags));
            }
            None => out.push_str(&format!(" | dy{dy:+.1}: clear")),
        }
    }
    out
}

fn load_level(iso: &str, level: u32) -> nf_formats::level::Level {
    let img = nf_formats::iso9660::IsoImage::open(iso).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let name = format!("{level:08x}.bin");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)).unwrap();
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    nf_formats::level::Level::parse(&data).unwrap()
}

/// The nearest static triangles to Bond's capsule, and dynamic boxes, for a MISS line.
fn blockers(sim: &nf_view::scene::Sim) -> String {
    let p = sim.bond.pos;
    let mut s = String::new();
    for d in sim.drones.doors.list.iter().filter(|d| dxz(d.pos, p) < 3.0) {
        s.push_str(&format!(" door({:.1},{:.1},{:.1}) pos01 {:.2} lock {}", d.pos[0], d.pos[1], d.pos[2], d.pos01, d.ch_lock));
    }
    for (i, d) in sim.drones.list.iter().enumerate().filter(|(_, d)| d.present && dxz(d.pos, p) < 2.0) {
        s.push_str(&format!(" drone#{i}({:.1},{:.1},{:.1})", d.pos[0], d.pos[1], d.pos[2]));
    }
    s
}

/// The sweep for one level: every live placed pickup. Returns the misses.
pub fn sweep_level(iso: &str, level: u32) -> Option<Vec<Miss>> {
    let scene = nf_view::load(iso, &format!("{level:08x}.bin"), false).ok()?;
    if scene.start.is_none() { return None; }
    let mut w = Walker::new(scene);
    w.stop_on_exit = false;
    for _ in 0..10 { w.tick(Input::default(), false); }
    let n = w.scene.sim.as_ref().unwrap().drones.pickups.len();
    let mut misses = Vec::new();
    let mut count = 0;
    for k in 0..n {
        let (at, kind, weapon, skip) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = &sim.drones.pickups[k];
            (p.centre(), p.kind, p.weapon, p.gone || p.state == 2 || p.from.is_some())
        };
        if skip { continue; }
        count += 1;
        let starts = start_nodes(w.scene.sim.as_ref().unwrap(), at);
        // Up to four start nodes: a miss from one side of a wall is not a miss.
        let mut last: Option<Miss> = None;
        if starts.is_empty() {
            last = Some(Miss { k, kind, weapon, at, from: at, end: at, best: f32::MAX, why: "no nav node within 6 u of its height".into() });
        }
        for &(node, los) in starts.iter().take(4) {
            place(&mut w, node, at);
            let (ok, best, ticks) = steer(&mut w, k, los, 1500);
            if ok { last = None; break; }
            let sim = w.scene.sim.as_ref().unwrap();
            let end = sim.bond.pos;
            let why = format!("los_node {los} ticks {ticks} cel {:?}{}", sim.bond.cel, blockers(sim));
            if last.as_ref().map_or(true, |m| best < m.best) {
                last = Some(Miss { k, kind, weapon, at, from: node, end, best, why });
            }
        }
        if last.is_some() {
            let (route, why) = grid_search(w.scene.sim.as_ref().unwrap(), k, None);
            match route {
                Some(rt) if walk_grid(&mut w, k, &rt) => { eprintln!("[reach] {level:08x} #{k}: reached on the grid route ({} cells)", rt.len()); last = None; }
                Some(rt) => { if let Some(m) = last.as_mut() { m.why.push_str(&format!(" grid route {} cells, walk failed", rt.len())); } }
                None => { if let Some(m) = last.as_mut() { m.why.push_str(&format!(" NO grid route{why}")); } }
            }
            if last.is_some() {
                if let Some(lc) = ladder_leg(&mut w, k) {
                    eprintln!("[reach] {level:08x} #{k}: reached over the ladder at ({:.1},{:.1},{:.1})", lc[0], lc[1], lc[2]);
                    last = None;
                }
            }
        }
        if let Some(mut m) = last {
            let spot = take_spot(&mut w, k);
            let link = spot.map(|f| spot_link(&mut w, f)).unwrap_or_default();
            m.why = format!("{} take_spot {:?}{link}", m.why, spot);
            misses.push(m);
        }
    }
    if !misses.is_empty() {
        let lv = load_level(iso, level);
        let inst = lv.instances(true).0;
        for m in misses.iter_mut() {
            let h = name_hits(&lv, &inst, m.end, m.at);
            m.why.push_str(&h);
        }
    }
    eprintln!("[reach] {level:08x}: {count} pickups, {} unreached", misses.len());
    for m in &misses {
        eprintln!("[reach] MISS {level:08x} #{} kind {} weapon {:#x} at ({:.2},{:.2},{:.2}) from ({:.2},{:.2},{:.2}) end ({:.2},{:.2},{:.2}) best {:.2} {}",
            m.k, m.kind, m.weapon, m.at[0], m.at[1], m.at[2], m.from[0], m.from[1], m.from[2], m.end[0], m.end[1], m.end[2], m.best, m.why);
    }
    Some(misses)
}

fn run(level: u32) {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    if let Ok(only) = std::env::var("NF_REACH_LEVELS") {
        if !only.split(',').any(|s| u32::from_str_radix(s.trim(), 16).ok() == Some(level)) { return; }
    }
    assert!(nf_view::dev::LEVELS.contains(&level));
    let Some(misses) = sweep_level(&iso, level) else { eprintln!("[reach] {level:08x}: no campaign start"); return };
    let unexplained: Vec<&Miss> = misses.iter().filter(|m| !MISS_OK.iter().any(|(l, at, _)| *l == level
        && dxz(*at, m.at) < 0.2 && (at[1] - m.at[1]).abs() < 0.3)).collect();
    assert!(unexplained.is_empty(), "{level:08x}: {} unreached pickups outside the traced list: {:#?}", unexplained.len(), unexplained);
}

/// The pickups the sweep does not reach, each with what blocks it. Every static blocker named
/// here is CEL geometry (placement flags 0x8000, the only static set the game's pick builds [M
/// parsemap_block_map_data_static 0x1d1998:92]) and the dynamic ones are game objects in their
/// load state, so none is a port-only wall. "Not walked" = a route the sweep does not drive
/// (grapple, ladder descent, a mission event); whether the game reaches it that way is unproven.
const MISS_OK: &[(u32, [f32; 3], &str)] = &[
    (0x0700_0003, [21.47, 9.17, 11.59], "armour on the 9.1 walkway; no floor path in 25 u (lowest step in 1.92), the ladder at (34.3, 18.2) did not lead there: not walked"),
    (0x0700_0005, [72.79, 5.04, 78.71], "armour on the gorge slope; no nav node within 25 u, no floor path from any: not walked"),
    (0x0700_0005, [49.24, 49.06, -60.52], "ammo past the castle exit door Locked_Door_Top_I (LoadLevel door mesh, solid [M Trigger_Create 0x1b1720:92-96])"),
    (0x0700_0005, [48.07, 49.06, -59.26], "ammo past the castle exit door Locked_Door_Top_I (as above)"),
    (0x0700_0008, [172.72, 43.53, -116.23], "the briefcase in the safe (oBriefcase_closed #290, ch 0x4A on take): the safe door SS #205 opens on ch 0x88 [D docs/levels/07000008-audit.md]; no stand point within the take radius before that"),
    (0x0700_0008, [172.84, 42.75, -116.78], "the safe's rocket launcher (SafePickUps #286, as above)"),
    (0x0700_0008, [110.67, 27.55, -124.57], "inside the gondola carriage (script object 0x1C): its body blocks the take ray from outside"),
    (0x0700_000b, [2.53, 58.45, 42.34], "on the helicopter (script object 0x33); no nav node within 6 u of its height"),
    (0x0700_000c, [37.50, 0.22, -149.02], "low ground with no nav node within 25 u (lowest step in 2.42); grapple at (61.1, 12.3, -143.5): not walked"),
    (0x0700_000c, [33.86, -2.21, -146.57], "sealed floor pocket (80 cells, no way in on foot)"),
    (0x0700_000c, [76.01, 5.23, -149.83], "a ledge 4.5 over the floor beside it; grapples at (61.1, 12.3, -143.5) / (77.5, 12.1, -144.0): not walked"),
    (0x0700_000c, [53.98, 1.26, -173.46], "behind the secret door Door_secret1 (SimpleScript, opens on ch 0x08 [D placement 544 p3])"),
    (0x0700_000d, [192.82, 10.79, -272.03], "no floor way in within 25 u; grapples at (203.4, 14.6, -291.3) / (178.8, 16.3, -296.4): not walked"),
    (0x0700_000d, [180.64, 11.18, -208.82], "a ledge 2.7 over the floor beside it: not walked"),
    (0x0700_0012, [-33.66, 1.00, -4.45], "behind breakable pane office-jk-glass11 (kind 2): the live pane blocks the take ray and the capsule; shoot it first"),
    (0x0700_0014, [-28.03, 2.77, 6.86], "on a crate stack (1.8 then 2.7, CargoArea3); Bond's jump rises 0.81"),
    (0x0700_0014, [33.74, 3.67, -3.00], "on a crate stack (1.8 then 3.6, CargoArea2); Bond's jump rises 0.81"),
    (0x0700_0015, [12.18, 40.35, 0.18], "behind door #35 (10.4, 41.6, -1.1), locked on ch 0xC8 [D placement 357 p3]"),
    (0x0700_0015, [16.24, 40.52, -1.37], "behind door #35, locked on ch 0xC8 (as above)"),
    (0x0700_0015, [22.53, 40.35, 35.53], "behind door #43 (25.1, 41.6, 36.6), locked on ch 0xC8 [D placement 365 p3]"),
    (0x0700_0041, [20.65, 0.01, -39.33], "station floor with no nav node within 25 u (lowest step in 1.82); grapple at (2.1, 11.0, -47.4): not walked"),
];

macro_rules! level_tests {
    ($($name:ident = $lvl:expr;)*) => { $( #[test] fn $name() { run($lvl); } )* };
}

level_tests! {
    reach_07000001 = 0x0700_0001; reach_07000002 = 0x0700_0002; reach_07000003 = 0x0700_0003;
    reach_07000004 = 0x0700_0004; reach_07000005 = 0x0700_0005; reach_07000006 = 0x0700_0006;
    reach_07000007 = 0x0700_0007; reach_07000008 = 0x0700_0008; reach_07000009 = 0x0700_0009;
    reach_0700000a = 0x0700_000a; reach_0700000b = 0x0700_000b; reach_0700000c = 0x0700_000c;
    reach_0700000d = 0x0700_000d; reach_07000011 = 0x0700_0011; reach_07000012 = 0x0700_0012;
    reach_07000013 = 0x0700_0013; reach_07000014 = 0x0700_0014; reach_07000015 = 0x0700_0015;
    reach_07000016 = 0x0700_0016; reach_0700001b = 0x0700_001b; reach_07000041 = 0x0700_0041;
    reach_0700004a = 0x0700_004a;
}

/// The sister of the MISS_OK match: an unreached pickup that is not on the list (wrong level,
/// wrong place) is NOT explained, so the sweep's assert can fail.
#[test]
fn an_unlisted_miss_is_not_explained() {
    let explained = |level: u32, at: [f32; 3]| MISS_OK.iter().any(|(l, p, _)| *l == level && dxz(*p, at) < 0.2 && (p[1] - at[1]).abs() < 0.3);
    assert!(explained(0x0700_0014, [-28.03, 2.77, 6.86]), "a listed miss is explained");
    assert!(!explained(0x0700_0013, [-28.03, 2.77, 6.86]), "the same spot on another level is not");
    assert!(!explained(0x0700_0014, [-27.29, 0.87, 30.96]), "the booth armour is not on the list");
    assert!(!explained(0x0700_0014, [-28.03, 3.20, 6.86]), "a pickup 0.43 higher is another pickup");
}

/// 07000014's security-booth armour (placement #1019, PICKUP at (-27.29, 0.87, 30.96)): the
/// walkreal resupply wedge 1.82 u short of it, "a solid face 0.64 u east of Bond", is the booth's
/// own counter — SecurityCheckpointA15 (map 54 object 32, placement flags 0xC047: CEL geometry
/// [D]) raises the booth deck to 0.8 behind a 65-degree face (x -28.72 to -28.44) topped with
/// window bars at x -28.37 (z 30.40..31.96 and 34.43..36.00) [D the object's triangles]. The deck's
/// way in is the bar-free gap at z 31.96..34.43, where the floor ramps 0.2 → 0.6 → 0.8 and the nav
/// web runs node 118 (-29.24, 0.4, 33.20) → 121 (-28.00, 1.2, 33.21) → 122 (-27.31, 1.2, 31.36)
/// [D Default Navigation]. Walked through the gap the armour is taken; the straight line across the
/// counter stops at the face (the sister) — the wedge was the walk's line, not a port wall.
#[test]
fn the_booth_armour_is_taken_through_the_deck_gap() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let mut w = Walker::new(scene);
    w.stop_on_exit = false;
    for _ in 0..10 { w.tick(Input::default(), false); }
    let k = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .position(|p| p.kind == 3 && dxz(p.centre(), [-27.285, 0.867, 30.958]) < 0.05)
        .expect("the booth armour");
    // The sister first: the straight line from the corridor across the counter.
    place(&mut w, [-29.6, 0.4, 30.96], [-27.285, 0.867, 30.958]);
    for _ in 0..300 { w.tick(Input { forward: 1.0, ..Default::default() }, false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!reached(sim, k), "the counter keeps Bond out of take reach");
        assert!(sim.bond.pos[0] < -28.4, "stopped at the counter face: bond {:?}", sim.bond.pos);
    }
    // The gap: node 118 → (just inside node 121's lane) → the armour.
    place(&mut w, [-29.235344, 0.4, 33.204525], [-27.75, 1.2, 33.205353]);
    let mut used = Vec::new();
    let (_, d, _) = steer_to(&mut w, Some(k), [-27.75, 1.2, 33.205353], 0.15, 900, &mut used);
    assert!(d < 0.15, "onto the deck through the gap: bond {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    assert!(w.scene.sim.as_ref().unwrap().bond.pos[1] - 1.1 > 0.3, "up on the deck, not the corridor floor");
    let target = w.scene.sim.as_ref().unwrap().drones.pickups[k].centre();
    let (ok, _, _) = steer_to(&mut w, Some(k), target, 0.0, 900, &mut used);
    assert!(ok, "the armour is taken from the deck");
}
