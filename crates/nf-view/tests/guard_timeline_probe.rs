//! Scratch probe (guard-first-shot-timeline): 07000001's guard layout, the real walk-in, and the
//! guard's per-tick sight timeline. Also checks the stunner's availability on 01.

use nf_game::player::Input;

#[path = "common/walker_b2.rs"]
mod walker_b2;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }


#[test]
#[ignore = "scratch probe: run by hand with --ignored"]
#[test]
#[ignore = "scratch probe: run by hand with --ignored"]
fn probe_01_disc_volumes() {
    use nf_formats::{disc, iso9660::IsoImage, level::Level};
    let iso_path = iso();
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000001.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let mut counts = std::collections::BTreeMap::new();
    for m in level.maps.iter() {
        for p in m.placements.iter() {
            if matches!(p.flags, 0xf5 | 0xf8) { *counts.entry(p.flags).or_insert(0) += 1; }
        }
    }
    eprintln!("[dvol] 07000001 0xf5/0xf8 placements: {:?}", counts);
    for m in level.maps.iter() {
        for p in m.placements.iter() {
            if matches!(p.flags, 0xf5 | 0xf8) {
                eprintln!("[dvol] flags {:#x} map {} idx {} pos ({:.1},{:.1},{:.1}) params {:?}",
                    p.flags, m as *const _ as usize % 1000, p.index, p.translation[0], p.translation[1], p.translation[2],
                    p.params.iter().take(4).collect::<Vec<_>>());
            }
        }
    }
}

#[test]
#[ignore = "scratch probe: run by hand with --ignored"]
fn probe_01_volumes() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    // The level's AI volumes as the port built them.
    let vols = { let sim = scene.sim.as_ref().unwrap(); sim.volumes.clone() };
    eprintln!("[vol] port built {} ai volumes", vols.len());
    for (i, v) in vols.iter().enumerate() {
        eprintln!("[vol] {i}: kind {} factor {:.2} pos ({:.1},{:.1},{:.1}) half ({:.1},{:.1},{:.1})",
            v.kind, v.factor, v.m[3][0], v.m[3][1], v.m[3][2], v.half[0], v.half[1], v.half[2]);
    }
    // Light at Bond's path points.
    for p in [[-2.78, 7.0, -12.40], [-8.49, 7.0, -9.78], [-7.3, 7.0, -10.5], [-5.0, 7.0, -4.0], [-4.9, 7.0, -0.1]] {
        eprintln!("[vol] light at {:?} = {:.3}", p, nf_game::aivolume::visibility_factor(&vols, p, false));
    }
    // The guard's sight params.
    let sim = scene.sim.as_ref().unwrap();
    let dr = &sim.drones.list[0];
    eprintln!("[vol] guard0 sight_range {:.2} half_fov {:.3} light {:.2} vis at bond {:.3} dist {:.1}",
        dr.sight_range, dr.half_fov, dr.light, dr.visibility(), dr.dist);
}

#[test]
#[ignore = "scratch probe: run by hand with --ignored"]
#[test]
#[ignore = "scratch probe: run by hand with --ignored"]
fn probe_01_guard0_props() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    for _ in 0..2200 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let dr = &sim.drones.list[0];
    eprintln!("[props] guard0 skin {:?} alert {:.2} props words {:?}", dr.skin, dr.alert, dr.props);
    eprintln!("[props] prop 0x20 = {} prop 0x51 = {} prop 0x3c = {} prop 0x40 = {} mode_normal {} mode_alert {}",
        sim.props.get(0x20, &dr.props), sim.props.get(0x51, &dr.props), sim.props.get(0x3c, &dr.props),
        sim.props.get(0x40, &dr.props), dr.mode_normal, dr.mode_alert);
}

#[test]
#[ignore = "scratch probe: run by hand with --ignored"]
#[test]
#[ignore = "scratch probe: run by hand with --ignored"]
fn probe_01_guard0_placement() {
    use nf_formats::{disc, iso9660::IsoImage, level::Level};
    let iso_path = iso();
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000001.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let mi = level.main_map().unwrap();
    let m = &level.maps[mi];
    let mut shown = 0;
    for (i, p) in m.placements.iter().enumerate() {
        if p.flags & 0xff == 0x0f {
            eprintln!("[plc] placement {i} flags {:#x} params {:?}", p.flags, p.params);
            shown += 1;
            if shown >= 3 { break; }
        }
    }
}

#[test]
#[ignore = "scratch probe: run by hand with --ignored"]
fn probe_01_intro_timeline() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    let gi = 0usize;
    let mut last = (0xffu8, None, false, false);
    for t in 0..2600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let dr = &sim.drones.list[gi];
        let cur = (dr.state, dr.anim.script, dr.los, dr.seen);
        if cur != last {
            eprintln!("[tl] t={t} state {:#x} script {:?} anim {}.{}/{} los {} seen {} enabled {} shots {}",
                dr.state, dr.anim.script, dr.anim.state, dr.anim.class, dr.anim.time, dr.los, dr.seen, dr.enabled, dr.shots.len());
            last = cur;
        }
        if !dr.shots.is_empty() && cur == last && t > 2145 { break; }
    }
}

#[test]
#[ignore = "scratch probe: run by hand with --ignored"]
fn probe_01_walk_in() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    for t in 0..4000u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if !scene.sim.as_ref().unwrap().disabled { eprintln!("[01] free at t={t} bond at {:?}", scene.sim.as_ref().unwrap().bond.pos); break; }
    }
    // The guard enables with the intro tail; give his start script time to settle.
    for _ in 0..300 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    eprintln!("[01] owned weapons: {:?}", scene.sim.as_ref().unwrap().drones.inventory.owned);
    // Guard 0: the first guard (llist order = placement order).
    let gi = {
        let sim = scene.sim.as_ref().unwrap();
        *sim.drones.llist.iter().find(|&&i| sim.drones.list[i].present && sim.drones.list[i].team == 1 && sim.drones.list[i].weapon < 0x47)
            .expect("a team-1 guard on 07000001")
    };
    {
        let sim = scene.sim.as_ref().unwrap();
        eprintln!("[01] guard {gi} state {:#x} pos {:?} yaw {:.2} enabled {}", sim.drones.list[gi].state, sim.drones.list[gi].pos, sim.drones.list[gi].yaw, sim.drones.list[gi].enabled);
    }
    // Wait for him to enable + his start script to finish (stand still, real ticks).
    let mut enabled_at = None;
    for t in 0..600 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if enabled_at.is_none() && sim.drones.list[gi].enabled { enabled_at = Some(t); }
        if t % 60 == 0 { eprintln!("[01] t={t} guard pos ({:.1},{:.1},{:.1}) yaw {:.2} state {:#x} enabled {}", sim.drones.list[gi].pos[0], sim.drones.list[gi].pos[1], sim.drones.list[gi].pos[2], sim.drones.list[gi].yaw, sim.drones.list[gi].state, sim.drones.list[gi].enabled); }
    }
    let (gpos, gyaw) = { let sim = scene.sim.as_ref().unwrap(); (sim.drones.list[gi].pos, sim.drones.list[gi].yaw) };
    eprintln!("[01] after warm: guard {gi} at ({:.2},{:.2},{:.2}) yaw {:.2} (fwd {:.2},{:.2}) state {:#x}", gpos[0], gpos[1], gpos[2], gyaw, gyaw.sin(), gyaw.cos(), scene.sim.as_ref().unwrap().drones.list[gi].state);
    // Now walk Bond forward (stick) — the real walk-in toward the guard.
    let mut los_tick = None; let mut ack_tick = None; let mut shot_tick = None;
    let mut last_state = 0xff;
    for t in 0..400u32 {
        // steer toward the guard, walking speed
        let (turn, fwd) = {
            let sim = scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let dx = gpos[0] - bp[0]; let dz = gpos[2] - bp[2];
            let desired = dx.atan2(dz);
            let err = walker_b2::wrap_pi(desired - sim.bond.yaw);
            (err.clamp(-0.08, 0.08), if dxz2(bp, gpos) > 3.0 { 1.0 } else { 0.0 })
        };
        scene.sim.as_mut().unwrap().step_bond(Input { forward: fwd, turn, ..Default::default() }, false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let dr = &sim.drones.list[gi];
        if los_tick.is_none() && dr.los { los_tick = Some(t); }
        if ack_tick.is_none() && dr.seen { ack_tick = Some(t); }
        if !dr.shots.is_empty() && shot_tick.is_none() { shot_tick = Some(t); }
        if dr.state != last_state {
            eprintln!("[01] t={t} guard state {:#x} (was {last_state:#x}) los {} seen {} alert {:.2} dist {:.1} anim {}.{} cls {} firing {}",
                dr.state, last_state, dr.los, dr.seen, dr.alert, dr.dist, dr.anim.state, dr.anim.class, dr.firing);
            last_state = dr.state;
        }
        if shot_tick.is_some() { break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    let dr = &sim.drones.list[gi];
    eprintln!("[01] RESULT los {los_tick:?} ack {ack_tick:?} shot {shot_tick:?} final state {:#x} bond at {:?}", dr.state, sim.bond.pos);
    eprintln!("[01] shots {:?}", dr.shots);
}

fn dxz2(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0]-b[0]).powi(2) + (a[2]-b[2]).powi(2)).sqrt() }
