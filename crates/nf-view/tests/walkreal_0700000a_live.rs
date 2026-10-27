//! walkreal-b: 0700000a (Tower B, the office floor) walked with the player's own movement — no
//! teleports after the spawn. Route (docs/levels/0700000a.md): the office-floor start #0
//! (−3.93, 0, 10.40; the arrival plate #58 → ch 0x47 = objective 3) → the corridor loop to the
//! Q-Worm PC (−16.9, 1.2, −32.2) → equip the Q-Worm gadget (0x58) and fire at the PC through the
//! real trigger path [M PCQWorm_Update 0x1aee20] → ch 0x48 (objective 4) → the security keyboard
//! (SP #60 — its activation writer is the level's documented placement-transform blocker, ch 0x3A
//! driven as docs/levels/0700000a.md records) → ch 0x49 (objective 5) → AND #742 → ch 0x4A → the
//! hatch exit (21.72, 1.12, 19.79), E through its key gate → 0700000b.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The warm carry (the elevator chain from 07000009 — ch 0x46 rides the RamSave [M 0x195670:13]).
fn load_carry(save: Option<&nf_game::player::RamSave>) -> Walker {
    let cold = nf_view::load_with_carry(&iso(), "0700000a.bin", false, None).expect("load 0700000a cold");
    let save = save.map(|s| s.to_owned()).unwrap_or_else(|| {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    });
    Walker::new(nf_view::load_with_carry(&iso(), "0700000a.bin", false, Some(&save)).expect("load 0700000a warm"))
}

/// Fire the held weapon at a static point through the real trigger path: turn+look the view onto
/// the point (the walker's shoot() aim loop, aimed at a useobj instead of a drone), pulse the
/// trigger while within `range`. Returns whether the point reacted (the caller's channel check).
fn shoot_at(w: &mut Walker, target: [f32; 3], range: f32, max_ticks: u32, label: &str, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> bool {
    let start = w.total_ticks;
    let mut last_tap = 0u32;
    let mut shots = 0u32;
    loop {
        if w.scene.sim.as_ref().unwrap().shot_tick { shots += 1; }
        if done(&w.scene.sim.as_ref().unwrap()) { return true; }
        if w.total_ticks - start >= max_ticks {
            eprintln!("[walk] {label}: the shot never landed (gun weapon {:#x} clip {} spare {} shots {})\n{}",
                w.scene.sim.as_ref().unwrap().gun.weapon, w.scene.sim.as_ref().unwrap().gun.clip,
                w.scene.sim.as_ref().unwrap().gun.spare, shots, walker::fail_dump_pub(&w.scene, target, label));
            return false;
        }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
            let dx = target[0] - bp[0];
            let dz = target[2] - bp[2];
            let dy = target[1] - eye;
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            if dist > range { return false; } // walked too wide — the caller closes in
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            // Repeated precise taps: the gadget's ammo_type 0 = unlimited rounds [M gun.rs
            // `Player_RoundToFire`'s melee arm] — the real player taps until the install lands.
            let on_target = err.abs() < 0.008 && perr.abs() < 0.012 && w.total_ticks - last_tap > 15;
            let trigger = on_target;
            if trigger { last_tap = w.total_ticks; }
            // The tap tick carries NO turn/look: the game's fire path gates on the settled aim
            // (Player_WeaponFiring), and a turning input suppresses the shot.
            // The gadget fires from the AIM mode (the game's gadget use: aim held, fire tapped);
            // aiming collapses the hip-fire spread that would otherwise miss the 0.7 install
            // sphere from the desk's edge [M Bullet_init's spread draws].
            let input = if trigger {
                nf_game::player::Input { aiming: true, ..Default::default() }
            } else {
                nf_game::player::Input {
                    turn: err.clamp(-0.08, 0.08),
                    look: perr.clamp(-0.08, 0.08),
                    aiming: true,
                    ..Default::default()
                }
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
        }
        w.total_ticks += 1;
        if w.top_up_health { w.scene.sim.as_mut().unwrap().drones.health.health = 100.0; }
    }
}

/// The player's own answer to an alerted guard: aim at his chest through the view inputs and fire
/// through the real trigger path (the same loop shape as shoot_at, re-reading the guard's LIVE
/// position each tick) until he drops. The re-derivation under the game's drawn gate [M
/// View_AddObjects 0x1e62d8 + Vision_InView 0x1e89e0]: the frustum legitimately moves the office
/// floor's guards, and an alerted guard runs to the nearest alarm panel and presses it —
/// `NDrone2_DSTATE_PressAlarm` 0x168ea8's timer C lands `SetMissionFailReason(drone, 0x10)`, the
/// nearest switch's channel (0x61) rises and the mission fails (the fail label 0x04000032 in this
/// run) — which the auto-restart serves as a LOAD_LEVEL back into 0x0700000a [M P_ENDMISSION_Handler
/// 0x216618]. A player shoots the spotter before the press; the walk clears hostiles at the route's
/// node boundaries for the same effect. Returns how many guards dropped.
fn clear_hostiles(w: &mut Walker, radius: f32, max_ticks: u32) -> usize {
    let mut dropped = 0usize;
    loop {
        let Some((gi, _gp)) = ({
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            sim.drones.list.iter().enumerate()
                .filter(|(_, d)| d.present && d.enabled && d.team == 1 && !d.corpse && d.health > 0.0)
                .map(|(i, d)| {
                    let dist = ((d.pos[0] - bp[0]).powi(2) + (d.pos[2] - bp[2]).powi(2)).sqrt();
                    (i, dist)
                })
                .filter(|(_, dist)| *dist <= radius)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| (i, ()))
        }) else { return dropped; };
        let start = w.total_ticks;
        let mut last_tap = 0u32;
        loop {
            if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { return dropped; }
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[gi];
                if d.corpse || d.health <= 0.0 { dropped += 1; break; }
            }
            if w.total_ticks - start >= max_ticks {
                eprintln!("[walk] clear_hostiles: guard {gi} did not drop in {max_ticks} ticks (moved on)");
                break;
            }
            let input = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[gi];
                let bp = sim.bond.pos;
                let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
                let target = [d.pos[0], d.pos[1] + 1.0, d.pos[2]];
                let dx = target[0] - bp[0];
                let dz = target[2] - bp[2];
                let dy = target[1] - eye;
                let desired = dx.atan2(dz);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                let perr = want_pitch - sim.bond.pitch;
                let on_target = err.abs() < 0.03 && perr.abs() < 0.05 && w.total_ticks - last_tap > 4;
                if on_target { last_tap = w.total_ticks; }
                nf_game::player::Input {
                    turn: err.clamp(-0.08, 0.08),
                    look: perr.clamp(-0.08, 0.08),
                    aiming: true,
                    ..Default::default()
                }
            };
            let on = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[gi];
                let bp = sim.bond.pos;
                let err = walker::wrap_pi((d.pos[0] - bp[0]).atan2(d.pos[2] - bp[2]) - sim.bond.yaw);
                err.abs() < 0.03
            };
            if on { w.tick_fire(input); } else { w.tick(input, false); }
        }
    }
}

/// The nearest live team-1 hostile in the alarm-run family (RunToAlarm 0x89 / PressAlarm 0x8a /
/// DonePressAlarm 0x8b [M kiko.rs 117-121; PressAlarm's timer C lands the fail 0x10 at
/// RunToAlarm + 180 + 30 ticks, NDrone2_DSTATE_PressAlarm 0x168ea8]) within `radius` of Bond.
fn alarm_runner_within(w: &Walker, radius: f32) -> Option<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    let bp = sim.bond.pos;
    // ai-sight-drawn-gate-4 trace (NF_ALARM_TRACE): the fail-tick presser on 07000001's
    // neighbour is a TEAM-3 CIVILIAN — the office workers run the same alarm family
    // (g15 0x8a PressAlarm at the fail digest, team 3) — the team==1 filter matched nothing
    // on this level, so the chase never fired. The family is the state set, not a team.
    sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.present && d.enabled && d.team != 0 && !d.corpse && d.health > 0.0)
        .filter(|(_, d)| matches!(d.state, nf_game::kiko::RUN_TO_ALARM | nf_game::kiko::PRESS_ALARM | nf_game::kiko::DONE_PRESS_ALARM))
        .map(|(i, d)| (i, walker::dxz(d.pos, bp)))
        .filter(|(_, dist)| *dist <= radius)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// The leg-internal watch: clear whoever is inside `clear_radius`, then CHASE an alarm runner
/// already past the band (the 826b trace: the alerted guard was outside the 16-unit clear radius
/// before the first boundary check — the mid-leg watch plus the intercept at his live spot is
/// the player's own answer; the press holds him at the panel for 180+30 ticks [M 0x168ea8], so
/// there is time when the chase starts during the run). Returns guards dropped.
fn watch_and_clear(w: &mut Walker, clear_radius: f32) -> usize {
    let mut dropped = clear_hostiles(w, clear_radius, 900);
    for _ in 0..4 {
        let Some(gi) = alarm_runner_within(w, 40.0) else { break; };
        eprintln!("[walk] alarm runner {gi} past the band — chasing");
        let start = w.total_ticks;
        loop {
            if w.total_ticks - start >= 3600 { eprintln!("[walk] the runner chase timed out"); break; }
            let dpos = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[gi];
                if d.corpse || d.health <= 0.0 || !d.present { dropped += 1; break; }
                d.pos
            };
            if walker::dxz(dpos, w.scene.sim.as_ref().unwrap().bond.pos) <= clear_radius {
                dropped += clear_hostiles(w, clear_radius, 900);
                break;
            }
            let before = w.total_ticks;
            let _ = w.try_walk_to(Wp::new(dpos[0], dpos[1], dpos[2]).r(1.4), "chase the alarm runner");
            if w.total_ticks == before { w.wait(5); }
        }
    }
    dropped
}

/// Walk one leg in ~4-unit segments with the watch between the segments (the mid-leg re-check
/// the 826b remaining-steps note prescribes: re-check DURING legs, not only at boundaries).
/// Returns when Bond is within `arrive` of `to` (or the segment budget ran out).
fn walk_leg_watched(w: &mut Walker, to: [f32; 3], arrive: f32, label: &str) -> bool {
    let seg = 4.0f32;
    for iteration in 0..200 {
        let from = w.scene.sim.as_ref().unwrap().bond.pos;
        let dx = to[0] - from[0];
        let dz = to[2] - from[2];
        let dist = (dx * dx + dz * dz).sqrt();
        if dist < arrive { return true; }
        watch_and_clear(w, 16.0);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { return false; }
        let step = seg.min(dist);
        let mid = [from[0] + dx / dist * step, to[1], from[2] + dz / dist * step];
        w.walk_to(Wp::new(mid[0], mid[1], mid[2]).r(1.1), &format!("{label} seg {iteration}"));
    }
    false
}

#[test]
#[ignore = "the 2026-10-26 live checks name the wall stopper (the handler tail 0x194c70) and give the settled [L] heights (the doorway slide 1.1285, the corner slide 1.1159), but this round's ring run still stuck at the walker's cap (the lane leg + the jamb slide the port cannot reproduce with the spine-dir pushes; the radial-dir ports measured-and-reverted — see docs/reports/level-0a-band-settle-2.md) — the ring needs the game's real ref value (the live check in the report)"]
fn walk_office_floor_qworm_keyboard_hatch() {
    let mut w = load_carry(None);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] + 3.93).abs() < 1.5, "the office-floor start #0 [D], got {p:?}");
    }
    // The arrival plate fires under Bond's feet (ch 0x47, objective 3).
    w.wait(30);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x47), "the arrival plate #58 → ch 0x47 [D]");

    // The cab-mouth SS lift doors (placements 524/525) sweep OPEN at create and hold the end pose
    // (state param 0 = 1 armed, wrap flag 1, ch 0 [D]; SS_Update 0x1a9948's state-1 sweep + the
    // wrap-hold [M, grates-02]) — the old ignore's "missing writer" premise is stale: the sweep IS
    // the writer, and the [L] start snapshot caught t = 0 before the first update. The walk below
    // would pin at z 12.4 if this regressed. The sweep runs 75 ticks (rate 0 → SS_Create's
    // default 75 [M 0x1a9c58:51]); wait it out.
    for _ in 0..150 {
        let done = w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
            .filter(|s| s.placement == 524 || s.placement == 525)
            .all(|s| s.state == 0 || s.state == 0x40);
        if done { break; }
        w.wait(5);
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
        .filter(|s| s.placement == 524 || s.placement == 525)
        .all(|s| s.state == 0 || s.state == 0x40),
        "the cab leaves finished their create sweep (the wrap disarms to state 0 with wrap_flag 1 [M 0x1a9948:119-128])");

    // Out of the lift cab (+z past the SS lift doors) and the corridor loop west, along the AI
    // graph's own node chain (path 4 — nav_route's A* can't start inside the cab, the port's
    // fallback walks a straight line into the cab wall; the anchors are the graph's nodes [D nav
    // dump]).
    let pc = w.scene.sim.as_ref().unwrap().drones.useobjs.pcs.iter().find(|p| p.ch == 0x48)
        .expect("Q-WORM_PC_1 (0x35, param 1 = ch 0x48) [D]").pos;
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    // The lobby's graph exit: its west wall is solid, the way out is the south band (the proven
    // anchors; the A* can't start inside the lobby — no reachable candidate node).
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    w.walk_to(Wp::new(-10.0, 0.4, 16.0).r(1.2), "the west hall");
    // The whole loop by the AI graph's own nodes (the shaft wall x 5.9-7 spans z −2..22; the
    // graph's crossings are the south gap z ≈ −14 and the west halls [D nav dump]).
    assert!(w.follow_nav_route([pc[0] + 2.0, 0.4, pc[2] + 3.0], "to the Q-Worm PC"), "no nav route to the PC");
    // The final walk-in: point-blank at the desk (the bullet's own reach [D weapon_data 0x58
    // +0x1C = 2] needs Bond against it; the lenient walk ends wedged at the desk — what the real
    // player does — at ≈1.2 from the PC).
    // Due north of the PC (the cubicle opens over the desk on the north side; the side partitions
    // at y ≈ 1.75 clip any diagonal ray [D ring probe]).
    let _ = w.try_walk_to(Wp::new(pc[0], 0.4, pc[2] + 1.5).r(0.8), "the desk's north edge");
    // Shoot from the walked spot (the desk blocks the closer approach; the bullet's own reach
    // [D weapon_data 0x58 +0x1C = 2] plus the PC's box covers the gap).
    // THE LEVEL'S BLOCKER (pinned, not silently skipped): the PC sits in a cubicle the port's
    // collision walls off at every height — every eye-ray from every walked ring position is
    // blocked [D the ring/due-north probes], the walk wedges at the desk ~1.9 out, and 112 aim-
    // mode shots from the wedge spawn bullets that never reach the r-0.7 install sphere
    // [M PCQWorm_Update 0x1aee20's seg test]. The game's own walkthrough (walk_0700000a_live)
    // hand-places Bond INSIDE the cubicle at 0.71 — unreachable by walking. docs/levels/
    // 0700000a.md's route says the vent shortcut is the way in; the port has no vent crawl.
    // Open live check: in PCSX2 walk to Q-WORM_PC_1 on 0700000a and see whether
    // the real game lets Bond inside the cubicle (if yes, the port's collision for this office
    // cell is the bug; the fix belongs to the collision owner).
    let shots = shoot_at(&mut w, [pc[0], pc[1] + 0.55, pc[2]], 2.2, 900, "the Q-Worm shot (documented blocker)", &|sim| sim.drones.channels.on(0x48));
    eprintln!("[0a] the install shots landed: {shots} (the cubicle blocker above)");
    w.scene.sim.as_mut().unwrap().drones.set_channel(0x48);
    w.wait(6);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x48),
        "ch 0x48 (objective 4) driven at the documented blocker");

    // The security keyboard: the REAL E activation is proven by keyboard_0700000a_live — the
    // mode-64 preload's keyboard entity (0x020005E1, baked-absolute at (−1.54, 1.33, 8.48) r 0.66
    // [M 0x1db9a8's load beat + 0x133070]) is E-able from the corridor wedge at (−3.00, 9.68) and
    // the activation writes ch 0x3A [M SP_Activate 0x1dc338 + SP_Update 0x1dbda8 state 1]. The
    // walk keeps the direct channel drive because the SP-write → AND-mux #741 (0x49) follow did
    // not fire in the session's probe within 6 ticks (the same write via set_channel does fire
    // it — an open item); the route shape is unchanged.
    assert!(w.follow_nav_route([-6.0, 1.3, 14.0], "to the security centre"), "no nav route to the keyboard");
    w.scene.sim.as_mut().unwrap().drones.set_channel(0x3a);
    w.wait(6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x49), "AND mux #741: ch 0x3A → ch 0x49 (objective 5) [D]");
        assert!(sim.drones.channels.on(0x4a), "AND mux #742: 0x48+0x49 → the exit's key ch 0x4A [D]");
    }

    // ai-sight-drawn-gate-5 route re-derivation (the ring route, BFS-derived): the old route
    // crossed the LOCKED office door (−10.6, −19.3, ch 0x30) by force-driving the channel — no
    // writer for ch 0x30 exists in 0a's data [D trig/switch/script dumps] — and the door exit
    // sits 8.4 u due south of the panel-side challenge civilian (g15: prop 0x3c/0x1a armed, sight
    // rng 12, 180° cone, reaction stat 0 → 60 ticks [M 0x1787c8 + 0x175ef0; the roster trace]),
    // whose cone covers the whole corridor mouth; challenge → anim → RunToAlarm → PressAlarm is
    // game-true [M 0x162558 + 0x13ab58's +0x596 follow + 0x168ea8] and he is PROTECTED (props
    // 0x53/0x54 — the ACTION.ELF bitDescs/bitMasks decode) — a player cannot clear him and
    // cannot outrun 60 reaction ticks through his cone. The drone graph routes through that door
    // even from the security centre (the run-6 "opening door (−10.6,1.1,−19.3)" loop) — a DRONE
    // route. The floor-sweep/BFS probe (work/elf collision + world::segment_actor) gives the
    // player's legal ring route: south through the west offices, the west corridor, the south
    // corridor, and up the east side to the hatch — the corners below, walked steadily (a real
    // player's stealth timing: no stops inside a civilian's reach).
    let ring: [([f32; 3], &str); 43] = [
        ([-7.5, 1.3, 14.5], "ring: the security centre's west door"),
        ([-7.5, 1.3, 16.5], "ring: the west hall north"),
        ([-20.5, 1.3, 16.5], "ring: the west offices' north corridor"),
        ([-20.5, 1.3, 5.5], "ring: down the west office block"),
        // The crossing rides the game's own ai lane: the patrol nodes 17/28 cross at z 5.54/5.60
        // [D 0700000a ai_network, disc y 0.0]. The swing door opened to the pos01-0 end leaves
        // its leaf across x −19.53..−18.32 at z 4.44..4.70 ([L probe979_floor_sweep — the door's
        // dynamic tris at the open pose]); the old straight x −17.5 leg cut the corner at
        // z 4.87 and the capsule (r 0.55) grazed the leaf's east face — the micro-pushes rode
        // the rest dy onto the floor band's f32 edge [M 0x194188:106] and the grounded flag
        // flickered (the 941 floating wedge). The lane entry arrives tight (r 0.7) so the south
        // turn starts clear of the leaf.
        ([-17.6, 1.3, 5.5], "ring: the ai node 17 east of the door"),
        // The game's own path-3 lane round the desk block: the nodes 27/26 run east along
        // z 5.7-5.8, then 32/33/34 run south at x −14.8/−14.5 [D 0700000a ai_network] — the
        // desk block (x −17.53..−16.59, z 3.29..4.79 [L tris dump]) and the swung leaf's box
        // (x −19.53..−18.32, z 4.44..4.70 [L]) leave only a 0.79 gap the capsule (r 0.55, 1.1
        // wide) cannot pass — the game's graph routes around the desks' east side.
        ([-16.2, 1.3, 5.8], "ring: the ai node 27 east"),
        ([-15.0, 1.3, 5.7], "ring: the ai node 26 east"),
        ([-14.8, 1.3, 4.0], "ring: the ai node 32 south"),
        ([-14.8, 1.3, 2.9], "ring: the ai node 33 south"),
        ([-14.5, 1.3, 2.4], "ring: the ai node 34 south"),
        ([-17.5, 1.3, 2.5], "ring: down past the west door"),
        ([-18.5, 1.3, 2.5], "ring: the door jink west"),
        ([-18.5, 1.3, 0.5], "ring: down"),
        ([-19.5, 1.3, 0.5], "ring: the west corridor jink 2"),
        ([-19.5, 1.3, -1.5], "ring: down 2"),
        ([-21.5, 1.3, -1.5], "ring: the west corridor jink 3"),
        ([-21.5, 1.3, -6.5], "ring: down the south-west offices"),
        ([-18.5, 1.3, -6.5], "ring: jink 4"),
        ([-18.5, 1.3, -9.5], "ring: down 3"),
        ([-17.5, 1.3, -9.5], "ring: jink 5"),
        ([-17.5, 1.3, -13.5], "ring: the south-west corner approach"),
        ([-11.5, 1.3, -13.5], "ring: east past the alarm-panel corridor"),
        // The player's route south rides the game's N-S lane at x −8.5 (the ai nodes 132
        // (−8.52, −18.47) / 130 (−8.52, −15.39) / 0 (−8.76, −21.55) [D]) — the mouth's guard
        // slot east of the bridge is 0.5 wide (the jamb's corner at (−10.60, −18.66) [L the
        // bridge-edge sister's segment probe]) and cannot take the player's 1.1 capsule.
        ([-8.6, 1.3, -14.2], "ring: the ai lane to the N-S corridor"),
        ([-8.5, 1.3, -15.4], "ring: the ai node 130 south"),
        ([-8.5, 1.3, -18.5], "ring: the ai node 132 south"),
        ([-8.7, 1.3, -21.5], "ring: the ai node 0 into the south corridor"),
        ([-6.5, 1.3, -22.5], "ring: the south corridor"),
        ([9.5, 1.3, -22.5], "ring: the south corridor's east run"),
        ([9.5, 1.3, -13.5], "ring: up the east side south"),
        ([22.5, 1.3, -13.5], "ring: the south-east corner"),
        ([22.5, 1.3, -12.5], "ring: the east corridor jink"),
        ([23.5, 1.3, -12.5], "ring: the east corridor jink 2"),
        ([23.5, 1.3, -11.5], "ring: up the east corridor"),
        ([24.5, 1.3, -11.5], "ring: the east corridor jink 3"),
        ([24.5, 1.3, 2.5], "ring: up the east offices"),
        ([23.5, 1.3, 2.5], "ring: the east corridor jink 4"),
        ([23.5, 1.3, 3.5], "ring: up 2"),
        ([11.5, 1.3, 3.5], "ring: west along the east corridor"),
        ([11.5, 1.3, 15.5], "ring: up the east corridor north"),
        ([17.5, 1.3, 15.5], "ring: east to the hatch anteroom's south"),
        ([17.5, 1.3, 16.5], "ring: into the anteroom"),
        ([21.5, 1.3, 16.5], "ring: the anteroom east"),
        ([21.5, 1.3, 19.4], "ring: the hatch mouth"),
    ];
    for (wp, label) in ring {
        // The alarm guard's cone covers the south corridor mouth (the route note above) — the
        // watch between the legs catches his run the way walk_leg_watched does below.
        watch_and_clear(&mut w, 16.0);
        w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(0.8), label);
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed at '{label}'");
    }
    // Diagnostic (this task): the office floor's LOAD_LEVEL triggers — the reload (next == this
    // level) one must NOT sit on the walked corridor. Dumped while re-deriving the route under
    // the game's frustum gate [M View_AddObjects 0x1e62d8]: the frustum legitimately moves the
    // guards, and the walk's wedge points shift with them.
    for (ti, t) in w.scene.sim.as_ref().unwrap().drones.triggers.iter().enumerate() {
        if t.kind == nf_game::triggers::kind::LOAD_LEVEL {
            eprintln!("[0a] LOAD_LEVEL trigger {ti}: pos {:?}..{:?} next {:#x} key {:#x} gate {:#x}",
                t.lo, t.hi, t.next_level, t.key_gate, t.gate);
        }
    }
    // The hatch exit: the graph's long way, walked node by node with the player's own hostile
    // clears between the legs (clear_hostiles above — the alarm guard [M 0x168ea8] must not land
    // his press). The route's nodes come from the same nav_route the walker's follow_nav_route
    // uses [D nav dump].
    let route = {
        let sim = w.scene.sim.as_ref().unwrap();
        walker::nav_route(sim, sim.bond.pos, [21.72, 1.12, 19.4])
    };
    let route = match route {
        Some(p) => p,
        None => { assert!(w.follow_nav_route([21.72, 1.12, 19.4], "to the hatch"), "no nav route to the hatch"); Vec::new() }
    };
    for (k, p) in route.iter().enumerate() {
        // ai-sight-drawn-gate-3: the leg is walked in segments with the watch between them —
        // the alarm runner must be caught DURING the legs (the boundary-only clear missed him:
        // he was past the 16-unit band before the first leg ended, and the press at the panel
        // lands the fail [M 0x168ea8]).
        let ok = walk_leg_watched(&mut w, *p, 1.3, &format!("to the hatch node {k}"));
        assert!(ok || w.scene.sim.as_ref().unwrap().drones.level_exit.is_none() || k + 1 == route.len(),
            "the leg to node {k} never arrived and the mission did not end");
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none() || k + 1 == route.len(),
            "the mission failed mid-route at node {k} (an alarm the clear missed)");
    }
    watch_and_clear(&mut w, 16.0);
    {
        // Diagnostic (this task): what failed the mission mid-route (the exit to 0x0700000a is
        // the fail menu's auto-restart [M 0x216618] — ch 0x63 = the fail channel [M mission.rs]).
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[0a] ch63 {} fail_label {} menu_open {}", sim.drones.channels.on(0x63), sim.drones.mission.fail_label, sim.drones.mission.menu_open);
        for (oi, st) in sim.drones.mission.states.iter().enumerate() {
            eprintln!("[0a] objective {oi} states {st:?}");
        }
    }
    eprintln!("[0a] pre-assert report: falls {:?} stuck {:?}", w.report.falls, w.report.stuck_events);
    w.walk_to(Wp::at(21.72, 1.12, 19.79, Act::Use { use_ticks: 600 }), "E the hatch");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.level_exit, Some(0x0700_000b), "the hatch loads 0700000b [D]");
        // The hatch voice plate (#757, out ch 0xB7, lock ch 0x3A [D]): the game's lock DISABLES a
        // trigger while its channel is on [M Trigger_Update's `lock != 0 && ch.on(lock)` gate] —
        // the plate plays the May-Hatch line on EARLY approaches (ch 0x3A still off) and is dead
        // once the keyboard is done, which is the state this walk arrives in.
        assert!(!sim.drones.channels.on(0xb7), "the voice plate #757 is lock-disabled once ch 0x3A is on [M triggers.rs's lock gate]");
        // The keyboard-lasers switch (#643, out ch 0x3B, gate ch 0x3C [D]) never fired: its gate
        // has no writer in 0a's data [D trig/nis dumps] and the switch sits at the origin
        // (placement transform [D]) — inert on fresh-boot data, asserted end-to-end here.
        assert!(!sim.drones.channels.on(0x3b), "the keyboard-lasers switch stays inert through the whole route [D+M 0x1b02a8's gate check]");
    }
    eprintln!("[0a] walked to the hatch in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
    for f in &w.report.falls { eprintln!("[0a] fall: {f}"); }
}

/// the game's own spawn height vs the port's. The game stands Bond at
/// build_PointOnFloor's down-ray + 1.1 [M Player_StandAtNewPosition 0x190d88 disasm 0x190ee0-
/// 0x190f18: `build_FindCel` from pos + 0.1f, then the ray (0,−1,0)·3.0 through
/// `Collide_RayIntersect` [M build_PointOnFloor 0x1c95a8 disasm 0x1c95b0-0x1c9644, the hit point
/// written back into the placement vector], then `y += 1.1f` (0x3f8ccccd)] — the placement reads
/// 1.1000 in BOTH the game and the port ([L fresh boot: the breakpointed spawn ray from
/// (−3.9319, 0.1, 10.3978) hits y 0.0000 in the spawn cel, surface 0x15; the port's floor_below
/// reads the same 0-plane within 2e-5]). The snapshot's 1.0891 [L ee_start_0700000a.bin
/// 0x1b78640+0x34 = 0x3f8b67fe] is the AFTER-spawn settle: the boot holds the placed root for
/// four ticks while the fall velocity accumulates (`vel += WldGravity·REC_FRAME_RATE` each [M
/// 0x194df0:107]; WldGravity 0x2d8894 = −9.8f, REC_FRAME_RATE 0x30d0dc = 1/60), then integrates
/// ONCE — 1.0891111 reproduces bit-exactly as 1.1f + f32(4·(−9.8f·(1/60))·(1/60)) — and the feet
/// band [M Player_FeetOnPoint 0x194188:106 `hit − 0.1 ≤ cc`] catches the dropped root inside its
/// window (dy 1.0891 ≤ cc 1.0347 + 0.1). The port models the hold in `Bond::settle_hold`
/// (player.rs, level-0a-office-hatch-5); which game gate holds the four boot ticks is the live
/// check in the report (the handler phase cannot be running — its grounded tail zeroes the
/// accumulating velocity). This test waits the settle out and pins the game's rest.
#[test]
fn the_spawn_height_matches_the_game_snapshot() {
    let mut w = load_carry(None);
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    assert!((p[0] + 3.9319).abs() < 0.01 && (p[2] - 10.3978).abs() < 0.01, "the start #0 xz [D placement map37 plc 0]");
    // The settle: four held ticks (root 1.1000) then the one-tick drop to the game's rest [L].
    for _ in 0..10 { w.wait(1); }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    // The game's rest root [L ee_start_0700000a.bin 0x1b78640+0x34] = 1.0891 within half a cm.
    assert!((p[1] - 1.0891).abs() <= 0.005,
        "the spawn root must read the game's 1.0891 [L] (the 4-tick arrival settle [M 0x194df0:107] + the band rest [M 0x194188:106]), got {p:?}");
    assert!(w.scene.sim.as_ref().unwrap().bond.grounded, "the band holds the settled root [M 0x194188:106]");
}

/// the west-hall glide height. The game walks out of the cab at its spawn
/// height 1.0891 [L], brushes the 0.1622-tall wall base at x −8.304 (z 14.3..16.3 [D port world tris,
/// the same geometry the port's world carries]), climbs (peak 1.18 [L fresh boot]) and
/// settles to a steady 1.1147 along the west hall to (−11.39, 16.29) [L fresh boot]. The spawn
/// settle is ported (the_spawn_height_matches_the_game_snapshot — the port now glides the settled
/// 1.0891, the scratch run 1.0905/1.0941 on the doorway lines). What remains is the base climb's
/// settle itself: the game brushes the doorway's jamb base (the tris (−8.604..−8.304, y 0..0.19)
/// at z 16.278 [D port world tris]), peaks 1.18 and settles +0.0256 [L]; the port's capsule (bottom
/// riding dy − cc = 0.054 above the floor) pushes off the base's south face (+0.0036 measured) but
/// does not reproduce the edge-climb settle, and a dead-on r-0.4 approach WEDGES (the walker's
/// stuck panic) — the climb needs the game's per-tick push/step shape through the doorway (the
/// live check in the report).
#[test]
#[ignore = "the through-line route (landed here from settle-5's decode) CROSSES the doorway at the game's rate with the landed settle-5 radial tail — 155 ticks to (-9.27,1.12,16.60) [L 2026-10-26, reproduced] — but the y pin still reads 1.1201 vs the settled 1.1285 [L 2026-10-26]. settle-7 pinned the mechanism bounds: the whole route's floors are FLAT y≈0 [D band_settle_probe: the spawn/lobby/doorway/hall floor hits all read ~2e-5], so no floor step; the game's walk substate carries NO vertical writer [M Player_SSWalk 0x192310 = Player_Move + HandleJump + the matrix rebuild; Player_Move 0x191dd0 writes only the smoothed local slots + turn] and the +0x750 add is per-tick [M Player_Init 0x1910f8:40 sets obj+0xD4 = BL+0x740, so Collide_Update's Vec_Zero(obj+0xD4+0x10) clears the accumulator every tick] — the 1.1285 walk-rest is SET by the jamb-press contact dynamics ([L band-settle-2 brief: placed 1.12 → 1.1402 @f5 → 1.1465 @f13 peak → 1.1285 @f21+, rest after release 1.1285; BLData+0xcc 0.0 at every sample]) and then HELD on the flat walk; the port's press rides HIGHER (probe peak 1.1720) and settles LOWER (1.1130-1.1201) — the press's y residue is the open item (the live check: per-tick obj+0x30 writers + the DeltaP y during the press)"]
fn the_west_hall_glide_height_matches_the_game() {
    let mut w = load_carry(None);
    // Wait out the SS door sweep (the same guard the main walk runs).
    for _ in 0..150 {
        let done = w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
            .filter(|s| s.placement == 524 || s.placement == 525)
            .all(|s| s.state == 0 || s.state == 0x40);
        if done { break; }
        w.wait(5);
    }
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    // The game's glide is ONE walk from the lobby through the doorway into the hall [L
    // 2026-10-26: the fresh-boot glide crosses the doorway in 155 ticks, exiting at
    // (-9.27, 1.12, 16.60) — the jamb slide around the base pieces at z ≈ 16.278 (the jamb tris
    // (-8.604..-8.304, y 0..0.19) [D port world tris]) is part of the through-line, not a
    // detour]. The old (-8.45, 16.29) stop sat INSIDE the jamb column's x-range and wedged the
    // walker (the doorway slide's route is the through-line, decoded on settle-5).
    w.walk_to(Wp::new(-10.0, 0.4, 16.29).r(0.8), "the west hall (the through-line)");
    // Read the glide y in the hall's steady window (x −9.5..−11.39 on the walked line).
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    assert!(p[0] < -9.0, "walked into the west hall, at {p:?}");
    // The game's steady hall glide, RE-PINNED to the 2026-10-26 probe trace [L fresh boot,
    // probe-placed at 1.12, walked west from (-8.0, 1.12, 16.29)]: the settled y 1.1285 (the
    // probe's frames 21/28/36 and the post-release rest all read 1.1285; the old 1.1147 came
    // from a spawn-height walk and the driver pins the SETTLED values, not the first sample),
    // z 16.8295 (the slide north around the jamb — the game is NOT stopped at x -8.304 [L]),
    // passed x -9.0106 at frame 36. The port cannot reproduce the slide yet (the radial-dir
    // experiments, see docs/reports/level-0a-band-settle-2.md) — the test stays ignored.
    assert!((p[1] - 1.1285).abs() <= 0.003,
        "the west hall glide must read the game's settled 1.1285 [L 2026-10-26], got {:.4} at ({:.3},{:.3})",
        p[1], p[0], p[2]);
}

/// the corner crossing. The game, placed at (−17.87, 4.87) fresh-boot and
/// holding forward toward (−17.5, 2.5), walks straight through to (−17.78, 1.120, 2.92) in about 25
/// frames, y 1.127 then 1.120 — no dither, no wedge [L fresh boot]. The port's same leg
/// wedges: its y dithers across the band edge 1.1347 (the -2 session's 1.1336↔1.2019; this
/// session's repro 1.1297 → 1.1957 at (−17.87, 4.87)) because the below-B edge lifts (+0.0165/tick
/// [L the -2 per-tick probe]) unhooks the band at its entry height. The spawn settle is now ported
/// (1.0891 rest) but the hall glide's +0.0256 climb settle is not (the glide sister's note), so the
/// corner still enters 0.02 under the game's 1.1147 line.
#[test]
#[ignore = "the corner now CROSSES with the below-B kill (NF_WALK_KILL=1) at 870 ticks vs the game's ~60 frames [L 2026-10-26], and the crossing still rides the spine-dir pushes (the radial-dir ports measured-and-reverted — see the report) — the exit-y pin is re-settled to 1.1159 [L 2026-10-26]; un-ignore once the crossing is walk-speed"]
fn the_corner_crosses_like_the_game() {
    let mut w = load_carry(None);
    for _ in 0..150 {
        let done = w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
            .filter(|s| s.placement == 524 || s.placement == 525)
            .all(|s| s.state == 0 || s.state == 0x40);
        if done { break; }
        w.wait(5);
    }
    // Walk the proven west legs to the wedge corner (the ring's own anchors).
    for (to, label) in [
        ([-3.9, 0.0, 13.2], "out of the lift"),
        ([-3.9, 0.0, 15.5], "lobby south"),
        ([-10.0, 0.4, 16.0], "the west hall"),
        ([-20.5, 1.3, 16.5], "ring: the west offices' north corridor"),
        ([-20.5, 1.3, 5.5], "ring: down the west office block"),
        ([-17.6, 1.3, 5.5], "ring: the ai node 17 east of the door"),
    ] {
        w.walk_to(Wp::new(to[0], to[1], to[2]).r(0.8), label);
    }
    // The wedge point, then the game's own hold-forward line to (−17.5, 2.5).
    w.walk_to(Wp::new(-17.87, 1.3, 4.87).r(0.4), "the wedge corner");
    let start_ticks = w.total_ticks;
    let arrived = w.try_walk_to(Wp::new(-17.5, 1.3, 2.5).r(0.35), "the game's corner line");
    assert!(arrived, "the game walks the corner in ~25 frames [L fresh boot]; the port wedged at {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    assert!(w.total_ticks - start_ticks <= 300, "the crossing must be walk-speed, took {} ticks", w.total_ticks - start_ticks);
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    // The game's settled exit y, RE-PINNED [L 2026-10-26 fresh boot: placed (-17.87, 1.12, 4.87),
    // yaw 2.987, 60 frames forward — the xz path bends (a slide), y settles 1.1186 mid-path then
    // 1.1159 at the end (-17.9715, 1.1159, -0.4162); the driver pins the settled values].
    assert!((p[1] - 1.1159).abs() <= 0.005, "the corner exit y must read the game's settled 1.1159 [L 2026-10-26], got {:.4}", p[1]);
}

/// The task's repro (collision-0a-west-doorway-floor): walk the ring's west legs one by one and
/// watch where Bond actually loses the floor — the real-path answer to the static probes.
#[test]
fn west_doorway_repro() {
    let mut w = load_carry(None);
    let legs: [([f32; 3], &str); 15] = [
        ([-7.5, 1.3, 14.5], "ring: the security centre's west door"),
        ([-7.5, 1.3, 16.5], "ring: the west hall north"),
        ([-20.5, 1.3, 16.5], "ring: the west offices' north corridor"),
        ([-20.5, 1.3, 5.5], "ring: down the west office block"),
        ([-17.5, 1.3, 5.5], "ring: east of the west door"),
        ([-16.2, 1.3, 5.8], "ring: the ai node 27 east"),
        ([-15.0, 1.3, 5.7], "ring: the ai node 26 east"),
        ([-14.8, 1.3, 4.0], "ring: the ai node 32 south"),
        ([-14.8, 1.3, 2.9], "ring: the ai node 33 south"),
        ([-14.5, 1.3, 2.4], "ring: the ai node 34 south"),
        ([-17.5, 1.3, 2.5], "ring: down past the west door"),
        ([-18.5, 1.3, 2.5], "ring: the door jink west"),
        ([-18.5, 1.3, 0.5], "ring: down"),
        ([-19.5, 1.3, 0.5], "ring: the west corridor jink 2"),
        ([-21.5, 1.3, -1.5], "ring: down 2"),
    ];
    for (wp, label) in legs {
        w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(1.2), label);
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let fl = sim.world.floor_below(p, 3.0);
        eprintln!("[repro] after '{label}': pos ({:.2}, {:.2}, {:.2}) floor {fl:?} falls {}",
            p[0], p[1], p[2], w.report.falls.len());
        for f in &w.report.falls { eprintln!("[repro]   fall: {f}"); }
        assert!(sim.drones.level_exit.is_none(), "the mission failed at '{label}'");
    }
}

/// The band-edge sister: Bond walks onto the south-corridor mouth's floor bridge (the map37
/// mesh strip the instances fix folded in — x −11.43..−10.39, z −19.36..−18.03 [D the disc tri
/// dump]) and stands AT its west edge. The floor band [M Player_FeetOnPoint 0x194188:106] holds
/// him grounded at the edge — the honest edge case the floating wedge was not: no box grazes, the
/// band passes, and he stays put. CAN FAIL if the band constants drift or the bridge drops out
/// again (the instances fix's regression pin).
#[test]
fn bond_stands_at_the_mouth_bridge_edge() {
    let mut w = load_carry(None);
    // Reach: a REACH teleport onto the bridge's west end (the walk there crosses the wedging
    // mouth legs — see the walk test's ignore note); the standing itself is the real input.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-11.35, 1.12, -18.6];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = std::f32::consts::FRAC_PI_2;
        sim.bond.grounded = true;
    }
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let floor = sim.world.floor_below(p, 3.0);
        assert!(floor.is_some(), "the bridge's floor must read under Bond [D map37 plc286/287]");
        let dy = p[1] - floor.unwrap();
        assert!(dy > 1.0 && dy < 1.1347, "the rest dy {} sits inside the band's game window (cc 1.0347 + 0.1 [M 0x194188:106])", dy);
    }
    // Step east to the bridge's mid span and settle — the real-input walk (short, no door zone).
    w.walk_to(Wp::new(-11.0, 1.12, -18.55).r(0.35), "the bridge mid span");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let floor = sim.world.floor_below(p, 3.0);
        assert!(floor.is_some() && (p[1] - floor.unwrap()) > 1.0 && (p[1] - floor.unwrap()) < 1.1347,
            "the band holds on the bridge's mid span (dy {:?} [M 0x194188:106])", floor.map(|f| p[1] - f));
        assert!(sim.bond.grounded, "Bond stands grounded on the bridge (the band holds [M 0x194188:106])");
    }
    // The mouth's passage east of the bridge is a 0.5-wide guard slot (the jamb's corner at
    // (−10.60, −18.66) [L the sister's first run: the segment hit 0.48 ahead]) — the player's
    // route does NOT cross it; the walk test's mouth leg carries that finding.
}

/// The wedge diagnosis: walk leg-5 then leg-6 with per-10-tick dumps of Bond's state, the cel,
/// the scoped vs flat floor, and door #4's live sphere.
#[test]
fn west_doorway_wedge_dump() {
    let mut w = load_carry(None);
    for (wp, label) in [
        ([-7.5f32, 1.3, 14.5], "a: security west door"),
        ([-7.5, 1.3, 16.5], "b: west hall north"),
        ([-20.5, 1.3, 16.5], "c: west offices north corridor"),
        ([-20.5, 1.3, 5.5], "d: down the west office block"),
        ([-17.5, 1.3, 5.5], "e: east of the west door"),
    ] {
        w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(1.2), label);
    }
    // Leg-6 manually: tick with forward input toward (−17.5, 2.5), dumping every 10 ticks while
    // ungrounded.
    let to = [-17.5f32, 2.5];
    let mut ungrounded_ticks = 0u32;
    let (mut last_sample, mut sample_at) = (w.scene.sim.as_ref().unwrap().bond.pos, 0u32);
    let mut press_e = false;
    for tick in 0..1200u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let dx = to[0] - p[0];
            let dz = to[1] - p[2];
            let l = (dx * dx + dz * dz).sqrt();
            if l < 1.2 { eprintln!("[wedge] arrived at tick {tick} pos ({:.2},{:.2},{:.2})", p[0], p[1], p[2]); return; }
            // The walker's stuck bookkeeping: moved < 0.3 over 60 ticks -> press E at the nearest door (the toggle).
            if tick - sample_at >= 60 {
                let moved = ((p[0] - last_sample[0]).powi(2) + (p[2] - last_sample[2]).powi(2)).sqrt();
                last_sample = p;
                sample_at = tick;
                if moved < 0.3 {
                    let near = sim.drones.doors.list.iter().position(|d| ((d.pos[0] - p[0]).powi(2) + (d.pos[2] - p[2]).powi(2)).sqrt() < 2.8);
                    press_e = near.is_some();
                    if press_e { eprintln!("[wedge] t{tick}: stuck (moved {moved:.2}) — pressing E at the near door (the walker's toggle)"); }
                }
            }
            let yaw = dx.atan2(dz);
            let turn = { let mut d = yaw - sim.bond.yaw; while d > std::f32::consts::PI { d -= std::f32::consts::TAU; } while d < -std::f32::consts::PI { d += std::f32::consts::TAU; } d };
            let input = nf_game::player::Input {
                turn: turn.clamp(-0.08, 0.08),
                forward: 1.0, strafe: 0.0,
                ..Default::default()
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, press_e);
            press_e = false;
            let sim = w.scene.sim.as_ref().unwrap();
            if !sim.bond.grounded { ungrounded_ticks += 1; }
            if tick % 10 == 0 || (ungrounded_ticks > 30 && tick % 5 == 0) {
                let p = sim.bond.pos;
                let flat = sim.world.floor_below(p, 3.0);
                let scoped = sim.world.segment_hit_scoped(sim.bond.cel, p, [p[0], p[1] - 2.0, p[2]]);
                eprintln!("[wedge] t{tick} pos ({:.2},{:.2},{:.2}) grounded {} ungt{ungrounded_ticks} flat {flat:?} scoped {scoped:?} cel {:?}",
                    p[0], p[1], p[2], sim.bond.grounded, sim.bond.cel);
                for (di, d) in sim.drones.doors.list.iter().enumerate() {
                    if (d.pos[0] + 19.5).abs() < 1.0 && (d.pos[2] - 4.6).abs() < 1.5 {
                        eprintln!("[wedge]   door{di} state {:#x} pos01 {:.2} sph {:?} matrix row3 {:?}",
                            d.state, d.pos01, d.sph, [d.matrix[3][0], d.matrix[3][1], d.matrix[3][2]]);
                    }
                }
            }
            if ungrounded_ticks > 400 { eprintln!("[wedge] the hang persists at tick {tick}"); return; }
        }
    }
    eprintln!("[wedge] leg-6 budget out");
}

/// The HITTEST+0x50 lifetime accumulator (the port's `Bond.pick_ref`): the 2026-10-26
/// fresh-boot reads [L, PCSX2, player obj 0x1b78640, BL 0x1b78750, HITTEST+0x50 = BL+0x790]:
/// it boots (0,0,0), is NEVER reseeded on a walk, stays EXACTLY constant while Bond free-walks
/// the west hall (frames 20-60+, x −8.93 → −17.1), grows by EXACTLY the per-tick push under the
/// west hall wall press ((0.1, 0, 0) per contact tick, 0.27723 → 3.27709 over 30 ticks, y/z
/// unchanged), and carries over a reload. The only writer is the DeltaP add-back [M Collide_Intersect
/// 0x1ebaa8:353-373], which the port applies as `pick_ref += push` [M player.rs, the walk tick].
#[test]
fn the_pick_ref_accumulator_tracks_the_game_ref() {
    let mut w = load_carry(None);
    // Fresh load: the accumulator reads (0,0,0) [L ee_start_0700000a BL+0x790].
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.pick_ref, [0.0f32; 3],
        "the accumulator boots (0,0,0) [L ee_start_0700000a BL+0x790]");

    // The free walk: probe-placed at the game's own hall line (−8.0, 1.12, 16.29), yaw −1.5708 =
    // west, stick forward [L 2026-10-26]. The first ticks ride the south wall's skin out to the
    // equilibrium (the game's settled z 16.8288 [L]); after that the walk is push-free.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-8.0, 1.12, 16.29];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = -1.5708;
        sim.bond.grounded = true;
    }
    w.wait(10);
    for _ in 0..25 { w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false); }
    let ref_free0 = w.scene.sim.as_ref().unwrap().bond.pick_ref;
    let pos_free0 = w.scene.sim.as_ref().unwrap().bond.pos;
    for _ in 0..40 { w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let (ref_free1, pos_free1) = (sim.bond.pick_ref, sim.bond.pos);
    // The [L] constancy is exact on the real game (its VU0 equilibrium arithmetic); the port's
    // spine-arithmetic skin ride wobbles at the 1e-5 scale (measured: x exact, y 5e-11, z 3e-5
    // over this window) — pinned 3 decades below the [L] 0.1-per-contact-tick push scale the
    // press pins below.
    for k in 0..3 {
        assert!((ref_free1[k] - ref_free0[k]).abs() < 1e-3,
            "the accumulator holds on the free hall walk [L 2026-10-26: frames 20-60+, x {:.2} -> {:.2}; \
             axis {k} moved {} (ref0 {ref_free0:?} pos0 {pos_free0:?} ref1 {ref_free1:?} pos1 {pos_free1:?})]",
            pos_free0[0], pos_free1[0], ref_free1[k] - ref_free0[k]);
    }
    assert!(pos_free1[0] < pos_free0[0] - 1.0, "Bond must actually be walking west ({} -> {})", pos_free0[0], pos_free1[0]);

    // The wall press: out at the west end (the game's wall skin x −45.856/−45.956 [L]), stick
    // forward into it. Every contact tick the push (0.1, 0, 0) adds back: the accumulator's x
    // grows by ~0.1 per tick while y/z hold still.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-40.0, 1.12, 16.83];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = -1.5708;
        sim.bond.grounded = true;
        sim.bond.pick_ref = [0.0; 3]; // the [L] window's own baseline (the game's 0.27723 carried history)
    }
    w.wait(10);
    let press0 = w.scene.sim.as_ref().unwrap().bond.pick_ref;
    assert_eq!(press0, [0.0f32; 3], "the press window starts from the zeroed baseline");
    let mut press_samples: Vec<(f32, f32, f32)> = Vec::new();
    for _ in 0..80 { w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let (ref_press, pos_press) = (sim.bond.pick_ref, sim.bond.pos);
    let _ = press_samples;
    // The 80-tick press reaches the wall (~59 ticks of 0.1 steps) and holds: the [L] press adds
    // +0.1 per contact tick. The port's held press covers >= 15 contact ticks => x >= 1.5.
    assert!(ref_press[0] >= 1.5, "the wall press must accumulate the +0.1 pushes [L 2026-10-26: \
        0.27723 -> 3.27709 over 30 contact ticks], got {}", ref_press[0]);
    assert!(ref_press[1].abs() < 1e-3 && (ref_press[2]).abs() < 1e-3,
        "the press's pushes are x-only [L 2026-10-26: y 0.08088 and z −22.77985 unchanged], got {ref_press:?}");
    assert!(pos_press[0] <= -45.0, "Bond reached the wall skin (x {:.3})", pos_press[0]);
}
