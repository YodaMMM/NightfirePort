//! walkreal-b: 0700000a (Tower B, the office floor) walked with the player's own movement — no
//! teleports after the spawn. Route (docs/levels/0700000a.md): the office-floor start #0
//! (−3.93, 0, 10.40; the arrival plate #58 → ch 0x47 = objective 3) → the corridor loop to the
//! Q-Worm PC (−16.9, 1.2, −32.2) → equip the Q-Worm gadget (0x58) and fire at the PC through the
//! real trigger path [M PCQWorm_Update 0x1aee20] → ch 0x48 (objective 4) → the ring through
//! keypad door #724 to the east corridor → the security room through door #725 (keypad #646's
//! code → ch 0x31) → the security keyboard (SP #60, the mode-64 preload entity) through the REAL
//! use key from the room floor — the pulse stops once ch 0x3A reads on [M SP_Activate 0x1dc338 +
//! SP_Update 0x1dbda8 state 1] → ch 0x49 (objective 5) → AND #742 → ch 0x4A → the hatch exit
//! (21.72, 1.12, 19.79), E through its key gate → 0700000b.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

#[path = "common/usepick.rs"]
mod usepick;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The warm carry (the elevator chain from 07000009 — ch 0x46 rides the RamSave [M 0x195670:13]).
fn load_carry(save: Option<&nf_game::player::RamSave>) -> Walker { load_carry_at(save, 1) }

/// `load_carry` at `difficulty` (1 Operative = the old warm load's replay parity; 2 Agent rides
/// the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty] on the
/// WARM load — the cold capture only harvests the boot's own health/switch/inventory/channels,
/// none difficulty-dependent).
fn load_carry_at(save: Option<&nf_game::player::RamSave>, difficulty: u32) -> Walker {
    let cold = nf_view::load_with_carry(&iso(), "0700000a.bin", false, None).expect("load 0700000a cold");
    let save = save.map(|s| s.to_owned()).unwrap_or_else(|| {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    });
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load_with_carry(&iso(), "0700000a.bin", false, Some(&save)).expect("load 0700000a warm")
    } else {
        nf_view::load_with_carry_difficulty(&iso(), "0700000a.bin", false, Some(&save), difficulty)
            .expect("load 0700000a warm")
    });
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    // The keypad codes: the game's own boot table over the port's local-stream default
    // [L ee_start_0700000a.bin @0x2beff8 — KeyCodes, 0x33·5 bytes; code index 1 reads
    // "3078", shown 0] — the same load-path ground truth the level doc's switch_channels
    // read uses [L @0x26fc90]. The port's Locks_Init stand-in generates on a local stream
    // (the game's boot draws predate the snapshot's capture point); this override pins the
    // digits to the snapshot boot's, so the walked keypad entry types the game's own code.
    if let Ok(snap) = std::fs::read("../../work/ee_start_0700000a.bin") {
        let mut rows = [[0u8; 5]; 0x33];
        for (r, row) in rows.iter_mut().enumerate() {
            let off = 0x2beff8 + r * 5;
            if off + 5 <= snap.len() { row.copy_from_slice(&snap[off..off + 5]); }
        }
        w.scene.sim.as_mut().unwrap().drones.key_codes = nf_game::lock::KeyCodes(rows);
    }
    w
}

/// The crouch toggle through the real input edge: the press pulses every 70 ticks (the game's
/// 60-tick stand/crouch lockout `BL+0x960` [M Player_SSCrouch 0x1924c0:84-105]) until the
/// substate flips — walkreal_07000015's `set_crouch` shape.
fn set_crouch(w: &mut Walker, on: bool) {
    for t in 0..400u32 {
        if w.scene.sim.as_ref().unwrap().bond.crouched == on { return; }
        w.tick(nf_game::player::Input { crouch: t % 70 == 0, ..Default::default() }, false);
    }
    panic!("the crouch toggle never reached {on}");
}

/// The low beams are jumped: no stance passes one (live: #621's low line fired ch 0x24 standing
/// AND crouched [L 2026-11-01]), and the two on the route leave no open end — beam #614 spans
/// the west corridor wall to wall (tris z 13.03, x −24.71..−19.45, y 0.25-0.52 [D triggers
/// dump]; the walls x −25.0 / −19.75 [D probe rays]) and beam #621 (tris (−14.95, −32.40) →
/// (−9.92, −33.95), y 0.13-0.39 [D]) closes the band south of the pocket. The jump is the
/// walk's own (`Player_HandleJump` 0x1916b0); live, a press 0.9 or 1.3 before #614's line
/// cleared it and a walk fired it [L 2026-11-01]. Walks straight at `to`, the jump pressed once as
/// Bond comes within `jump_before` of the beam's line (xz) on the approach side; panics if the
/// leg never arrives.
const BEAM_614: ([f32; 2], [f32; 2]) = ([-24.71, 13.03], [-19.45, 13.03]);
const BEAM_621: ([f32; 2], [f32; 2]) = ([-14.95, -32.40], [-9.92, -33.95]);
// Beam #625's static dump box (x 18.88, z 2.38-7.64, y 0.39-0.65 [D triggers dump]) lives in
// sweep_jump_beam625's doc: the crossing is sweep-aware (the band moves — see beam625_x), so the
// old const-line shape is gone.
fn jump_over_low_beam(w: &mut Walker, beam: ([f32; 2], [f32; 2]), to: [f32; 3], jump_before: f32, label: &str) {
    let (a, b) = beam;
    let (ux, uz) = (b[0] - a[0], b[1] - a[1]);
    let len = (ux * ux + uz * uz).sqrt();
    // Signed perpendicular distance from the beam's line.
    let side = |x: f32, z: f32| ((x - a[0]) * uz - (z - a[1]) * ux) / len;
    let target_side = side(to[0], to[2]);
    let mut jumped = false;
    for _ in 0..900u32 {
        let (p, yaw) = { let s = w.scene.sim.as_ref().unwrap(); (s.bond.pos, s.bond.yaw) };
        if walker::dxz(p, to) < 0.6 {
            eprintln!("[walk] {label}: arrived at {p:?} (jumped {jumped})");
            return;
        }
        let desired = (to[0] - p[0]).atan2(to[2] - p[2]);
        let d = side(p[0], p[2]);
        let jump = !jumped && d.abs() <= jump_before && d.signum() != target_side.signum();
        if jump { jumped = true; }
        w.tick(nf_game::player::Input { forward: 1.0, turn: walker::wrap_pi(desired - yaw).clamp(-0.05, 0.05), jump, ..Default::default() }, false);
    }
    panic!("{label}: the jump leg never arrived (bond {:?})", w.scene.sim.as_ref().unwrap().bond.pos);
}

/// The live z-extent of beam #614's posed collision tris. The glist rides key motion: 3 keys,
/// z 13.0287 → 10.27843 → 13.0287, one pass = 240 ticks (step 1/120 per tick [D keys, m_paths;
/// M Trigger_Create 0x1b1720:56-87 + Trigger_Update 0x1b1a78:60-70; the port re-poses the tris
/// per tick — Motion::advance + step_motion]). [L sweep-probe round 1] the live box parks
/// 12.896..13.173 at t≈224..18 and bottoms 10.146..10.423 at t≈110..148, sliding ≤0.034 u/tick.
fn beam614_z(sim: &nf_view::scene::Sim) -> (f32, f32) {
    sim.drones.triggers.iter().find(|t| t.placement == 614).expect("beam #614 [D]")
        .tris.iter().flat_map(|tri| tri.iter().map(|v| v[2]))
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), z| (a.min(z), b.max(z)))
}

/// The sweep-aware crossing of beam #614. The beam sweeps the whole corridor (the box's centre
/// rides 13.03 ⇄ 10.28 every 240 ticks [D keys; L probe]) so NO standing pass exists: the walk's
/// z(t) is monotone against the sweep and always meets it (the probe's 16/16 phases stamped 0x22
/// [L]). The JUMP is the dodge: its arc runs 0.1 u/tick (double the walk) with the feet rising to
/// 1.84 [L probe round 3], and the standing toucher's segment bottom (B = feet−0.485 [M
/// Player_Collision 0x194df0 default arm; the band top 0.5164 [L]]) clears the beam's band by
/// more than r 0.55 while feet ≥ 1.59 — the arc's middle (k 10..37) is untouchable at ANY beam
/// pose, so ONE jump crosses the whole band. Its low-y ends (k ≤ 9 and k ≥ 38, touch radius
/// sqrt(0.55²−(B−0.5164)²) ≤ 0.54) stay clear of the live box by construction:
/// - southbound: the takeoff stand at z ≤ 13.79 (the arc's k38-42 land south of the box's floor
///   10.146 with dz ≥ radius+0.03) and the arm `box_max ≤ 12.0` (the ascent's k9 keeps dz ≥ 0.43
///   vs the radius 0.25 even against the slide's 0.034/tick);
/// - northbound: the takeoff stand at z ≥ 9.52 (the arc's k39 clears the parked north box
///   13.173 by 0.43 vs 0.25) and the arm `box_min ≥ 12.7` (the beam parked/keeping north so the
///   ascent's k9 keeps dz ≥ 1.79).
/// The arc lands ~4.75 u past the takeoff — past the whole band either way — and the helper pins
/// the standing clearance at the takeoff, the landing clearance, and that ch 0x22 never stamped
/// (the "no laser/alert channel fired" pin, un-weakened [D #614 out 0x22]).
fn sweep_jump_beam614(w: &mut Walker, southbound: bool, x: f32, y: f32, label: &str) {
    // 1. the arm watch: the live box into the crossing window (≤ 3 cycles = 720 ticks).
    let mut armed = false;
    for _ in 0..720u32 {
        let (zmin, zmax) = { let sim = w.scene.sim.as_ref().unwrap(); beam614_z(sim) };
        if if southbound { zmax <= 12.0 } else { zmin >= 12.7 } { armed = true; break; }
        w.wait(1);
    }
    assert!(armed, "{label}: beam #614 never swept into the crossing window in 3 cycles");
    // 2. the takeoff stand (real input; the approach never enters the box's reach — the box's
    //    extreme poses are 13.173/10.146 and the approach stops at 13.75/9.7).
    let takeoff = if southbound { 13.75 } else { 9.7 };
    let desired = if southbound { std::f32::consts::PI } else { 0.0 };
    let mut stood = false;
    for _ in 0..240u32 {
        let p = { let sim = w.scene.sim.as_ref().unwrap(); sim.bond.pos };
        let there = if southbound { p[2] <= takeoff } else { p[2] >= takeoff };
        if there { stood = true; break; }
        let yaw = w.scene.sim.as_ref().unwrap().bond.yaw;
        let turn = walker::wrap_pi(desired - yaw).clamp(-0.05, 0.05);
        w.tick(nf_game::player::Input { forward: 1.0, turn, ..Default::default() }, false);
    }
    assert!(stood, "{label}: the takeoff stand at z {takeoff} never arrived");
    // 3. the standing-clearance pin at the takeoff tick.
    let (zmin, zmax) = { let sim = w.scene.sim.as_ref().unwrap(); beam614_z(sim) };
    let gap = if southbound { takeoff - zmax } else { zmin - takeoff };
    assert!(gap > 0.56, "{label}: the takeoff stand sits {gap:.3} from the live box ({zmin:.3}..{zmax:.3}) — the standing touch radius 0.55 [L the phase probe]");
    // 4. the jump: one tick of jump input, then forward through the arc to the far z.
    let z_far = if southbound { 9.35 } else { 13.9 };
    let mut jumped = false;
    let mut crossed = false;
    for _ in 0..160u32 {
        let p = { let sim = w.scene.sim.as_ref().unwrap(); sim.bond.pos };
        let there = if southbound { p[2] <= z_far } else { p[2] >= z_far };
        if there { crossed = true; break; }
        let yaw = w.scene.sim.as_ref().unwrap().bond.yaw;
        let turn = walker::wrap_pi(desired - yaw).clamp(-0.05, 0.05);
        w.tick(nf_game::player::Input { forward: 1.0, turn, jump: !jumped, ..Default::default() }, false);
        jumped = true;
    }
    assert!(crossed, "{label}: the sweep-jump arc never reached z {z_far}");
    // 5. the landing clearance + the definitive 0x22 pin.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let (zmin, zmax) = beam614_z(sim);
        let p = sim.bond.pos;
        let gap = if southbound { zmin - p[2] } else { p[2] - zmax };
        assert!(gap > 0.56, "{label}: the arc landed {gap:.3} from the live box ({zmin:.3}..{zmax:.3}) at {p:?}");
        assert!(sim.drones.channels.time[0x22] == 0,
            "{label}: ch 0x22 stamped — the swept beam was touched [D #614 out 0x22; the pin stands]");
        eprintln!("[{label}] the sweep-jump cleared the band: pos {p:?} box ({zmin:.3},{zmax:.3})");
    }
}

/// The live x-extent of beam #625's posed collision tris. The glist rides key motion: 3 keys,
/// x 18.877 -> 16.078 -> 18.877, one pass = 240 ticks (step 1/120 per tick [D keys, m_paths —
/// the probe's dump of the placement; M Trigger_Create 0x1b1720:56-87 + Trigger_Update
/// 0x1b1a78:60-70; the port re-poses the tris per tick — Motion::advance + step_motion]) — the
/// same sweep shape #614 rides, along the corridor's x axis. The key-0 box is the static dump's
/// x 18.744..19.022, z 2.38..7.64, y 0.39..0.65 [D triggers dump].
fn beam625_x(sim: &nf_view::scene::Sim) -> (f32, f32) {
    sim.drones.triggers.iter().find(|t| t.placement == 625).expect("beam #625 [D]")
        .tris.iter().flat_map(|tri| tri.iter().map(|v| v[0]))
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), x| (a.min(x), b.max(x)))
}

/// The sweep-aware westbound crossing of beam #625 (the ring's east corridor, z ~4.9-5.3): the
/// same law sweep_jump_beam614 pins, rotated to the x axis. The old blind press (jump 1.0 before
/// the STATIC line x 18.88 [D], the jump_over_low_beam shape) predates the sweep reading: the
/// beam's band sweeps x 18.877 <=> 16.078 [D keys] over its 240-tick pass, its west park reaches
/// x_min 15.94 [L probe_0a_beams: the key-0 lo 18.744 minus the 2.799 key travel], and the
/// whole westbound lane (z 4.9-5.3) sits inside the beam's z span 2.38..7.64 [D] — so the walk's
/// x(t) meets the sweep whatever the phase (the #614 probe's 16/16 phases, along x now). The
/// strict beam law (the glist flags 0x645 [D]; the touch goes through the posed plane records'
/// one-sided refine `seg_touches_tri_raw` [M Trigger_Hit 0x1b1fc8 -> Collide_Intersect
/// 0x1ebaa8:93-99 -> Intersect_CylGeom 0x109da0's RAW branch -> Dist2Tri 0x1dfca8, wired in
/// triggers.rs `Trigger::hits`'s seg_beam arm]) stamps ch 0x26 whenever a stand or the arc's low
/// tail overlaps the band's (x,z) footprint at the wrong phase — the walk's upstream timing
/// shifted and the four qworm walks crossed it (the 0x26 stamp, this round's red).
/// The re-timing, ring-sweep-timing's method:
/// - the arm watch: the beam parked EAST (posed box_max >= 18.75; the key-0 park hi is 19.022
///   [D], the wrap parks it there each pass) — the takeoff side clear and the whole westward
///   sweep ahead of the arc;
/// - the press at x <= 19.88 (1.0 before the static line 18.88 [D] — the old press distance,
///   kept: the arc's low early ticks stay east of the posed box, which only moves west from the
///   park);
/// - the arc west until x <= 15.3 (0.64+ west of the west park's reach x_min 15.94 - 0.55
///   touch radius [L]) — the arc's middle clears vertically at any pose (the #614 probe's
///   numbers: the arc runs ~0.1 u/tick, the feet rise to ~1.84, the standing toucher's bottom
///   B = feet-0.485 [M Player_Collision 0x194df0 default arm] clears the band top 0.65 [D]
///   while feet >= 1.69);
/// - the pins: the press/landing gaps > 0.56 from the POSED box, and ch 0x26 never stamped
///   [D #625 out 0x26].
fn sweep_jump_beam625(w: &mut Walker, label: &str) {
    // 1. the arm watch: the live box into the east-park window (<= 3 cycles = 720 ticks).
    let mut armed = false;
    for _ in 0..720u32 {
        let (xmin, xmax) = { let sim = w.scene.sim.as_ref().unwrap(); beam625_x(sim) };
        if xmax >= 18.75 { armed = true; break; }
        w.wait(1);
    }
    assert!(armed, "{label}: beam #625 never swept into the east-park window in 3 cycles");
    // 2. the press stand: walk west, no jump input, until 1.0 before the static line.
    let press_x = 19.88f32;
    let mut stood = false;
    for _ in 0..240u32 {
        let p = { let sim = w.scene.sim.as_ref().unwrap(); sim.bond.pos };
        if p[0] <= press_x { stood = true; break; }
        let yaw = w.scene.sim.as_ref().unwrap().bond.yaw;
        let desired = (9.9f32 - p[0]).atan2(5.3 - p[2]);
        w.tick(nf_game::player::Input { forward: 1.0, turn: walker::wrap_pi(desired - yaw).clamp(-0.05, 0.05), ..Default::default() }, false);
    }
    assert!(stood, "{label}: the press stand at x {press_x} never arrived");
    // 3. the standing-clearance pin at the press tick (the posed box, not the static dump: the
    //    beam only moves WEST from the park, so the gap here is the pass's worst east-side gap).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let (xmin, xmax) = beam625_x(sim);
        let p = sim.bond.pos;
        let gap = p[0] - xmax;
        assert!(gap > 0.56, "{label}: the press stand sits {gap:.3} from the live box ({xmin:.3}..{xmax:.3}) — the standing touch radius 0.55 [L the #614 phase probe]");
    }
    // 4. the arc: one tick of jump input, then forward through the arc past the whole band.
    let x_far = 15.3f32;
    let mut jumped = false;
    let mut crossed = false;
    for _ in 0..160u32 {
        let p = { let sim = w.scene.sim.as_ref().unwrap(); sim.bond.pos };
        if p[0] <= x_far { crossed = true; break; }
        let yaw = w.scene.sim.as_ref().unwrap().bond.yaw;
        let desired = (9.9f32 - p[0]).atan2(5.3 - p[2]);
        w.tick(nf_game::player::Input { forward: 1.0, turn: walker::wrap_pi(desired - yaw).clamp(-0.05, 0.05), jump: !jumped, ..Default::default() }, false);
        jumped = true;
    }
    assert!(crossed, "{label}: the sweep-jump arc never reached x {x_far}");
    // 5. the landing clearance + the definitive 0x26 pin.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let (xmin, xmax) = beam625_x(sim);
        let p = sim.bond.pos;
        let gap = xmin - p[0];
        assert!(gap > 0.56, "{label}: the arc landed {gap:.3} from the live box ({xmin:.3}..{xmax:.3}) at {p:?}");
        assert!(sim.drones.channels.time[0x26] == 0,
            "{label}: ch 0x26 stamped — the swept beam was touched [D #625 out 0x26; the pin stands]");
        eprintln!("[{label}] the sweep-jump cleared the band: pos {p:?} box ({xmin:.3},{xmax:.3})");
    }
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
        w.after_step();
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

/// The game's own sight triple against one point [M 0x175ef0 family: the distance gate, the yaw
/// cone gate, the LOS ray]: whether the drone `di` would see a body standing at `p` right now
/// (the eye ≈ pos+1.2, the point at +1.2; `world.segment` is the LOS arm — Some = blocked).
fn watcher_sees(sim: &nf_view::scene::Sim, di: usize, p: [f32; 3]) -> bool {
    let d = &sim.drones.list[di];
    if !d.present || !d.enabled || d.corpse || d.health <= 0.0 || d.team == 0 { return false; }
    let dx = p[0] - d.pos[0];
    let dz = p[2] - d.pos[2];
    let dist = (dx * dx + dz * dz).sqrt();
    if dist > d.sight_range { return false; }
    // The facing vector in the walker's own yaw convention (desired = dx.atan2(dz)).
    let fx = d.yaw.sin();
    let fz = d.yaw.cos();
    let dot = (fx * dx + fz * dz) / dist.max(1e-4);
    if dot < d.half_fov.cos() { return false; }
    // The LOS arm at three heights, ANY clear = exposed: the game's sight ray runs from the
    // drone's own eye height (not measured here — 0x175ef0's ray args are the live read), and a
    // single assumed height can duck under furniture the game's ray clears (the t14705 g17
    // sighting cleared LOS at d 10.8 through the east strip while the +1.2-only probe read
    // blocked [L 2026-11-05]). Over-conservative is the safe direction: the gate waits. The
    // band is LOW (0.4..2.0): the game's eye→chest ray runs near body height and the east
    // strip's overhead structure blocked every ≥2.2 probe while the game's LOS read true —
    // high probes under-report exposure, low probes can only over-report it (the gate waits).
    for h in [0.4f32, 0.7, 1.0, 1.3, 1.6, 2.0] {
        let eye = [d.pos[0], d.pos[1] + h, d.pos[2]];
        let tgt = [p[0], p[1] + h, p[2]];
        if sim.world.segment(eye, tgt).is_none() { return true; }
    }
    false
}

/// Every live non-team-0 drone that would see Bond at his current spot or at any of `points`
/// right now (the stealth gate's check arm).
fn sight_watchers(w: &Walker, points: &[[f32; 3]]) -> Vec<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    let here = sim.bond.pos;
    let mut bad = Vec::new();
    for di in 0..sim.drones.list.len() {
        if watcher_sees(sim, di, here) || points.iter().any(|&p| watcher_sees(sim, di, p)) {
            bad.push(di);
        }
    }
    bad
}

/// Hold at the current spot until no live civilian sees Bond here or any of `points`. The
/// civilians cannot be dropped (the office floor's ackers are team-3 civilians with props
/// 0x53/0x54 set — a player kill is fail reason 3 [M DroneFunc_OnInitDeath 0x145bd0's verdict;
/// hitdata-spawner-ally.md:203-206]) and a press cannot be undone once 0x8a starts (the fail is
/// the press timer C, RunToAlarm + 180 + 30 ticks [M NDrone2_DSTATE_PressAlarm 0x168ea8]) — the
/// only player move is to be SEEN BY NOBODY. Panics with the geometry dump when no window opens
/// (a route bug, not a wait).
fn wait_sight_clear(w: &mut Walker, points: &[[f32; 3]], what: &str) {
    let start = w.total_ticks;
    loop {
        let bad = sight_watchers(w, points);
        if bad.is_empty() { return; }
        if w.total_ticks - start >= 5400 {
            let here = w.scene.sim.as_ref().unwrap().bond.pos;
            let mut s = format!("no sight window for '{what}' after {} ticks (watchers {})\n",
                w.total_ticks - start, bad.iter().map(|i| format!("g{i}")).collect::<Vec<_>>().join(","));
            for &di in &bad {
                let d = &w.scene.sim.as_ref().unwrap().drones.list[di];
                s.push_str(&format!("  g{di} state {:#x} at ({:.2},{:.2},{:.2}) yaw {:.2} rng {:.1} fov {:.2}\n",
                    d.state, d.pos[0], d.pos[1], d.pos[2], d.yaw, d.sight_range, d.half_fov));
            }
            s.push_str(&walker::fail_dump_pub(&w.scene, here, what));
            panic!("{s}");
        }
        w.wait(1);
    }
}

/// Hold until no live team-3 drone is within `range` of Bond here or any of `points` —
/// DISTANCE-ONLY (the world-segment LOS probe over-blocks the east strip [L the t14717 note:
/// the probe read the ray blocked at every height while the game's los read true], so the gate
/// here is the sight RANGE itself). Capped: a patrol that never leaves the band proceeds racy
/// and the caller's pins judge. Returns whether the window was clean.
fn wait_distance_clear(w: &mut Walker, points: &[[f32; 3]], range: f32, max_ticks: u32, what: &str) -> bool {
    let start = w.total_ticks;
    loop {
        let bad: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            sim.drones.list.iter().enumerate().filter(|(_, d)| {
                d.present && !d.corpse && d.health > 0.0 && d.enabled && d.team != 0
                    && (walker::dxz(d.pos, bp) < range || points.iter().any(|p| walker::dxz(d.pos, *p) < range))
            }).map(|(i, _)| i).collect()
        };
        if bad.is_empty() { return true; }
        if w.total_ticks - start >= max_ticks {
            eprintln!("[0a] {what}: no distance window after {} ticks (in-range {:?}) — proceeding", w.total_ticks - start, bad);
            return false;
        }
        w.wait(1);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { return false; }
    }
}

/// Hold at a hidden stand until the coming legs are safe from the office floor's patrollers:
/// nobody sees Bond here or any of `points` (`watcher_sees`), and nobody within `near` of a point
/// is walking toward it (sampled over 20 ticks). The civilians cannot be shot (props 0x53/0x54,
/// see `wait_sight_clear`), so a player waits for the patrol to walk away from his path before
/// he takes it. Returns whether the window opened inside `max_ticks`.
fn wait_patrol_clear(w: &mut Walker, points: &[[f32; 3]], near: f32, max_ticks: u32, what: &str) -> bool {
    let start = w.total_ticks;
    let snap = |w: &Walker| -> Vec<Option<[f32; 3]>> {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter().map(|d| (d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team != 0).then_some(d.pos)).collect()
    };
    let mut before = snap(w);
    loop {
        w.wait(20);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { return false; }
        let now = snap(w);
        let seen = !sight_watchers(w, points).is_empty();
        let closing = now.iter().zip(before.iter()).any(|(a, b)| match (a, b) {
            (Some(a), Some(b)) => points.iter().any(|&p| {
                let da = walker::dxz(*a, p);
                da < near && da < walker::dxz(*b, p) - 0.1
            }),
            _ => false,
        });
        before = now;
        if !seen && !closing {
            eprintln!("[0a] {what}: held {} ticks", w.total_ticks - start);
            return true;
        }
        if w.total_ticks - start >= max_ticks {
            eprintln!("[0a] {what}: no patrol window after {} ticks — proceeding", w.total_ticks - start);
            return false;
        }
    }
}

/// Hold until sensor `placement` is sweeping (state 0) with its playhead in `[lo, hi)`.
fn wait_sensor_phase(w: &mut Walker, placement: usize, lo: f32, hi: f32, what: &str) {
    let start = w.total_ticks;
    loop {
        let (state, t) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let s = sim.drones.sensors.list.iter().find(|s| s.placement == placement).expect("the sensor placement [D]");
            (s.state, s.t)
        };
        if state == 0 && t >= lo && t < hi { break; }
        assert!(w.total_ticks - start < 2400, "{what}: the sweep never reached playhead {lo}..{hi} (state {state}, t {t})");
        w.wait(1);
    }
    eprintln!("[0a] {what}: held {} ticks", w.total_ticks - start);
}

/// `walk_leg_watched` with the sight gate before each ~4-unit segment: hold at the segment's
/// start until no civilian sees the start or the segment's end, then walk it. The hold is sound
/// because the PREVIOUS segment verified its own end clear and the civilians drift ≤ ~2 u per
/// segment (patrol pace ~0.02 u/tick [L 2026-11-05 trace]); if the arrival spot itself is
/// exposed the gate's panic dump shows the geometry (re-anchor the lane, never walk blind).
fn walk_leg_stealth(w: &mut Walker, to: [f32; 3], arrive: f32, label: &str) -> bool {
    walk_leg_stealth_cap(w, to, arrive, label, 200)
}

/// `walk_leg_stealth` with a seg cap: the pocket tail's furniture wedges the straight slices
/// ~1 u short of the far targets and the full 200-seg loop grinds there INSIDE GT #639's fire
/// arc — the capped variant stops at the grind (the wedge spot IS the E-reach stand, see the
/// keypad block) and lets the caller carry on from there.
fn walk_leg_stealth_cap(w: &mut Walker, to: [f32; 3], arrive: f32, label: &str, cap: usize) -> bool {
    let seg = 4.0f32;
    for iteration in 0..cap {
        let from = w.scene.sim.as_ref().unwrap().bond.pos;
        let dx = to[0] - from[0];
        let dz = to[2] - from[2];
        let dist = (dx * dx + dz * dz).sqrt();
        if dist < arrive { return true; }
        watch_and_clear(w, 16.0);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { return false; }
        let step = seg.min(dist);
        let mid = [from[0] + dx / dist * step, to[1], from[2] + dz / dist * step];
        wait_sight_clear(w, &[mid], &format!("{label} seg {iteration}"));
        w.walk_to(Wp::new(mid[0], mid[1], mid[2]).r(1.1), &format!("{label} seg {iteration}"));
    }
    false
}

/// Aim the view at `target` and pulse the use key until `done` reads true (the press lands on the
/// key's edge [M Input_Action mask 4]). Returns whether `done` came true.
fn aim_and_press(w: &mut Walker, target: [f32; 3], cycles: u32, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> bool {
    for _cycle in 0..cycles {
        for t in 0..40u32 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                if done(sim) { return true; }
                let bp = sim.bond.pos;
                let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
                let desired = (target[0] - bp[0]).atan2(target[2] - bp[2]);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                let perr = ((target[1] - eye) / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2 - sim.bond.pitch;
                let input = nf_game::player::Input {
                    turn: err.clamp(-0.05, 0.05),
                    look: perr.clamp(-0.05, 0.05),
                    ..Default::default()
                };
                let hold = t < 3 && err.abs() < 0.06;
                w.scene.sim.as_mut().unwrap().step_bond(input, false, false, hold);
            }
            w.total_ticks += 1;
        }
    }
    done(w.scene.sim.as_ref().unwrap())
}

/// The z ≈ 5 corridor west of beam #625, the keypad stand and door #725: the legs from the east
/// corridor into the security room.
const ROOM_APPROACH: [[f32; 3]; 6] = [[21.3, 1.3, 5.3], [15.0, 1.3, 5.3], [9.9, 1.3, 5.3], [9.75, 1.3, 2.2], [8.2, 1.3, 0.5], [7.6, 1.3, -0.9]];
/// The legs out of the security room to the hatch anteroom (the ring's north legs).
const ROOM_EXIT: [[f32; 3]; 7] = [[7.6, 1.3, -0.9], [9.75, 1.3, 2.2], [9.9, 1.3, 5.3], [9.75, 1.3, 8.8], [9.6, 1.3, 12.3], [11.5, 1.3, 15.5], [17.5, 1.3, 15.5]];

/// The security room, walked from the east corridor at (9.9, 5.3): door #725 → the keyboard →
/// objective 5 → back out to the corridor.
///
/// The door [D placement #725]: kind 0 swing, (6.83, 1.10, −1.46), p3 = 49 → obj+0x68, the lock
/// channel [M Door_Create 0x136100: level_tag+0x38 → +0x68]. While +0x68 is set the door refuses
/// E; the rising edge of switch_channels[+0x68] posts "Door Unlocked", activates the door and
/// zeroes +0x68 for good [M Door_Update 0x1363c8:88-107]. The only placements on 0a that carry
/// 49 are the door and its two keypads, #646 (east face, (7.01, 1.33, 0.26), yaw +π/2) and #729
/// (west face, yaw −π/2): type 0x2E locks, p0 = 0 (keypad kind), p1 = 2 (the KeyCodes row),
/// p5 = 49 (the out channel) [D the full 0a placement scan, every map]. E on #646 arms the keypad
/// [M Lock_Activate 0x1aed00 → state 3], the four digits of KeyCodes row 2 match [M Lock_Update
/// 0x1ae3d8 case 4] → `Lock_Unlock` 0x1b0650 → state 1, whose update holds switch_channels[49]
/// [M 0x1ae3d8 case 1]. The keyboard's collision faces face −z, into the room [D the entity glist
/// 0x020005E1], so the press is made from the room floor in front of its screen.
fn security_room_visit(w: &mut Walker) {
    let door = {
        let sim = w.scene.sim.as_ref().unwrap();
        let di = sim.drones.doors.list.iter().position(|d| d.placement == 725).expect("door #725 [D]");
        let d = &sim.drones.doors.list[di];
        assert_eq!(d.ch_lock, 49, "door #725 is locked on ch 0x31 [D p3]");
        assert!(d.state & 0xc == 0, "door #725 is closed before its keypad");
        assert!(!sim.drones.channels.on(49), "ch 0x31 is off before the keypad");
        assert!(!sim.drones.channels.on(0x3a), "the keyboard is not used before Bond is in its room");
        di
    };
    // South along the corridor's west wall to the keypad's stand, in front of #646 (east face).
    for (wp, label) in [([9.75f32, 1.3, 2.2], "room: the corridor south to the keypad"),
                        ([8.2, 1.3, 0.5], "room: in front of keypad #646")] {
        watch_and_clear(w, 16.0);
        w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(0.5), label);
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed at '{label}'");
    }
    // E the keypad: the world sphere of the east-face lock.
    let (kp, li) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let li = sim.drones.locks.iter().position(|l| l.placement == 646).expect("keypad #646 [D]");
        (sim.drones.locks[li].pos, li)
    };
    let armed = aim_and_press(w, kp, 8, &|s| s.drones.keypad.is_some());
    assert!(armed, "the keypad #646 E arms the mini-game (state 3) [M 0x1aed00's kind-0 no-key branch]");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.keypad.as_ref().map(|(i, _)| *i), Some(li),
        "the press armed #646, not another keypad");
    // The four digits of the lock's KeyCodes row [M C_KEYPAD_Handler 0x216810 msg 0x4b].
    let code = {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.locks[li].code, 2, "#646's KeyCodes row is p1 = 2 [D]");
        sim.drones.key_codes.code(sim.drones.locks[li].code)
    };
    for d in code {
        assert!(d.is_ascii_digit(), "the code digits are ASCII");
        w.scene.sim.as_mut().unwrap().drones.keypad_digit((d - b'0') as u32);
        w.wait(1);
    }
    // The match → ch 0x31 → the door's rising-edge unlock + auto-activate.
    {
        let mut ok = false;
        for _ in 0..120 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.channels.on(49) && sim.drones.doors.list[door].state == 4 { ok = true; break; }
            w.wait(1);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(ok, "the code match unlocks ch 0x31 [M 0x1ae3d8 case 4 → case 1] and door #725 opens by itself \
            [M 0x1363c8:88-107] (state {:#x})", sim.drones.doors.list[door].state);
        assert!(sim.drones.locks[li].unlocked, "the used lock's state 1 [M 0x1b0650]");
        assert_eq!(sim.drones.doors.list[door].ch_lock, 0, "the unlock clears the door's lock channel for good [M 0x1363c8: +0x68 = 0]");
    }
    // Through the doorway and across the room to the keyboard's screen.
    for (wp, label) in [([7.6f32, 1.3, -0.9], "room: the doorway's east side"),
                        ([6.1, 1.3, -0.92], "room: through door #725"),
                        ([2.8, 1.3, 0.49], "room: the room's south-west"),
                        ([2.4, 1.3, 4.87], "room: north along the room"),
                        ([-1.0, 1.3, 7.2], "room: in front of the keyboard")] {
        w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(0.5), label);
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed at '{label}'");
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > -2.0 && p[0] < 6.7 && p[2] > -4.7 && p[2] < 8.7,
            "Bond stands inside the security room (west of the x 6.9 wall, south of the screen), got {p:?}");
    }
    // The keyboard [M SP_Activate 0x1dc338 + SP_Update 0x1dbda8 state 1 → ch 0x3A], aimed at its
    // screen's middle. The press toggles the out channel, so the pulse stops once it reads on.
    let kb_fired = aim_and_press(w, [-1.55, 1.5, 8.7], 12, &|s| s.drones.channels.on(0x3a));
    assert!(kb_fired, "the keyboard SP's E activation writes ch 0x3A \
        [M SP_Activate 0x1dc338 + SP_Update 0x1dbda8 state 1]");
    // The AND mux #741 follows on the mission graph's own update.
    let mut mux49 = false;
    for _ in 0..180 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x49) { mux49 = true; break; }
        w.wait(1);
    }
    assert!(mux49, "AND mux #741: ch 0x3A → ch 0x49 (objective 5) [D]");
    // #742 reads (0x48, 0x49) one settle tick behind #741's write.
    let mut mux4a = false;
    for _ in 0..30 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4a) { mux4a = true; break; }
        w.wait(1);
    }
    assert!(mux4a, "AND mux #742: 0x48+0x49 → the exit's key ch 0x4A [D]");
    // Back out the way in. With +0x68 cleared, E opens the door from inside like any unlocked
    // door [M Door_Activate 0x135be0] when it has swung shut.
    for (wp, label) in [([2.4f32, 1.3, 4.87], "room: back south along the room"),
                        ([2.8, 1.3, 0.49], "room: back to the room's south-west"),
                        ([5.4, 1.3, -0.92], "room: inside door #725")] {
        w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(0.5), label);
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed at '{label}'");
    }
    // Behind the shut door Bond is out of sight; the way out runs up the patrol's north lane.
    wait_patrol_clear(w, &ROOM_EXIT, 20.0, 9000, "out of the security room");
    w.use_door(door, "room: E door #725 from inside");
    for _ in 0..120 {
        if w.scene.sim.as_ref().unwrap().drones.doors.list[door].state == 4 { break; }
        w.wait(1);
    }
    for (wp, label) in [([7.6f32, 1.3, -0.9], "room: out through door #725"),
                        ([9.75, 1.3, 2.2], "room: back up the corridor")] {
        watch_and_clear(w, 16.0);
        w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(0.5), label);
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed at '{label}'");
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    assert!(p[0] > 7.0, "Bond is back in the east corridor, got {p:?}");
}

/// The office-floor prologue from the fresh load on real input: the arrival plate (obj 3), the
/// lift-cab sweep, the corridor loop to the Q-Worm PC, the REAL install (ch 0x48, objective 4),
/// and the walk back to the lobby. Everything here is walked (step_bond only). The security
/// keyboard (objective 5) is the ring's, from inside the security room (`security_room_visit`).
fn walk_qworm_and_back() -> Walker { walk_qworm_and_back_at(1) }

/// `walk_qworm_and_back` at `difficulty` (2 = the front-end's launch path, see load_carry_at).
fn walk_qworm_and_back_at(difficulty: u32) -> Walker {
    let mut w = load_carry_at(None, difficulty);
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
    // Beam #607 (out ch 0x1F, mask 8, x ≈ −8.45, y 1.49-1.76, z 15-20.25 [D triggers dump])
    // spans the west-hall mouth: the player's answer is the CROUCH. Live on a fresh boot
    // [L 2026-11-01, PCSX2: laser_band live script]: crouched (substate 4, pos y 0.608) Bond
    // walked (−7.61, 16.22) → (−11.43, 16.71) through the plane with ch 0x1F (and 0x0F) staying
    // 0; standing (pos y 1.131) back east the same line fired 0x1F + 0x0F at x −8.95 — the
    // crouched pick arm [M Player_Collision 0x194df0 case 3] passes under the band, the standing
    // body [M default arm] does not. The same boot showed the crouched straight line
    // (−3.9, 15.4) → (−10, 16) wedges at x −7.75, z 16.0 (the doorway's south jamb): the walked
    // crossing rides the doorway line z ≈ 16.6 (the ring's own west-hall line).
    set_crouch(&mut w, true);
    watch_and_clear(&mut w, 16.0);
    w.walk_to(Wp::new(-7.4, 0.4, 16.6).r(0.6), "the west hall's doorway line (crouched)");
    w.walk_to(Wp::new(-10.0, 0.4, 16.6).r(0.6), "through beam #607's plane (crouched)");
    w.walk_to(Wp::new(-11.6, 0.4, 16.6).r(0.6), "west, clear of beam #607's reach (crouched)");
    set_crouch(&mut w, false);
    // The west corridor south: beam #614 spans it wall to wall at ankle height (the
    // jump_over_low_beam doc) — the only player lane south from the hall (the office block east of
    // the corridor is closed at z 15.25 / the x −19.75 wall down to its door at z ≈ 5.5 [D probe
    // rays]; the A*'s own lane is this corridor). Walk the north corridor to the corridor head,
    // then JUMP the line. The alarm-runner catch runs between the legs (the slower crouched pace
    // opens the panel-side civilian's window).
    for (wp, label) in [([-16.6f32, 0.4, 17.4], "the west offices' north corridor"),
                        ([-22.2, 0.4, 16.6], "the west corridor's head"),
                        ([-22.2, 0.4, 15.4], "the west corridor, north of beam #614")] {
        watch_and_clear(&mut w, 16.0);
        w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(0.6), label);
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission ended at the outbound leg '{label}'");
    }
    // The sweep-aware crossing (see sweep_jump_beam614): the blind 1.8 press landed the arc in
    // the beam's swept pose — the takeoff moves to 13.75 behind the arm box_max ≤ 12.0, the arc
    // clears the whole swept band, and the 0x22 pin stays.
    sweep_jump_beam614(&mut w, true, -22.2, 0.4, "jump beam #614 southbound (the sweep)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let on: Vec<String> = [0x1f, 0x20, 0x21, 0x22, 0x24, 0x25, 0x26, 0x0f, 0x10, 0x11, 0x12, 0x81]
            .iter().filter(|c| sim.drones.channels.time[**c as usize] != 0).map(|c| format!("{c:#04x}")).collect();
        eprintln!("[0a] outbound past #614: pos {:?} fired [{}] exit {:?}", sim.bond.pos, on.join(","), sim.drones.level_exit);
    }
    assert!(w.follow_nav_route([pc[0] + 2.0, 0.4, pc[2] + 3.0], "to the Q-Worm PC"), "no nav route to the PC");
    // The cubicle's south side, walked (this round's route — probe_l0a3_install, all legs
    // arrived): the office block's south wall runs z −25.2..−27.1 (north faces n (0.26,0,0.97),
    // x −18..−6 [D probe rays]); the pocket south of it (the nav route's end (−14.88, ·, −29.15))
    // is walled off from the band behind the cubicle by the diagonal wall whose north face
    // (n (0.30,0,0.95)) spans x −18..−15 at z −29.8..−31.5 [D probe rays] — but its EAST END
    // stops at x ≈ −15: the column at x −13.5 runs south into the band (northward rays from
    // z −34.4 clear at x −13 and x −9 [D probe rays]; the prior round's grid stopped at x −15).
    // The band's floor (y 1.10) holds the bullet-clear stands: from (−15.89, 1.10, −34.24) the
    // Q-Worm BULLET reaches the PC clear [M PCQWorm_Update 0x1aee20's class-5 seg test; D probe:
    // segment_bullet CLEAR from every stand x −16..−17.8, z −33.5..−34.4].
    // The band legs are FATAL (walk_to panics at the 6000-tick cap [walker]): the passage is
    // probe-proven (probe_l0a3_install, all legs arrived), and a silently-stranded leg would
    // leave Bond short of the stand so the install shot never lands.
    w.walk_to(Wp::new(-13.9, 0.4, -30.5).r(0.9), "pocket south-east");
    // Beam #621 (out ch 0x24, OR #668 → ch 0x10 → gun #639) closes the band south of the pocket
    // at ankle height (jump_over_low_beam): down the x −13.5 column (northward rays from z −34.4
    // clear at x −13 [D probe rays]), jumping the line, then west along the band.
    w.walk_to(Wp::new(-13.5, 0.4, -30.6).r(0.5), "the column north of beam #621");
    jump_over_low_beam(&mut w, BEAM_621, [-13.5, 0.4, -34.4], 1.0, "jump beam #621 southbound");
    w.walk_to(Wp::new(-16.5, 0.4, -34.6).r(0.7), "the band west, south of #621");
    w.walk_to(Wp::new(-16.8, 0.4, -34.0).r(0.3), "the south stand");
    // The REAL install: equip the Q-Worm through the real gadget cycle (mode-1 bounds 0x4A..0x5E,
    // owned entries only [M player.rs Input::gadget_cycle / Gun_Update's cycle]) — the prior
    // round proved the old legs' 112 blocked shots were the spawn loadout, never the gadget —
    // then shoot the PC from the walked south stand. The writer is the PC's own: a weapon-0x58
    // BULLET entering the PC's contact list writes switch_channels[0x48]
    // [M PCQWorm_Update 0x1aee20]. No stand-in: the walked stand is bullet-clear and the shot
    // lands (probe_l0a3_install: 1 shot → ch 0x48).
    let mut cycles = 0;
    loop {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x58 { break; }
        assert!(cycles < 20, "the gadget cycle never reached the Q-Worm 0x58 (weapon {:#x})",
            w.scene.sim.as_ref().unwrap().gun.weapon);
        w.tick(nf_game::player::Input { gadget_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
        cycles += 1;
    }
    // The aim sits on the PC's own mesh (y ±0.34 about the placement [D the glist's collision
    // bounds]): the Q-Worm fires only with the PC first under the crosshair [M 0x1a5460:531-545].
    let shots = shoot_at(&mut w, [pc[0], pc[1] + 0.2, pc[2]], 3.0, 400, "the Q-Worm install (real, from the walked south stand)", &|sim| sim.drones.channels.on(0x48));
    w.wait(6);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x48),
        "ch 0x48 (objective 4) from the REAL install: {shots} shots from the walked south stand [M PCQWorm_Update 0x1aee20]");

    // The walk back to the lobby. The security keyboard is NOT pressed here: its collision faces
    // face −z, into the security room [D the entity glist 0x020005E1], and the room's only
    // walkable entrance is door #725 on its east wall (x ≈ 6.9 [D probe rays: the west part of
    // the floor and the east corridor are split by the wall x 6.75..7.0 from z −2 to z 17.5]),
    // locked on ch 0x31 [D #725 p3 = 49]. The ring reaches that door from the east corridor and
    // presses the keyboard from inside the room (`security_room_visit`).
    // The walk-back with the alarm-runner watch between the legs (the ring's own pattern): the
    // corridor legs pass the alarm-panel mouth (the ring's "east past the alarm-panel corridor"
    // at (−11.5, −13.5)), and a guard challenged en route runs for the panel while Bond walks on
    // — without the per-leg catch his press landed mid-keyboard-leg this round (level_exit at
    // +342 into "round the wall end", ch 0x61 still off at the walk-back's end) [M the runner
    // 0x168ea8's follow].
    // The walk-back is the outbound in reverse: the band east to the column, the jump north
    // over beam #621, the nav up to the west corridor south of beam #614, the jump north, the
    // north corridor, and beam #607's plane crouched on the doorway line.
    w.walk_to(Wp::new(-13.5, 0.4, -34.4).r(0.5), "the band east to the column (back)");
    // The northbound press widens 1.0 → 2.2: the walked route's phase (the #614 sweep-jump's
    // arm watch re-timed the ticks) shifts the band-west arrival ~0.66 east, and at press 1.0
    // the takeoff stand lands INSIDE #621's (x,z) footprint (x −14.8, z −33.4 [L the 621watch
    // round: 0x24 stamped exactly here]) — the band's record planes face the SOUTH side
    // [L: the southbound crossing from the north stays clean at the same footprint], so the
    // northbound raw refine reads it. Press 2.2 fires the jump at (−15.6, −34.4), 0.72 clear
    // of the band's west-south corner in 3D, and the arc's footprint ticks ride y-safe B ≥ 1.3.
    jump_over_low_beam(&mut w, BEAM_621, [-13.5, 0.4, -30.6], 2.2, "jump beam #621 northbound");
    let back = {
        let sim = w.scene.sim.as_ref().unwrap();
        // The final node holds SOUTH of the beam's swept band (the box floor 10.146, the
        // standing touch radius 0.55 [L the probe]: z ≤ 9.5 is clear at every sweep pose; the
        // old 10.6 sat inside the bottom park's exclusion and the approach stamped 0x22).
        let mut nodes = walker::nav_route(sim, sim.bond.pos, [-22.2, 0.4, 9.0]).expect("nav: the pocket → the west corridor");
        nodes.push([-22.2, 0.4, 9.0]);
        nodes
    };
    for (k, p) in back.iter().enumerate() {
        // Stealth-gated: the west-corridor civilian #6 (team 3, props 0x53/0x54 — no shot answers
        // him) patrols this lane and met the ungated walk at 1.5 u near (−22.8, 6.5); his run for
        // the panel then failed the mission. Bond holds each segment until nobody sees it.
        let ok = walk_leg_stealth(&mut w, *p, 1.3, &format!("back to the west corridor node {k}"));
        assert!(ok || k + 1 == back.len(), "the walk-back leg {k} never arrived (exit {:?})", w.scene.sim.as_ref().unwrap().drones.level_exit);
    }
    // The sweep-aware northbound crossing (see sweep_jump_beam614): the takeoff stand 9.7
    // behind the arm box_min ≥ 12.7 (the beam parked north), the arc lands ~14.45.
    sweep_jump_beam614(&mut w, false, -22.2, 0.4, "jump beam #614 northbound (the sweep)");
    for (wp, label) in [([-22.2f32, 0.4, 16.8], "the west corridor's head (back)"),
                        ([-16.6, 0.4, 17.4], "the north corridor east (back)"),
                        ([-11.6, 0.4, 16.6], "the west hall, west of beam #607 (back)")] {
        watch_and_clear(&mut w, 16.0);
        w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(0.6), label);
    }
    set_crouch(&mut w, true);
    w.walk_to(Wp::new(-10.0, 0.4, 16.6).r(0.6), "through beam #607's plane eastbound (crouched)");
    w.walk_to(Wp::new(-6.6, 0.4, 16.6).r(0.6), "east, clear of beam #607's reach (crouched)");
    set_crouch(&mut w, false);
    watch_and_clear(&mut w, 16.0);
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.0), "the lobby (back)");
    w.walk_to(Wp::new(-3.9, 0.0, 13.2).r(1.0), "the lobby's south door (back)");
    {
        // The walk-back's state gate (this round): the corridor legs pass the alarm-panel mouth,
        // and a guard challenged there runs for the panel while Bond finishes the walk — the
        // press must NOT land. The same catch the ring legs use (watch_and_clear →
        // clear_hostiles [M the alarm runner 0x168ea8's follow]): kill the runner before the
        // keyboard legs; the mission graph's ch 0x61 must still be off here.
        watch_and_clear(&mut w, 40.0);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[0a] post-walkback: ch61 {} fail {} exit {:?} pos {:?}", sim.drones.channels.on(0x61),
            sim.drones.mission.fail_label, sim.drones.level_exit, sim.bond.pos);
        assert!(!sim.drones.channels.on(0x61), "the alarm runner from the walk-back corridor was not caught in time");
        assert!(sim.drones.level_exit.is_none(), "the mission ended during the walk-back");
    }
    w
}

/// The office-floor prologue on real input: the arrival plate, the cab sweep, the walked PC loop,
/// the real Q-Worm install from the walked south stand, and the walk back to the lobby.
#[test]
fn walk_office_floor_qworm_install_and_back() {
    let w = walk_qworm_and_back();
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x48), "objective 4's ch 0x48 holds");
}

/// The full mission walk, GREEN end-to-end this round (2026-10-29): the prologue (the REAL
/// install from the walked south stand) → the keyboard → the RING through the game's own mouth —
/// the locked keypad door #724. The wall x −10.45..−10.70 seals the west offices from the atrium
/// lane from z −15.38 to −23.70 [D coll-scan: Room-L-atrium_top's west face x −10.45 spans
/// z −17.96..−15.38 and −21.62..−19.30 and −23.70..−21.62; the -3 round's probe rays] — the ONLY
/// opening is door #724's doorway (the jamb faces z −18.04/−19.36 [D], 1.32 wide) with the door
/// locked on ch 0x30, whose only writers are the kind-0 keypads #648/#732 (p1 = 1, p5 = 48
/// [D placements]) mounted at (−10.69/−10.45, 1.52, −17.46) facing the pocket [D rotations ±π/2].
/// The game's own ai graph crosses exactly there: the pocket → (−11.01,−18.60) → (−9.88,−18.62)
/// → node 0 (−8.76,−21.55) [D 0700000a ai_network]. The player's route, all real input: E the
/// keypad [M Player_Activate 0x1a5170 case 0x3A's facing gate → Lock_Activate 0x1aed00: class 3,
/// state 0/5, no pause manager → kind-0 no-key → state 3], type the 4 digits [M Lock_Update
/// 0x1ae3d8 case 4's match + C_KEYPAD_Handler 0x216810's digit handler], the match fires
/// `Lock_Unlock` 0x1b0650 → `Lock_Update` case 1 holds switch_channels[0x30] [M 0x1ae3d8 case 1]
/// → the door's rising-edge unlock + auto-activate [M 0x1363c8:101-107] → walk the doorway
/// (z −18.7, the capsule r 0.55 clears the 1.32 jamb gap by 0.11 a side [D rays]; the open leaf
/// parks clear — every eastward ray z −18.3..−19.15 clear with state 4 [D probe]) → the ai lane
/// south → the south corridor → the hatch E.
#[test]
fn walk_office_floor_qworm_keyboard_hatch() { walk_office_floor_qworm_keyboard_hatch_at(1) }

/// The walk body at `difficulty` (1 Operative = the old warm load's replay parity; 2 Agent rides
/// the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]).
/// The #614 southbound landing and the south-west lane's first legs (the ring's south mouth).
const LANE_MOUTH: [[f32; 3]; 3] = [[-22.2, 1.3, 9.2], [-22.18, 1.3, 6.5], [-23.04, 1.3, 2.48]];

/// The south mouth's patrol window, waited from the west offices' north corridor (out of the
/// west corridor's line). The office civilian g8 (team 3, sight 12, 180° cone [the alarm trace's
/// roster]) loops (−15.3, 6.5) → the mouth (−22, 6.3) → the west room (−26.4, 10.0) → (−35.9, 8.7)
/// → back east along z ≈ 4-5 → the mouth again (the port's walked trace, ~2400 ticks a lap).
/// At the game's crouch pace (0.039/tick [M Player_SSCrouch 0x1924c0:58 + the +0x10 → +0x20 copy,
/// Player_Update 0x193600:296-299]) the ring reaches #614 as he crosses the mouth: he sees the
/// north-of-beam stand down the corridor (d 9.9) or the landing (d 3.3) → 0x3e → PressAlarm. The
/// player's answer is to wait in cover until he has gone into the west room walking away, then
/// make the crossing behind his back: every live non-team-0 drone is either beyond sight range of
/// the mouth and the lane's first legs, or inside the west room (x < −26.5, z > 9) facing west
/// (the walker's yaw convention: facing x = sin(yaw)).
fn wait_mouth_window(w: &mut Walker) {
    let start = w.total_ticks;
    loop {
        let bad: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().enumerate().filter(|(_, d)| {
                d.present && !d.corpse && d.health > 0.0 && d.enabled && d.team != 0
                    && LANE_MOUTH.iter().any(|p| walker::dxz(d.pos, *p) < 12.0)
                    && !(d.pos[0] < -26.5 && d.pos[2] > 9.0 && d.yaw.sin() < 0.0)
            }).map(|(i, _)| i).collect()
        };
        if bad.is_empty() {
            eprintln!("[0a] the south mouth's window opened after {} ticks", w.total_ticks - start);
            return;
        }
        assert!(w.total_ticks - start < 6000, "no south-mouth window in 6000 ticks (in range {bad:?})");
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed waiting for the south mouth");
        w.wait(1);
    }
}

fn walk_office_floor_qworm_keyboard_hatch_at(difficulty: u32) {
    let mut w = walk_qworm_and_back_at(difficulty);

    // ai-sight-drawn-gate-5 route re-derivation (the ring's west legs, BFS-derived): the old
    // route crossed the LOCKED office door (−10.6, −19.3, ch 0x30) by force-driving the channel
    // — no writer for ch 0x30 exists in 0a's data [D trig/switch/script dumps] — and the door
    // exit sits 8.4 u due south of the panel-side challenge civilian (g15: prop 0x3c/0x1a armed,
    // sight rng 12, 180° cone, reaction stat 0 → 60 ticks [M 0x1787c8 + 0x175ef0; the roster
    // trace]), whose cone covers the whole corridor mouth; challenge → anim → RunToAlarm →
    // PressAlarm is game-true [M 0x162558 + 0x13ab58's +0x596 follow + 0x168ea8] and he is
    // PROTECTED (props 0x53/0x54 — the ACTION.ELF bitDescs/bitMasks decode) — a player cannot
    // clear him and cannot outrun 60 reaction ticks through his cone. The floor-sweep/BFS probe
    // (work/elf collision + world::segment_actor) gives the player's legal ring route: south
    // through the west offices and the west corridor (the legs below), then — THIS ROUND — the
    // south-west lane (the splice below), NOT the old ai-lane crossing.
    let ring: [([f32; 3], &str, bool); 7] = [
        ([-7.5, 1.3, 14.5], "ring: the security centre's west door", false),
        ([-7.5, 1.3, 16.5], "ring: the west hall north", false),
        // Beam #607's plane (x −8.45, z 15-20.25 [D triggers dump]) spans this corridor: the
        // ring crosses it CROUCHED — the under-beam pass (the crouched pick arm tops ≈ feet+1.26
        // [M Player_Collision 0x194df0 case 3] vs the beam band 1.49-1.76 [D]; standing 1.86
        // [M 0x194df0 default arm] crosses and arms gun #640 through OR #667 → ch 0x0F).
        // The crouched leg ends 2+ west of the plane (live, the standing body fired 0x1F at
        // x −8.95 — 0.36 off the west face [L 2026-11-01]): the stand-up happens out of reach.
        ([-11.4, 1.3, 16.6], "ring: crouched through beam #607's plane", true),
        ([-16.6, 1.3, 17.4], "ring: the west offices' north corridor", false),
        ([-22.2, 1.3, 16.6], "ring: the west corridor's head", false),
        ([-22.2, 1.3, 15.4], "ring: the west corridor, north of beam #614", false),
        // Beam #614 (out ch 0x22, OR #667 → ch 0x0F → gun #640) spans the corridor wall to
        // wall at ankle height — jumped (jump_over_low_beam, keyed on this label).
        // The sweep-aware southbound crossing (see sweep_jump_beam614): the arc lands ~9.15,
        // past the whole swept band.
        ([-22.2, 1.3, 9.2], "ring: sweep-jump beam #614", false),
    ];
    for (wp, label, crouched) in ring {
        // The alarm guard's cone covers the south corridor mouth (the route note above) — the
        // watch between the legs catches his run the way walk_leg_watched does below.
        let jump = label == "ring: jump beam #614";
        let sweep = label == "ring: sweep-jump beam #614";
        if crouched && !jump && !sweep { set_crouch(&mut w, true); }
        watch_and_clear(&mut w, 16.0);
        if label == "ring: the west corridor's head" { wait_mouth_window(&mut w); }
        if sweep {
            sweep_jump_beam614(&mut w, true, wp[0], wp[1], label);
        }
        else if jump { jump_over_low_beam(&mut w, BEAM_614, wp, 1.0, label); }
        else { w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(0.8), label); }
        if crouched && !jump && !sweep { set_crouch(&mut w, false); }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let chs: Vec<String> = [0x1f, 0x20, 0x21, 0x22, 0x24, 0x25, 0x26, 0x0f, 0x10, 0x11, 0x12, 0x81]
                .iter().filter(|c| sim.drones.channels.time[**c as usize] != 0).map(|c| format!("{c:#04x}")).collect();
            let tur: Vec<String> = sim.drones.gt_turrets.iter()
                .map(|t| format!("{}:s{}", t.placement, t.state)).collect();
            eprintln!("[ring] after '{label}' pos ({:.2},{:.2},{:.2}) fired [{}] turrets [{}]",
                sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], chs.join(","), tur.join(","));
        }
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed at '{label}'");
    }
    // THE SOUTH-WEST LANE RING (this round's re-route, the alarm leak's measured fix): the old
    // ring crossed the central ai lane (the old node-17..34 legs above) and g7 — the patrolling
    // office civilian (skin 0x500002a, team 3, sight 12, 180° cone) whose patrol lane IS the
    // route's own ai lane — challenged Bond there at t11812 (d 3.3, seen true [L 2026-11-05
    // trace]); the shout chain alerted g0 ONE tick later (t11813) and g0 ran 12.6 u to the
    // alarm panel (−21, 10), pressed at t12163, and the press timer landed the fail 0x4000032
    // at t12400 [L this run; M 0x168ea8's timer C]. The civilians cannot be dropped (props
    // 0x53/0x54 — a player kill is fail reason 3 [M 0x145bd0]) and a started press cannot be
    // undone — so the ai-lane crossing is REMOVED. The ring rides the outbound's own
    // proven-quiet lane instead: walk_qworm_and_back's PC nav ("to the Q-Worm PC" nodes)
    // and the walk-back's return nav both ran this exact lane UNSEEN on the same branch [L
    // 2026-11-05: the arrived node positions below, e.g. (−22.18, 6.50) → (−26.46, −3.93) →
    // (−16.88, −15.65)]. The lane runs WEST of g7's diamond (his pauses (−25.6, 7.5) /
    // (−20.6, 5.3) and his east excursion to (−13.1, 2.3) all sit east of x −26 while the lane
    // is southbound), and every leg holds the stealth gate (wait_sight_clear — nobody sees the
    // start or the end before it walks [M 0x175ef0's sight triple]). The pocket tail keeps the
    // old ring's pillar-crossing note: (−14.5, −15.0) rides south of every pillar face (the
    // pillar's south face z −13.10, x −12.98..−10.9 [D probe rays]), then the keypad's wall.
    let lane: [([f32; 3], &str); 9] = [
        ([-22.12, 1.3, 8.6], "lane: the west corridor's south mouth"),
        ([-22.18, 1.3, 6.5], "lane: south of the office block's west face"),
        ([-23.04, 1.3, 2.48], "lane: the west lane's mid"),
        ([-25.99, 1.3, 0.48], "lane: the west lane's south-west corner"),
        ([-26.46, 1.3, -3.93], "lane: the west offices' south lane"),
        ([-26.06, 1.3, -7.36], "lane: the west lane's south end"),
        ([-22.21, 1.3, -11.97], "lane: the cut east"),
        ([-16.88, 1.3, -15.65], "lane: the pocket's west reach"),
        ([-14.5, 1.3, -15.0], "ring: south of the pillar, into the pocket"),
    ];
    for (k, (wp, label)) in lane.iter().enumerate() {
        let ok = walk_leg_stealth(&mut w, *wp, 1.3, label);
        assert!(ok || w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(),
            "the south-west lane leg {k} ({label}) never arrived (exit {:?})", w.scene.sim.as_ref().unwrap().drones.level_exit);
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed at '{label}'");
        {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[lane] after '{label}' pos ({:.2},{:.2},{:.2}) fail {:#x} exit {:?}",
                sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.drones.mission.fail_label, sim.drones.level_exit);
        }
    }
    // The keypad approach (this round's walk): the mouth is the locked door #724's doorway —
    // see the test doc. The keypad #648 (−10.69, 1.52, −17.46 [D]) faces the pocket
    // (facing (−1,0,0) [D rotation −π/2]). THE POCKET TURRET (measured this round, [L the
    // hit-source probe on this branch]): the kills are NOT a world volume — they are **GT
    // #639's minigun rounds** (weapon record 96 = TURRET_WEAPON_RECORD, owner usize::MAX = the
    // not-a-drone mark [M 17828's GT Fire arm]; 2.4000015 a hit at d1 = the record's 5.0 × the
    // d1 pain factor 0.48; the turret sits at (−21.98, 2.28, −11.31) and reads state 4 COMBAT
    // while Bond stands at (−12.7,−16.9)). The lane (x −26..−14) and the pocket's west half
    // took ZERO hits on every run — the turret's track only opens fire from the keypad
    // approach's last ~2 u (its first hit landed ~120 ticks after the wedge began). So: the
    // approach hooks SOUTH-EAST first (the plain walk's own doorway path (−12.73,−16.90) →
    // (−11.53,−18.11) proved that floor walkable — the direct north-west line to the stand
    // wedges on the console at 1.0 u out and grinds the seg loop in the turret's line), the
    // stand is taken from the south-east, and the E-arm → digits → door-wait all run from
    // COVER (see below).
    {
        // The ORIGINAL stand line (−11.9,−17.5): the seg walk CONVERGES to (−12.73,−16.90)
        // by seg 1-2 on every run (the furniture wedges the slices ~1 u short of the far
        // target — the exact spot is target-dependent, and this target's wedge is the proven
        // E-reach stand: the plain walk armed the keypad from it, dxz 2.11 [L this branch]).
        // The cap stops the grind there — 200 segs of it stood Bond in #639's fire arc for
        // ~400 ticks (the sisters' 42-hit death); 4 segs = the wedge spot + ~20 ticks.
        let ok = walk_leg_stealth_cap(&mut w, [-11.9, 1.3, -17.5], 0.8, "ring: the keypad's wall", 4);
        assert!(ok || w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(),
            "the keypad's wall leg never arrived (exit {:?})", w.scene.sim.as_ref().unwrap().drones.level_exit);
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed at 'ring: the keypad's wall'");
    }
    // THE MOUTH, through the game's own writer chain (all real input — see the test doc):
    // aim the view at the keypad's world sphere and pulse the use key — the class-0x3A arm's
    // facing gate + sphere pick arm the keypad (state 3) [M 0x1a5170:158-166 + 0x1aed00].
    let kp: [f32; 3] = {
        let sim = w.scene.sim.as_ref().unwrap();
        // The keypad's posed world-sphere centre (use-pick: the local centre through the lock's
        // matrix [M Control_BuildWorldSph 0x133070]; `Lock.pos` adds the centre unrotated).
        let li = sim.drones.locks.iter().position(|l| l.placement == 648).expect("keypad #648 [D]");
        sim.drones.use_picks.locks[li].sphere.0
    };
    let _ = kp;
    // use-pick: from the wall leg's wedge the use point does not reach within 1.0 in front of the
    // keypad's faces [M Collide_Intersect 0x1ebaa8 → Intersect_PointGeom 0x1e92d8;
    // nf_game::usepick] — Bond steps to the first stand in front of it (the lock arm's facing
    // gate [M 0x1a5170:158-166] refuses the ones behind it) and presses there.
    let armed = {
        let sim = w.scene.sim.as_mut().unwrap();
        let li = sim.drones.locks.iter().position(|l| l.placement == 648).expect("keypad #648 [D]");
        let t = sim.drones.use_picks.locks[li];
        let tris = usepick::posed(sim, t.hash, &t.matrix);
        // Only #648's keypad counts: a stand whose press arms a neighbour's is backed out of.
        usepick::for_each_stand(sim, &tris, t.sphere, &mut |sim| {
            let hit = usepick::pulse(sim, 12, &|s| s.drones.keypad.is_some());
            if hit && sim.drones.keypad.as_ref().is_some_and(|(k, _)| *k != li) { sim.drones.keypad = None; return false; }
            hit
        })
    };
    assert!(armed, "the keypad #648 E arms the mini-game (state 3) [M 0x1aed00's kind-0 no-key branch]");
    // THE DIGITS RUN AT THE WEDGE, NO RETREAT (civ-unignore, the 00agent fix): the old
    // "out of the turret's line" retreat to (−14.5,−15.0) bought nothing any more — the
    // civilians-raise-the-alarm merge (576909ed6) left the whole pocket LOS-clear to wall gun
    // #639 under the game's own track rule (probe, the game's `line_of_sight(turret.pos, p,
    // LOS_TURRET_TRACK)` + the `dot(fire_dir, dir) ≥ 0.01` cone [M gt.rs GT_Track]: every
    // pocket cell x −17..−10.5, z −20..−15 reads clear; #639's fire_dir is (0,−1,0), so the
    // cone passes every floor point), and the measured 00agent run took the retreat's own
    // seg 0 for 11 rounds (69 ticks) — the retreat is now part of the exposure, not the cure.
    // The deadly window is #639's COMBAT state, which starts ~script-timed after its first
    // track (the d2/agent ledger: first hit t12423, the retreat's seg 0) and ends when the
    // LOS finally blocks east of x ≈ −10.0 (the same probe; the d2 ledger's hits stop at
    // (−9.8,−18.6)). So the tail is cut to the minimum: the digits type AT the wedge (the
    // armed keypad is a MENU state — position-independent [M C_KEYPAD_Handler 0x216810 +
    // 0x1ae3d8 case 4]), the door sweep check is the ~2-tick gate it measures at, and the
    // doorway crossing runs straight from the wedge — the whole post-arm tail ends inside
    // #639's first ~20 COMBAT rounds (the measured 00agent ledger after this change: 21
    // rounds for 84.0, first hit t12409 at (−12.3,−17.2), the last t12549 at (−9.8,−18.6)
    // where the LOS blocks — end health 16.0; the d1/d2 sisters' ledgers shrink the same
    // way, the retreat's 11-round seg 0 is gone).
    // The 4 digits [M C_KEYPAD_Handler 0x216810 msg 0x4b — the handler the keypad menu's digit
    // controls dispatch; the cursor/page UI is the view's presentation (GAP-1's UI half, the
    // audit §5)]. The code is the game's own boot table [L ee_start_0700000a.bin @0x2beff8 —
    // KeyCodes, code index 1 = the locks' p1], loaded over the port's local-stream default in
    // `load_carry`.
    let code = {
        let sim = w.scene.sim.as_ref().unwrap();
        let idx = sim.drones.locks.iter().find(|l| l.placement == 648).expect("keypad #648").code;
        sim.drones.key_codes.code(idx)
    };
    for d in code {
        assert!(d >= b'0' && d <= b'9', "the snapshot code digits are ASCII [L 0x2beff8]");
        w.scene.sim.as_mut().unwrap().drones.keypad_digit((d - b'0') as u32);
        w.wait(1);
    }
    // The match fires Lock_Unlock [M 0x1ae3d8 case 4] and the NEXT case-1 tick holds
    // switch_channels[0x30] [M case 1] — the door's rising-edge unlock + auto-activate
    // [M 0x1363c8:101-107]. Wait the door's sweep out (state 4), the walked crossing rides the
    // open doorway (every eastward ray z −18.3..−19.15 clear at state 4 [D probe]).
    {
        let mut ok = false;
        for _ in 0..120 {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.doors.list.iter().find(|d| d.placement == 724).expect("door #724 [D]");
            if sim.drones.channels.on(0x30) && d.state == 4 { ok = true; break; }
            w.wait(1);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(ok, "the code match unlocks ch 0x30 [M 0x1ae3d8 case 4 → case 1] and the door auto-opens [M 0x1363c8:101-107]");
        assert!(sim.drones.locks.iter().filter(|l| l.placement == 648).all(|l| l.unlocked), "the used lock's state 1 [M 0x1b0650]");
    }
    // The crossing + the lane south (this round's walked legs):
    const RING2_LEGS: usize = 9;
    let ring2: [([f32; 3], &str); RING2_LEGS] = [
        ([-11.0, 1.3, -18.7], "ring: the doorway west"),
        ([-9.6, 1.3, -18.7], "ring: through the doorway, the atrium lane"),
        // The player's route south rides the game's N-S lane at x −8.5 (the ai nodes 132
        // (−8.52, −18.47) / 0 (−8.76, −21.55) [D 0700000a ai_network]) — the mouth's guard
        // slot east of the bridge is 0.5 wide (the jamb's corner at (−10.60, −18.66) [L the
        // bridge-edge sister's segment probe]) and cannot take the player's 1.1 capsule.
        ([-8.5, 1.3, -18.5], "ring: the ai node 132 south"),
        ([-8.7, 1.3, -21.5], "ring: the ai node 0 into the south corridor"),
        ([-6.5, 1.3, -22.5], "ring: the south corridor"),
        ([9.5, 1.3, -22.5], "ring: the south corridor's east run"),
        // THE EAST-STRIP RACE (known residual, this round): g17 (the east-side patrol civilian,
        // spawn (24.6, −6.8) [L 2026-11-05 trace]) walks a SW leg that CONVERGES on the strip —
        // at his closest observed patrol point (20.6, −12.8) the strip mouth is ~11 u inside his
        // 12-unit sight with the game's LOS clear, and his sighting here cascades through the
        // shout chain (g2 pressed the south-west panel at t14796 → fail 0x4000032 [L]). The
        // stealth gate's world-segment probe reads the same ray BLOCKED at every height
        // [L 2026-11-05: the t14717 sighting repeated identically with the gate live], so the
        // gate cannot see this exposure — the crossing is left to the watched legs + the
        // watch_and_clear chase, and the report asks for a live read of g17's full patrol cycle
        // (the window is his NE return, his back to the west lane).
        ([9.5, 1.3, -13.5], "ring: up the east side south"),
        ([10.6, 1.3, -13.9], "ring: the south gap east"),
        ([15.7, 1.3, -13.9], "ring: before the laser fence"),
    ];
    for (wp, label) in ring2 {
        // The alarm guard's cone covers the south corridor mouth (the route note above) — the
        // LONG runs walk in segments with the watch BETWEEN them (the east-side boundary-only
        // watch let a runner past the 40-unit band: the press landed mid-leg at +446); the short
        // doorway/jink legs keep the precise r-0.8 walk_to (a 1.2-arrive stall here stands Bond
        // in the open for 1200 ticks — exactly what invites the runner). KNOWN RESIDUAL (this
        // round): the east-strip legs carry a racy civilian sighting (see the hold-leg note) —
        // the live-check ask in the report covers timing it.
        // THE STRIP WINDOW (measured this round): the plain walk's racy g17 sighting became a
        // BEAM TOUCH under the real-damage run's longer keypad timeline — a converging runner
        // crossed the web and the alert OR #670 wrote ch 0x12 (the laser pin popped at the
        // hatch, Bond alive at 28 hp [L this run]). The hold: cross the strip only while no
        // team-3 drone is within 12.5 u of Bond or the strip's leg targets — g17's NE return
        // (spawn (24.6, −6.8) → 15.6 u from the strip mouth) is that window; his SW beat
        // (20.6, −12.8) holds the gate. Distance-only — the segment probe over-blocks here.
        if label.contains("up the east side south") {
            wait_distance_clear(&mut w,
                &[[9.5, 1.3, -13.5], [10.6, 1.3, -13.9], [15.7, 1.3, -13.9]], 12.5, 3600,
                "the east strip crossing");
        }
        watch_and_clear(&mut w, 16.0);
        let here = w.scene.sim.as_ref().unwrap().bond.pos;
        let dist = walker::dxz(here, wp);
        if dist > 5.0 {
            let _ = walk_leg_watched(&mut w, wp, 1.3, label);
        } else {
            w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(0.8), label);
        }
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed at '{label}'");
    }
    // The 0x25 laser fence (#651-654: four stacked rows y 0.22..2.34 across the north-east lane,
    // diagonal (18.92, −14.06) → (23.12, −10.47) [D triggers dump]) is the AI graph's only way to
    // the hatch (the lane (15.74, −13.85) → (20.06, −13.19) → (23.16, −8.87) [D nav]; the office
    // north of it is a dead end in the graph). Its rows carry lock ch 0x19 [D p6], held while a
    // guard stands on his drone plate (OR #634: 0x19 ← [0x18, 0x37] [D trig dump]; a locked
    // trigger does not fire [M Trigger_Locked 0x1b2fc8:24-29, the touch path's gate]) — the player crosses
    // while the fence is locked. The lock is held only while the guard is on his plate (the port's
    // trace: 0x18/0x19 on for ~56 ticks as he walks through [S] unverified live), so the wait is AT the fence:
    // the lane's last node before it (20.1, −13.2) sits 1.29 west of the fence line (posts
    // (22.93, −13.89) / (19.20, −10.76) [D #655/#656]); the crossing to (23.2, −8.9) runs
    // near-perpendicular to the line and clears its 0.55 reach in ~20 ticks.
    w.walk_to(Wp::new(20.1, 1.3, -13.2).r(0.4), "ring: at the laser fence");
    let mut locked = false;
    for _ in 0..9000u32 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x19) { locked = true; break; }
        w.wait(1);
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed waiting at the laser fence");
    }
    assert!(locked, "the laser fence's lock ch 0x19 never came on (no guard on the fence plate)");
    eprintln!("[0a] fence lock 0x19 on at tick {}", w.total_ticks);
    for (wp, label) in [([23.2f32, 1.3, -8.9], "ring: through the locked laser fence"),
                        ([25.5, 1.3, -3.2], "ring: the east corridor"),
                        ([25.9, 1.3, 5.2], "ring: up the east corridor"),
                        ([21.3, 1.3, 5.3], "ring: the corridor west, east of beam #625")] {
        w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(0.6), label);
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed at '{label}'");
        if label == "ring: the east corridor" {
            // The east corridor (25.3, −3.7) is out of sight of the z ≈ 5 corridor. The spawner
            // that releases the antechamber civilian (#709, group 6, one of templates 3/18/19 by
            // the draw [M DroneSpawner_SpawnDrone 0x137cb0]) can put a patroller on the corridor
            // and the security-room keypad lane; Bond waits here until the patrol is not facing
            // or walking toward the corridor, the keypad and the room door.
            wait_patrol_clear(&mut w, &ROOM_APPROACH, 20.0, 9000, "the corridor to the security room");
        }
    }
    // Beam #625 (out 0x26, x 18.88, z 2.38-7.64, y 0.39-0.65 [D]) — a low line across the
    // corridor: jumped, sweep-aware (the band sweeps x 18.877 <=> 16.078 [D keys] — see
    // sweep_jump_beam625; the old blind press landed the arc's low tail inside the west park's
    // reach once the walk's upstream timing shifted).
    sweep_jump_beam625(&mut w, "ring: jump beam #625 westbound (the sweep)");
    // The north corridor legs are g3's patrol lane (the hatch-antechamber civilian, spawn
    // (9.8, 21.3) [D level doc; L 2026-11-05 trace: he walked south to (9.9, 10.1) and
    // challenged the ungated walk at d 4.9, t15305] — his press was also only beaten by the
    // walk's finish; same residual race as the strip, same live-check ask).
    watch_and_clear(&mut w, 16.0);
    w.walk_to(Wp::new(9.9, 1.3, 5.3).r(0.8), "ring: the corridor west end");
    assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed at 'ring: the corridor west end'");
    // The security room: in through door #725, the keyboard, and back out (objective 5).
    security_room_visit(&mut w);
    w.walk_to(Wp::new(9.9, 1.3, 5.3).r(0.8), "ring: the corridor west end (back)");
    // Security camera #627 ((7.52, 2.76, 17.43), out ch 0x1E → alert OR #670 → ch 0x12 [D]) sweeps
    // the north lane and the anteroom legs: a 7.5° cone, range 7 (cos_half 0.99144, the
    // (f32) param 2 [M Sensor_Create 0x1af010:63-73]), four keys over 600 ticks — south (down
    // the lane) at playhead ~1.0, east (the anteroom leg) at ~0 and ~2, north at ~3 [port trace
    // of the sweep]. One sighting starts the track and the trip follows 2 s later [M
    // Sensor_Update 0x1af618:154-157]. A player waits here, out of its range, for the sweep to
    // swing off the lane, then walks the lane and the anteroom leg behind it. Measured on this
    // route: leaving at playhead 1.0-3.5 passes unseen, 0.0-0.6 trips it.
    wait_sensor_phase(&mut w, 627, 2.0, 2.6, "camera #627 off the north lane");
    for (wp, label) in [([9.9f32, 1.3, 5.3], "ring: the corridor west end (back)"),
                        ([9.75, 1.3, 8.8], "ring: up the east corridor north"),
                        ([9.6, 1.3, 12.3], "ring: up the east corridor north 2"),
                        ([11.5, 1.3, 15.5], "ring: the east corridor's north turn"),
                        ([17.5, 1.3, 15.5], "ring: east to the hatch anteroom's south"),
                        ([17.5, 1.3, 16.5], "ring: into the anteroom"),
                        ([21.5, 1.3, 16.5], "ring: the anteroom east"),
                        ([21.5, 1.3, 19.4], "ring: the hatch mouth")] {
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
    {
        // The route's laser pin: the walk passed every beam without a touch — #607 crouched,
        // the low lines #614/#621/#625 jumped, the 0x25 fence while its lock 0x19 held — so no
        // mask-8 laser out (0x1F/0x20/0x21/0x22/0x24/0x25/0x26 [D 0700000a triggers dump])
        // fired, and through the laser-fed alert ORs (#667 0x0F ← [0x1F,0x20,0x21,0x22]; #669
        // 0x11 ← [0x24,0x25]; #670 0x12 ← [0x1E,0x25,0x26]; #782 → 0x81 [D trig dump, M
        // Trigger_MultiplexOrIn 0x1b2d38]) no gun armed by a beam — #640's enable is ch 0x0F
        // (GT placement param 4 [D 0700000a-audit]). #668's 0x10 ← [0x1D, 0x24] is left out: its
        // other input 0x1D is a security sensor's out (the walk-back's west corridor passes one
        // [port trace: Sensor::update writes 0x1D]); its beam input 0x24 is pinned here.
        // The beam outs are PULSES (live: 0x1F/0x0F and 0x24/0x10 read 0 again ~30 frames after
        // the crossing [L 2026-11-01]), so the pin reads the write stamp, not the level: the
        // game stamps switch_channels_time on every nonzero write [M 0x1b1a78:87,107,188-189] —
        // a never-stamped channel never fired.
        let sim = w.scene.sim.as_ref().unwrap();
        for ch in [0x1f, 0x20, 0x21, 0x22, 0x24, 0x25, 0x26, 0x0f, 0x11, 0x12, 0x81] {
            assert!(sim.drones.channels.time[ch] == 0 && !sim.drones.channels.on(ch as u32),
                "laser/alert channel {ch:#x} fired on the walked route — a beam was touched");
        }
        eprintln!("[0a] laser pin: no mask-8 beam out, no alert OR out — gun #640 never armed");
    }
    w.walk_to(Wp::at(21.72, 1.12, 19.79, Act::Use { use_ticks: 600 }), "E the hatch");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.level_exit, Some(0x0700_000b), "the hatch loads 0700000b [D]");
        // The hatch voice plate (#757, out ch 0xB7, lock ch 0x3A [D]): the game's lock DISABLES a
        // trigger while its channel is on [M Trigger_Update's `lock != 0 && ch.on(lock)` gate] —
        // the plate plays the May-Hatch line on EARLY approaches (ch 0x3A still off) and is dead
        // once the keyboard is done, which is the state this walk arrives in.
        assert!(!sim.drones.channels.on(0xb7), "the voice plate #757 is lock-disabled once ch 0x3A is on [M triggers.rs's lock gate]");
        // The keyboard-lasers switch (#643, out ch 0x3B, gate ch 0x3C [D]) is usable (nothing
        // writes its gate) but this route never presses it: keyboard_lasers_0a_live covers the press.
        assert!(!sim.drones.channels.on(0x3b), "the walk leaves the laser keyboard alone");
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
/// 2026-10-31 session (level-0a-ignored-checks) — the decompile side of the press-y hunt: the
/// walk path has NO step-up and NO wall-stop — the move integrates unclipped [M Player_Update
/// 0x193600:284-295] and the cyl push tail is dead on 0a fresh boot: the ref HITTEST+0x50 is the
/// lifetime DeltaP accumulator, boots (0,0,0) [L ee_start_0700000a 0x1b78ee0 = 00×16] and every
/// walked contact sits |closest| ≈ 10-25 from the origin ≫ r 0.55, so the tail never fires and
/// pos += BL+0x750 adds zero [M 0x1949a8:131-140; _DeltaP zeroed per call 0x109da0:136, the
/// add-back the only +0x50 writer 0x1ebaa8:353-373]; the head-on cull is inert on the raw walk
/// spine denom (0.7597 < 0.999 [M 0x109da0:321-322]). On a fresh boot the press-y therefore
/// cannot come from CylGeom at all; the remaining y writers on a walked line are the FeetOnPoint
/// band [M 0x194188:106] and the walk-anim root ride (cc read fresh per tick [M 0x194df0:86] —
/// the port's SP anim set is the 2-frame stub [D 0x2b91b0]). Named live checks: (1) dump
/// BL+0x790 per frame through the doorway brush — pins the dead tail on 0a; (2) trace the
/// walk-anim root motion → AnimObject+0xcc during the walked glide — the +0.0256 candidate.
#[test]
#[ignore = "the through-line crosses the doorway on every measured tree, but the settled y never reaches the game's 1.1285 [L 2026-10-26]: 1.1201 on the settle-5 radial-tail tree, 1.0905 on master (the press-y is 0.0000 there — every doorway contact pushes horizontally, NF_CAPS_LOG trace this round). The walked-dy growth is NOT any statically-pinned writer: the walk move is yaw-only (Player_Move 0x191dd0 writes +0x10/+0x18, the walk matrix RotMatrix(BL+0x30) reads the pitch lane ZEROED at Player_Update 0x193600:134 — the look writes +0x40/+0x8a8 [M]); the SP body anim set is the static 2-frame stub (set 0x2b91b0 = all 0x06000a00 → seq 0x84f, 2 frames [D ACTION.ELF @0x2b91b0 + the script dump]) so no anim bob rides the walk; gravity is band-gated off [M 0x194df0:104]. Decompile side (2026-10-31): no step-up and no wall-stop exist — the push tail is dead on 0a fresh boot (ref HITTEST+0x50 boots (0,0,0) [L ee_start_0700000a 0x1b78ee0 = 00×16], every walked contact |closest| ≈ 10-25 ≫ r 0.55, _DeltaP zeroed per call [M 0x109da0:136], add-back only [M 0x1ebaa8:353-373]), so the press-y cannot come from CylGeom on a fresh boot; the remaining y writers are the FeetOnPoint band [M 0x194188:106] and the walk-anim root ride (cc fresh per tick [M 0x194df0:86]). Named live checks: (1) dump BL+0x790 per frame through the doorway brush; (2) trace the walk-anim root motion → AnimObject+0xcc during the walked glide — the +0.0256 candidate"]
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
    // passed x -9.0106 at frame 36. The slide itself crosses on master too (230 ticks to
    // (-9.93, 17.08), the [L] rate); the settled y stays short (the ignore note carries the
    // ruled-out writers and the open ref live check, level-0a-hall-glide-settle) — the test
    // stays ignored. 2026-10-31: the decompile side is now read (no step-up/wall-stop, the
    // push tail dead on 0a fresh boot — the doc's session paragraph); the press-y's remaining
    // writers are the band and the walk-anim root ride, both named live checks. The note is
    // re-pinned with them.
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
///
/// 2026-10-30 session: the mechanism is the game's MOVE CLIP, not a push. With NF_WALK_CLIP=1 the
/// corner CROSSES at +76 ticks, arrived (−17.56, 1.09, 2.83) — the walk-speed crossing, no kill
/// switch [L measured this session; the game's ~60 frames]. The exit-y pin still reads 1.0891 vs
/// the settled 1.1159 — the 0.0268 gap is the west-hall leg's unported +0.0256 climb settle (the
/// glide sister's mechanism), not the corner. Decompile side: the cyl push tail v = p −
/// HITTEST+0x50 [M 0x109da0:419-457] cannot sustain the walk's wall response (p is spine-clamped,
/// so for a vertical walk spine any on-spine ref gives a purely vertical v whose dot(v,n) > 0 gate
/// kills vertical-wall pushes; an ELF-wide store sweep finds no BLData+0x790/0x7a0 writer at all
/// beyond the add-back 0x1ebaa8:353-373), and the clip's spine-plane rest matches the game's
/// ~0.011-inside-the-plane glide rest [L fresh boot]. The flip is not landed: under the flag
/// the_pick_ref_accumulator_tracks_the_game_ref, walk_office_floor_qworm_keyboard_hatch and
/// walk_office_floor_to_the_security_keyboard go red (master: 7 passed / 0 failed / 2 ignored).
///
/// 2026-10-31 session CORRECTION (level-0a-ignored-checks): the 2026-10-30 crossing reading ran on
/// a DEAD flag path — pre_move_pos was captured after the walk add (the fix hoists it before every
/// position move; contacts read the post-move root [M Player_Update 0x193600:284-295]), so
/// move_v = (0, gravity, 0) and the clip never engaged: the "crossing" was wall-response-less
/// ghosting. Re-measured with the capture fixed: NF_WALK_CLIP=1 now WEDGES the west-hall
/// through-line at the doorway (STUCK (−8.28, 1.03, 15.83), y decaying 1.03→0.71) — the clip model
/// does not reproduce the doorway slide even under its own flag. Decompile side the walk path has
/// NO clip and NO wall-stop at all: the move integrates unclipped [M 0x193600:284-295], the cyl
/// push tail is dead on 0a fresh boot (ref = HITTEST+0x50, the lifetime DeltaP accumulator, boots
/// (0,0,0) and every walked contact sits |closest| ≈ 10-25 away ≫ r 0.55 [L ee_start_0700000a
/// 0x1b78ee0 = 00×16; _DeltaP zeroed per call 0x109da0:136, add-back only 0x1ebaa8:353-373]), the
/// head-on cull is inert on the raw walk spine denom (0.7597 < 0.999 [M 0x109da0:321-322]) and the
/// only vertical mechanism is the FeetOnPoint band [M 0x194188:106]. The game slides the corner
/// (xz bends) and settles 1.1159 [L 2026-10-26]; what produces the game's walked wall response is
/// the open live check (BL+0x790 watch through the corner + doorway brush).
#[test]
#[ignore = "the crossing mechanism is NOT the port's move-clip: the 2026-10-30 'crosses at +76 ticks with NF_WALK_CLIP=1' ran on a DEAD flag path (pre_move_pos captured after the walk add → move_v = (0, gravity, 0) → the clip never engaged — the crossing was wall-response-less ghosting); re-measured 2026-10-31 with the capture fixed and the flag WEDGES the west-hall through-line at the doorway (STUCK (−8.28, 1.03, 15.83), y decaying). Decompile side the walk path has no clip and no wall-stop: the move integrates unclipped [M 0x193600:284-295], the cyl push tail is dead on 0a (ref = HITTEST+0x50, the lifetime DeltaP accumulator [M 0x1ebaa8:353-373], boots (0,0,0) [L ee_start_0700000a 0x1b78ee0 = 00×16], every walked contact |closest| ≈ 10-25 ≫ r 0.55), the head-on cull is inert on the raw walk spine denom (0.7597 < 0.999 [M 0x109da0:321-322]) and the only vertical mechanism is the FeetOnPoint band [M 0x194188:106]. The game slides the corner and settles 1.1159 [L 2026-10-26]; the port wedges on master. The open live check is the game's walked wall response: watch BL+0x790 through the corner + doorway brush (pins the dead tail) and find the mechanism; the climb settle is the glide sister's live check"]
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
///
/// guard-standing-capsule-2: the capsule push rode the fresh-load crossing onto civilian g8 —
/// his lane wanders further west than the roster note (the trace caught him at (−20.3..−21.2,
/// 5.2..7.1)), his desks blind Bond's leg-3 end ONLY from east of the corner (t803 d 2.3 seen
/// false los false from (−18.6,5.4); t890 d 1.4 seen true los true from (−20.3,5.3) [L the
/// NF_ALARM_TRACE runs]) and no static window outwaits a 90° cone that faces every approach.
/// The repair is the sister's own phase: `walk_qworm_and_back`'s prologue delays the lane
/// crossing to where g8's patrol keeps him out of sight the whole way [L bond_stands_at_the_
/// mouth_bridge_edge passes on this tree], and the lane legs below then match the sister's ring
/// verbatim (spots, arrive radii, the mouth window, the watch) so the shared RNG — and g8's
/// phase at the crossing — is the proven one. The leg SET (and the floor readings) is unchanged.
#[test]
fn west_doorway_repro() {
    let mut w = walk_qworm_and_back();
    let legs: [([f32; 3], &str); 15] = [
        ([-7.5, 1.3, 14.5], "ring: the security centre's west door"),
        ([-7.5, 1.3, 16.5], "ring: the west hall north"),
        ([-20.5, 1.3, 16.5], "ring: the west offices' north corridor"),
        ([-20.5, 1.3, 5.5], "ring: down the west office block"),
        ([-17.6, 1.3, 5.5], "ring: the ai node 17 east of the door"),
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
        // The sister ring's own mechanics at the same points (the phase is the proof).
        watch_and_clear(&mut w, 16.0);
        if label == "ring: down the west office block" { wait_mouth_window(&mut w); }
        w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(0.8), label);
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let fl = sim.world.floor_below(p, 3.0);
        eprintln!("[repro] after '{label}': pos ({:.2}, {:.2}, {:.2}) floor {fl:?} falls {}",
            p[0], p[1], p[2], w.report.falls.len());
        for f in &w.report.falls { eprintln!("[repro]   fall: {f}"); }
        assert!(sim.drones.level_exit.is_none(), "the mission failed at '{label}'");
    }
}

/// The band-edge sister: Bond WALKS onto the south-corridor mouth's floor bridge (the map37
/// mesh strip the instances fix folded in — x −11.43..−10.39, z −19.36..−18.03 [D the disc tri
/// dump]) and stands AT its west edge. The floor band [M Player_FeetOnPoint 0x194188:106] holds
/// him grounded at the edge — the honest edge case the floating wedge was not: no box grazes, the
/// band passes, and he stays put. CAN FAIL if the band constants drift or the bridge drops out
/// again (the instances fix's regression pin).
///
/// The reach, all real input (this round — the old REACH teleport onto (−11.35, 1.12, −18.6) is
/// deleted): the game's route to the bridge is the player's own ring through the west offices to
/// the pocket — the SAME walked timeline as walk_office_floor_qworm_keyboard_hatch (the Q-Worm
/// PC detour included, green end-to-end), verbatim to the keypad's wall stand (−11.9, −17.5)
/// [D keypad #648 at (−10.69, 1.52, −17.46)], then south-east onto the bridge's west end. The
/// timeline must be bit-identical to the main walk's up to the divergence: civilian g8 patrols
/// the desk lane itself (his loop (−14.8, 8.3) ↔ (−18.7, 5.4), rng 16 fov 1.57 [L the
/// NF_ALARM_TRACE roster dump]) and a fresh-state crossing meets him head-on (d 0.5, seen →
/// challenge 0x3e → the alarm press → fail 0x4000032 [L this round's trace runs]) — the main
/// walk's phase (the PC detour delays the lane crossing) keeps him out of sight the whole way,
/// so the shared timeline is the honest walk. The strip's west edge x −11.43 [D] and the wall's
/// west face x −10.70 [D] stay east of the final leg; the closed door #724's leaf sits in the
/// gap (x −10.70..−10.45, z −18.04..−19.36 [D]) and is never crossed — the bridge's west half
/// is pocket-side floor, walled off from the corridor-mouth civilian g15 [D the coll-scan wall,
/// the line to (−11.35, −18.6) crosses x −10.70 inside the wall's z −15.38..−17.96 span].
#[test]
fn bond_stands_at_the_mouth_bridge_edge() {
    // The main walk's own prologue + ring, bit-identical (the objectives 3-5 chain included —
    // the timeline is the phase-setter for the desk lane's patroller).
    let mut w = walk_qworm_and_back();
    let ring: [([f32; 3], &str); 23] = [
        ([-7.5, 1.3, 14.5], "ring: the security centre's west door"),
        ([-7.5, 1.3, 16.5], "ring: the west hall north"),
        ([-20.5, 1.3, 16.5], "ring: the west offices' north corridor"),
        ([-20.5, 1.3, 5.5], "ring: down the west office block"),
        ([-17.6, 1.3, 5.5], "ring: the ai node 17 east of the door"),
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
        ([-14.5, 1.3, -15.0], "ring: south of the pillar, into the pocket"),
        ([-11.9, 1.3, -17.5], "ring: the keypad's wall"),
    ];
    for (wp, label) in ring {
        // The ring's west-office legs cross g8's patrol at his (−18.6, 5.4) pause and the door
        // jinks south of it; at the game's crouch pace the prologue lands Bond there with him
        // (bumped at d 0.1 → 0x3e → PressAlarm). Wait in the north corridor for his west-room
        // leg (the hatch walk's own window, wait_mouth_window).
        if label == "ring: down the west office block" { wait_mouth_window(&mut w); }
        watch_and_clear(&mut w, 16.0);
        w.walk_to(Wp::new(wp[0], wp[1], wp[2]).r(0.8), label);
        assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "the mission failed at '{label}'");
    }
    // The bridge's west end, walked: south-east from the keypad's wall across the pocket's
    // floor onto the strip (the target is the old teleport's own spot; the arrive r 0.3 keeps
    // the root east of the strip's west edge x −11.43 [D] without touching the wall's west
    // face x −10.70 [D] — the capsule's east edge at the root x −11.35 still clears it).
    watch_and_clear(&mut w, 16.0);
    w.walk_to(Wp::new(-11.35, 1.12, -18.6).r(0.3), "the bridge's west end");
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
        if label.starts_with("e:") {
            // The game's standing bodies (`NDrone2_Collision` 0x140e90, on within 2.0 of the
            // player [M 0x149ec8 / 0x14b190]): guard #8 walking his GoToGoal 0x63 route west
            // through this door met Bond in it and both stood there, 0.95 u apart, for the leg's
            // whole 6000-tick cap [L capsule-join red: #8 at (−16.7, 5.8) heading for the door
            // as the leg began, then (−19.2, 4.9) wedged]. A player lets the man through the
            // doorway first, from out of his way back up the office block.
            let near_door = |w: &Walker, r: f32| {
                let sim = w.scene.sim.as_ref().unwrap();
                sim.drones.list.iter().any(|d| d.present && d.enabled && !d.corpse && d.health > 0.0
                    && ((d.pos[0] - -19.5f32).powi(2) + (d.pos[2] - 4.6f32).powi(2)).sqrt() < r && (d.pos[1] - 1.1).abs() < 1.5)
            };
            if near_door(&w, 4.0) {
                w.walk_to(Wp::new(-20.5, 1.3, 9.5).r(0.8), "d2: back up the office block");
                for _ in 0..1500 {
                    if !near_door(&w, 3.0) { break; }
                    w.tick(Default::default(), false);
                }
                w.walk_to(Wp::new(-20.5, 1.3, 5.5).r(1.2), "d3: down the office block again");
            }
        }
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

    // The free walk: the game's own hall line, reached ON REAL INPUT (this round — the old
    // probe placement write to (−8.0, 1.12, 16.29) is deleted). The route is the game's own:
    // the through-line THROUGH the doorway into the west hall (the glide leg, green on master —
    // the jamb slide around the base pieces at z ≈ 16.278 [D port world tris] is part of the
    // through-line), then back east along the hall to the spot. The straight lobby approach
    // (lobby south → the spot) was tried this round and WEDGES where the game does not: its
    // diagonal runs z 16.16 at the column's x, south of the base course, and the raw forward
    // hold pins on the column's south-east corner (root −7.75, z slid 16.21 → 16.02, the
    // accumulator +0.1/tick — the capsule pressed on the column's east face x −8.304). The
    // game's route enters the hall through the doorway, not the south pocket.
    w.wait(30);
    for _ in 0..150 {
        let done = w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
            .filter(|s| s.placement == 524 || s.placement == 525)
            .all(|s| s.state == 0 || s.state == 0x40);
        if done { break; }
        w.wait(5);
    }
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    w.walk_to(Wp::new(-10.0, 0.4, 16.29).r(0.8), "the through-line into the west hall");
    w.walk_to(Wp::new(-8.0, 1.12, 16.29).r(0.3), "the game's hall line");
    // Face west (the [L] window's own yaw −1.5708) with real turn input.
    let mut turn_ticks = 0u32;
    loop {
        let err = walker::wrap_pi(-1.5708 - w.scene.sim.as_ref().unwrap().bond.yaw);
        if err.abs() < 0.02 { break; }
        assert!(turn_ticks < 600, "the west turn never settled (yaw {})",
            w.scene.sim.as_ref().unwrap().bond.yaw);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
        turn_ticks += 1;
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

/// Real damage: the same walk with the health top-up off for every walker it makes (the
/// tests' own `top_up_health = true` included) and the walker's return fire on — Bond's death
/// panics with the death spot and the last hits (common/damage_log.rs), so a pass pins that the
/// walk completes on the port's own damage.
///
/// The player's route (2026-11-01, level-0a-ceiling-gun-route): the west hall's laser #607
/// (out ch 0x1F, mask 8, x ≈ −8.45, y 1.49-1.76, z 15-20.25 [D triggers dump]) fed OR #667 →
/// ch 0x0F, ceiling gun #640's enable (param 4, at (−22.1, 2.3, 17.2) aimed straight down [D
/// 0700000a-audit]); the old standing route armed it on every #607 crossing and the two bursts
/// (27 + 15 rounds of 2.4 ≈ 101) outlast Bond's 100 health — the gun re-arms after each crossing
/// through the case-3 fold [M GT_Update 0x180498 case 3 → state 0], and only explosions damage a
/// turret (`GT_ApplyDamage` 0x1816d8's one caller is `Explode_Propagate` 0x17c518 case 0x36 [M]),
/// of which 0a's start loadout has none. The game's answer is the CROUCH: the crouched pick arm
/// tops out ≈ feet+1.26 [M Player_Collision 0x194df0 case 3: A = pos+0.1−(cc−0.55), B =
/// pos+0.45−cc, r 0.55; the crouch park pos−floor 0.607 [L level_reload_live.rs:215]] — under
/// the beam band 1.49-1.76 [D] — while the standing body 1.86 [M 0x194df0 default arm] crosses.
/// The walk now crouches every #607 crossing (the outbound hall leg, the walk-back's crossing
/// legs, the ring's west leg), the final laser pin asserts no mask-8 out and no alert-OR out
/// fired (gun #640 never armed), and the turret rounds themselves carry the game's Bullet_init
/// cone (spread 4.0 [L ee_start_05.bin @0x2c55d0+0x24], the 3+1 shared-stream draws [M
/// 0x12c5a0:166-184 + 0x180498:349]) — the dead-on port gap fixed this round.
#[test]
fn walk_office_floor_qworm_keyboard_hatch_real_damage() {
    walker::with_real_damage(walk_office_floor_qworm_keyboard_hatch);
}

/// The laser band rule, pinned phase by phase on fresh loads (each phase asserts right after its
/// crossing, before any guard reaction can land):
/// - **Crouched passes #607** — the crouched pick arm tops out ≈ feet+1.26 [M Player_Collision
///   0x194df0 case 3: A = pos+0.1−(cc−0.55), B = pos+0.45−cc, r 0.55; the crouch park
///   pos−floor 0.607 [L level_reload_live.rs:215]] vs beam #607's band y 1.49-1.76 (x ≈ −8.45,
///   z 15-20.25 [D triggers dump]): walking the west hall CROUCHED leaves ch 0x1F off (sister:
///   off before), so OR #667 never raises ch 0x0F and ceiling gun #640 stays dormant
///   [D the OR wiring; M Trigger_MultiplexOrIn 0x1b2d38].
/// - **Standing raises 0x1F** — the standing body 1.86 [M 0x194df0 default arm] crosses the band:
///   the same west-hall leg STANDING fires ch 0x1F.
/// - **The low beams fire on either stance** — beam #621 (out 0x24, y 0.13-0.39 [D]) walked
///   through standing and crouched fires ch 0x24 both times, live on fresh boots [L 2026-11-01];
///   the route jumps the low lines instead (jump_over_low_beam).
/// - **A jump clears a low line** — beam #614 walked through fires 0x22; jumped from 0.9-1.3
///   before the line it stays silent [L 2026-11-01].
#[test]
fn laser_band_rule_crouch_clears_607_and_low_beams_fire_either_way() {
    // Phase A — standing through #607's plane.
    let mut w = Walker::new(nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a"));
    w.wait(150); // the SS lift doors' create sweep (the route's wait)
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x1f), "sister: the beam is silent before the standing crossing");
    // The interactables laser test's exact through-crossing leg (the walk passes THROUGH the
    // plane — the toucher's own x crosses it, not just its radius reaching it).
    w.walk_to(Wp::new(-10.0, 0.4, 17.5).r(1.2), "through the laser wall STANDING");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x1f),
        "the standing body 1.86 [M 0x194df0 default arm] crosses beam #607's band 1.49-1.76 [D] → ch 0x1F");
    drop(w);
    // Phase B — the same plane crouched: the beam stays silent.
    let mut w = load_carry(None);
    w.wait(150);
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x1f), "sister: the beam is silent before the crouched crossing");
    set_crouch(&mut w, true);
    w.walk_to(Wp::new(-10.0, 0.4, 16.0).r(0.6), "the west hall crouched under #607");
    // Stay crouched and walk clear of the beam's standing reach (the toucher's 0.55 radius [M
    // 0x194df0 default arm] around the box reaches x −9.14 from the tri face −8.59 [D tris]):
    // the walker's 1.2-arrive path can swing east into that zone, and a standing step there
    // fires the beam.
    w.walk_to(Wp::new(-12.0, 0.4, 14.5).r(1.2), "south-west, clear of beam #607's reach (crouched)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = &sim.bond;
        eprintln!("[0a] crouch probe: pos {:?} cc {} crouched {} substate {} eye {} drop {}",
            b.pos, b.cc, b.crouched, b.substate, b.eye, b.crouch_drop);
    }
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x1f),
        "the crouched pick arm tops ≈ feet+1.26 [M 0x194df0 case 3], under the band 1.49-1.76 [D] → ch 0x1F off");
    // Phase C — the low beam fires crouched: the live crouched crossing of #621's line at
    // x −12.6 fired ch 0x24 (and 0x10) [L 2026-11-01, fresh boot: at z −32.60; the line
    // z ≈ −33.05 there] — a crouch never passes a low line (the standing half is
    // low_beam_621_fires_standing, a port gap).
    low_beam_621_crossing(true);
    // Phase D — a jump clears a low line: live on a fresh boot, jumping down the west corridor
    // with the press 0.9 (and 1.3) before beam #614's line (z 13.03) passed it silent —
    // airborne +0.43 over the walk height at the line — while the walk fired ch 0x22 + 0x0F
    // [L 2026-11-01; a press 0.5 before, still +0.23 at the line, fired]. The route's
    // jump_over_low_beam presses at 1.0. Sister (the comparison can fail): the same line walked
    // CROUCHED fires — the crouched pick arm reaches the floor (case 3's B = pos+0.45−cc, r 0.55
    // [M 0x194df0:228-248]), as the live crouched #621 crossing showed (phase C). The standing
    // walk's fire is the port gap in low_beam_621_fires_standing.
    for jump in [true, false] {
        let mut w = load_carry(None);
        w.wait(30);
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [-22.2, 1.03, 15.8];
            sim.bond.yaw = std::f32::consts::PI;
        }
        w.wait(30);
        assert_eq!(w.scene.sim.as_ref().unwrap().drones.channels.time[0x22], 0, "sister: beam #614 silent before the crossing");
        if jump {
            jump_over_low_beam(&mut w, BEAM_614, [-22.2, 0.4, 11.0], 1.0, "jump beam #614 (band rule)");
            assert_eq!(w.scene.sim.as_ref().unwrap().drones.channels.time[0x22], 0,
                "the jump clears beam #614 [L 2026-11-01: press 0.9 before the line, silent]");
        } else {
            set_crouch(&mut w, true);
            w.walk_to(Wp::new(-22.2, 0.4, 11.0).r(0.6), "walk through beam #614 crouched");
            assert_ne!(w.scene.sim.as_ref().unwrap().drones.channels.time[0x22], 0,
                "walking crouched through beam #614 fires ch 0x22");
        }
    }
}

/// Walk south through beam #621's line at x −12.6 from a fresh load, standing or crouched;
/// asserts ch 0x24 fired (sister: silent before).
fn low_beam_621_crossing(crouch: bool) {
    let mut w = Walker::new(nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a"));
    w.wait(30);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-12.6, 1.13, -31.2];
        sim.bond.yaw = std::f32::consts::PI;
    }
    w.wait(30);
    if crouch { set_crouch(&mut w, true); }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.channels.time[0x24], 0, "sister: beam #621 silent before the crossing (crouch {crouch})");
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.crouched, crouch);
    w.walk_to(Wp::new(-12.6, 0.4, -35.0).r(0.6), "south through beam #621's line");
    assert_ne!(w.scene.sim.as_ref().unwrap().drones.channels.time[0x24], 0,
        "the low beam #621 fires ch 0x24 on the crossing, crouch {crouch} [L 2026-11-01]");
}

/// The standing half of the low-beam rule: live, Bond walking STANDING through #621's line
/// fired ch 0x24 + 0x10 at z −32.77 (the line z ≈ −33.05 at x −12.6) [L 2026-11-01, fresh
/// boot]. The game's standing trigger toucher is the collide-pass HITTEST segment
/// (`Player_Collision` 0x194df0's default arm A = pos+0.275, B = pos+(0.55−cc), r 0.55,
/// mode 0x800 [M :315-333]; picked by `Collide_PickObj` 0x1eae80:96-119, refined by
/// `Intersect_CylGeom` vs the trigger's glist [M 0x1ebaa8:226-247 → 0x109da0:447], mirrored
/// into the trigger's +0xD0 [M 0x1ea398:160-215], credited class-3 by `Trigger_Hit` 0x1b1fc8
/// [L ee_start_0700000a.bin: the player obj+0xFF = 3]). trigger-hittest-segment completed the
/// chain: the trigger cels' glist flags bit 0x40 [D #607/#614/#621 = 0x645] routes the refine
/// to the RAW per-triangle law (the plane-crossing anchor + `Dist2Tri`'s one-sided entry +
/// the strict < radius refine — triggers.rs `seg_touches_tri_raw`), and trigger-plane-records
/// landed the missing half: the one-sided entry reads the glist's OWN plane records (the tri's
/// 4th u16 → 3×i16 normal · 2⁻¹⁴ + the f32 distance at byte +6 [M asm 0x10a1d8-0x10a1e8],
/// parsed per tri — `Collision::triangles_with_planes`), not the winding cross whose sign broke
/// the 05 spawn plate and the office-ring route on the e7f80cd6 wiring. The standing toucher is
/// `player.body` (the HITTEST segment) and this walk's #614 jump presses ride 1.8 (the
/// arc-height-vs-press-ticks law: the game's own arc clears the line at +0.43 [L], the port's
/// full-speed arc crossed it at the 1.0 press).
#[test]
fn low_beam_621_fires_standing() {
    low_beam_621_crossing(false);
}

/// Load-only probe: every mask-8 (player) touch trigger's tri bounding box — the laser web's
/// positions the route must crouch or dodge.
#[test]
fn probe_laser_web_positions() {
    let cold = nf_view::load_with_carry(&iso(), "0700000a.bin", false, None).expect("load");
    let sim = cold.sim.as_ref().unwrap();
    for t in &sim.drones.triggers {
        if t.mask & 8 == 0 || t.tris.is_empty() { continue; }
        if t.placement == 621 || t.placement == 614 {
            for (ti, tri) in t.tris.iter().enumerate() {
                eprintln!("[laser{}] tri{} {:?}", t.placement, ti, tri);
            }
        }
        let (mut lo, mut hi) = (t.tris[0][0], t.tris[0][0]);
        for tri in &t.tris { for v in tri { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
        eprintln!("[laser] plc {} lock {:#04x} out {:#04x} lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2})",
            t.placement, t.lock, t.out, lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
    }
}


/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty-08-09-0a-0b): the
/// same walked office-floor ring — Q-Worm, the security keyboard, the keypad door, the hatch —
/// at the menu's Agent pick — GameState+0x28 = 2 through the front-end's launch path
/// [M C_SBNFDFCTY 0x216ca8:15; the `difficulty[]` table 0x2df4c0 row 1 carries 2; Txt 0x14e =
/// "Agent"] on the warm carry (the elevator chain from 09). On this level group (09-0b) the pick
/// scales Bond's pain by Plr_DMod_Normal 0.8 instead of Easy 0.6 and the guards' hit chance by
/// DroneFiring_Accuracy_Normal 1.0 instead of Easy 0.8 [M Player_HandlePain 0x1902c8;
/// DroneWeap_DoBulletAccuracy 0x179740:74-88; tuning.rs 09-0b group ← ReadTuningVars 0x1d3a80];
/// the min-difficulty placements pass Drone_Create's gate [M 0x136be0:49]. Real input only — the
/// load path is the only difference.
#[test]
fn walk_office_floor_qworm_keyboard_hatch_real_damage_agent() {
    walker::with_real_damage(|| walk_office_floor_qworm_keyboard_hatch_at(2))
}

/// 00 Agent-difficulty sister of `..._real_damage` (real-health-00agent-08-09-0a-0b): the same
/// walked office-floor ring — Q-Worm, the security keyboard, the keypad door, the hatch — at the
/// menu's 00 Agent pick — GameState+0x28 = 3 through the front-end's launch path
/// [M C_SBNFDFCTY 0x216ca8:15; the `difficulty[]` table 0x2df4c0 row 2 carries 3 [D]; Txt 0x14f
/// = "00 Agent"] on the warm carry (the elevator chain from 09). On this level group (09-0b)
/// the pick scales Bond's pain by Plr_DMod_Hard 1.0 (Easy 0.6, Agent 0.8); the guards' hit
/// chance stays DroneFiring_Accuracy_Hard 1.0 — the SAME row Agent runs (Easy 0.8) — and the
/// guards' base bullet damage rides DroneDamage_Hard 0.8 (Agent 1.0) [M Player_HandlePain
/// 0x1902c8; DroneWeap_DoBulletAccuracy 0x179740:74-88; ReadTuningVars 0x1d3a80 cases
/// 0x7000009-a-b; tuning.rs 09-0b group]; the min-difficulty placements pass Drone_Create's
/// gate at d2 AND d3 [M 0x136be0:49]. Real input only — the load path is the only difference
/// (plus the d3-gated tactics the deaths force, inside this file).
#[test]
fn walk_office_floor_qworm_keyboard_hatch_real_damage_00agent() {
    walker::with_real_damage(|| walk_office_floor_qworm_keyboard_hatch_at(3))
}
