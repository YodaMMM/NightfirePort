//! drone-bump-collision-12 probe (assertion-free, run with --ignored): WHO reaches the
//! 07000008 GuardRoom / safe-bolt stands in the port sim, and what routes do they hold?
//! The -11 open question: do the disc's kind-8 guest routes actually reach the GuardRoom
//! (then the Jointy shove at the watch stand is game-true and the tests re-derive), or does
//! the port's route walk overshoot into it (then the walk is the fix)?
//! Route.start/dest/target are world coords resolved from the disc route params [D].

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

const BOLT_STAND_134: [f32; 3] = [162.2365, 43.33, -115.64];
const BOLT_STAND_135: [f32; 3] = [162.2368, 42.15, -115.64];
const ROOM: (f32, f32, f32, f32, f32, f32) = (150.0, 168.0, 40.0, 48.0, -124.0, -110.0);

fn in_room(p: [f32; 3]) -> bool {
    p[0] >= ROOM.0 && p[0] <= ROOM.1 && p[1] >= ROOM.2 && p[1] <= ROOM.3 && p[2] >= ROOM.4 && p[2] <= ROOM.5
}

#[test]
#[ignore]
fn probe_guest_routes_07000008() {
    let Some(iso) = iso() else { eprintln!("no NF_ISO"); return };
    let mut scene = nf_view::load(&iso, "07000008.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    eprintln!("=== initial drones (present, with routes) ===");
    for (i, dr) in sim.drones.list.iter().enumerate() {
        if !dr.present { continue; }
        let near_room = in_room(dr.pos);
        eprintln!("[t0] #{i} dtype {} team {} state {:#x} mode {} pos ({:.2},{:.2},{:.2}) route {} in_room {}",
            dr.dtype, dr.team, dr.state, dr.mode, dr.pos[0], dr.pos[1], dr.pos[2],
            dr.route.as_ref().map(|r| format!("path {} nodes {} idx {} mode {:?}", r.path, r.nodes.len(), r.index, r.mode)).unwrap_or("none".into()),
            near_room);
        if let Some(r) = &dr.route {
            eprintln!("      route start ({:.2},{:.2},{:.2}) dest ({:.2},{:.2},{:.2})",
                r.start[0], r.start[1], r.start[2], r.dest[0], r.dest[1], r.dest[2]);
        }
    }
    eprintln!("=== sampling 600 ticks ===");
    let mut ever_near: std::collections::HashSet<usize> = Default::default();
    for t in 0..600 {
        sim.step_bond(Input::default(), false, false, false);
        if t % 60 == 0 {
            for (i, dr) in sim.drones.list.iter().enumerate() {
                if !dr.present { continue; }
                let d134 = dxz(dr.pos, BOLT_STAND_134);
                let d135 = dxz(dr.pos, BOLT_STAND_135);
                if in_room(dr.pos) || d134 < 3.0 || d135 < 3.0 {
                    if d134 < 3.0 || d135 < 3.0 { ever_near.insert(i); }
                    eprintln!("[t{t}] #{i} dtype {} team {} state {:#x} pos ({:.2},{:.2},{:.2}) d134 {d134:.2} d135 {d135:.2}",
                        dr.dtype, dr.team, dr.state, dr.pos[0], dr.pos[1], dr.pos[2]);
                }
            }
        }
    }
    eprintln!("=== ever within 3.0 of a bolt stand: {:?} ===", ever_near);
    eprintln!("=== final drone positions ===");
    for (i, dr) in sim.drones.list.iter().enumerate() {
        if !dr.present { continue; }
        eprintln!("[end] #{i} dtype {} team {} state {:#x} pos ({:.2},{:.2},{:.2}) in_room {}",
            dr.dtype, dr.team, dr.state, dr.pos[0], dr.pos[1], dr.pos[2], in_room(dr.pos));
    }
    eprintln!("=== teleport Bond to the bolt-135 stand, tick 400 ===");
    sim.bond.pos = BOLT_STAND_135;
    for t in 0..400 {
        sim.step_bond(Input::default(), false, false, false);
        if t % 20 == 0 {
            eprintln!("[s{t}] bond ({:.2},{:.2},{:.2})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
            for (i, dr) in sim.drones.list.iter().enumerate() {
                if !dr.present { continue; }
                let d = dxz(dr.pos, BOLT_STAND_135);
                if d < 5.0 {
                    eprintln!("[s{t}] #{i} dtype {} team {} state {:#x} pos ({:.2},{:.2},{:.2}) d {d:.2}",
                        dr.dtype, dr.team, dr.state, dr.pos[0], dr.pos[1], dr.pos[2]);
                }
            }
        }
    }
    eprintln!("=== spots-ray discrimination (static vs dynamic blocker) ===");
    let mut scene2 = nf_view::load(&iso, "07000008.bin", false).expect("load");
    let sim2 = scene2.sim.as_mut().unwrap();
    for _ in 0..3000u32 {
        if sim2.disabled { sim2.drones.scripts.skip_request = true; }
        sim2.step_bond(Input::default(), false, false, false);
        if !sim2.disabled { break; }
    }
    sim2.bond = nf_game::player::Bond::new([162.24, 43.33, -115.64], 3.142);
    sim2.bond.pitch = -0.063;
    for _ in 0..60u32 { sim2.step_bond(Input::default(), false, false, false); }
    let eye = sim2.bond.head();
    let bolt_top: [f32; 3] = [162.2365, 43.875843, -117.14455];
    let d = [bolt_top[0] - eye[0], bolt_top[1] - eye[1], bolt_top[2] - eye[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(0.001);
    let to2 = [bolt_top[0] - d[0] / l * 0.4, bolt_top[1] - d[1] / l * 0.4, bolt_top[2] - d[2] / l * 0.4];
    eprintln!("[ray] eye {eye:?} to2 {to2:?}");
    eprintln!("[ray] segment_hit (all): {:?}", sim2.world.segment_hit(eye, to2));
    eprintln!("[ray] segment_hit_static: {:?}", sim2.world.segment_hit_static(eye, to2));
    eprintln!("[ray] drones near the ray corridor:");
    for (i, dr) in sim2.drones.list.iter().enumerate() {
        if !dr.present { continue; }
        let dxy = dxz(dr.pos, [162.28, 0.0, -115.9]);
        if dxy < 2.5 {
            eprintln!("[ray] #{i} dtype {} state {:#x} pos ({:.2},{:.2},{:.2}) dxz {dxy:.2}",
                dr.dtype, dr.state, dr.pos[0], dr.pos[1], dr.pos[2]);
        }
    }
    eprintln!("[ray] doors near z -116:");
    for (i, door) in sim2.drones.doors.list.iter().enumerate() {
        if (door.pos[2] - (-116.0)).abs() < 3.0 && (door.pos[0] - 162.0).abs() < 4.0 {
            eprintln!("[ray] door #{i} pos ({:.2},{:.2},{:.2}) state {:#x} ch_open {}", door.pos[0], door.pos[1], door.pos[2], door.state, door.ch_open);
        }
    }
}

fn dxz(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

#[test]
#[ignore]
fn probe_ray_sensitivity() {
    let Some(iso) = iso() else { eprintln!("no NF_ISO"); return };
    let mut scene = nf_view::load(&iso, "07000008.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000u32 {
        if sim.disabled { sim.drones.scripts.skip_request = true; }
        sim.step_bond(Input::default(), false, false, false);
        if !sim.disabled { break; }
    }
    let eye0 = [162.31874f32, 44.11647, -115.05467];
    let bolt: [f32; 3] = [162.2365, 43.875843, -117.14455];
    for (dy, dz) in [(0.0f32, 0.0f32), (-0.05, 0.0), (-0.10, 0.0), (0.05, 0.0), (0.0, 0.10), (0.0, -0.10), (-0.05, 0.10), (0.10, 0.0)] {
        let eye = [eye0[0], eye0[1] + dy, eye0[2] + dz];
        let hit = sim.world.segment_hit(eye, bolt);
        eprintln!("[sens] dy {dy:+.2} dz {dz:+.2} -> {:?}", hit.map(|(t, _)| t));
    }
    eprintln!("[sens] bond pos after its own settle: {:?}", sim.bond.pos);
}
