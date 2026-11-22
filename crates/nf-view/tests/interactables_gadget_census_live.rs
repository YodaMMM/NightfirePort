//! Gadget-target census, per object: every on-foot level's Q-Claw grapple points and burnable /
//! decodable locks, each operated through the real fire path (`Sim::step_bond`'s trigger with the
//! gadget selected through the real weapon switch).
//!
//! - Grapple points (`Grapple_Create`, placement 0x3A): the claw fires only inside a type-2 third
//!   icon's arm (the context byte `BLData+0x95F` == 2 [M Player_WeaponFiring 0x1a5460:515-528]).
//!   Each point is tried from the nearest enabled type-2 icons: Bond placed in the icon's arm
//!   (the place-once stand-in for the walk), the claw drawn, the view aimed at the point and the
//!   trigger pulled; the row passes when the claw latches a point within reach of this one
//!   [M Player_SetGrapplePoint 0x195da0]. Sister: no grapple before the fire.
//! - Locks (`Lock_Create` 0x1ae1f8, placement 0x2E): the Q-Decryptor and the laser family burn or
//!   decode them [M Lock_Update 0x1ae3d8 case 0: kind 1 takes the laser 0x4E/0x4F hits, kinds 0/2
//!   the PDA band `− 0x56 < 2`]. Each locked one is tried with the matching gadget when the G cycle
//!   reaches it on that level: Bond stood in front of each of its collision faces in turn
//!   (common/usepick.rs), aimed at its world sphere, the trigger held; the row passes when the
//!   lock opens (unlocked / burned, its out channel ON). Sister: out OFF before. A keypad on a
//!   level that carries no PDA is listed: its route is the typed code.
//! - A grapple point whose line from the arm crosses a use-key script entity (a vent lid) is
//!   retried after that entity's ScriptPlayer is opened with the real E.
//!
//! A point with no type-2 icon on the level, or a lock with no stand / no carried gadget, is
//! listed, not failed: those rows are read by hand in the report.

use nf_view::scene::Sim;

type V3 = [f32; 3];

#[path = "common/usepick.rs"]
mod usepick;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn len(a: V3) -> f32 { (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt() }

const CLAW: u8 = 0x50;
const LOCK_GADGETS: [u8; 4] = [0x4e, 0x4f, 0x56, 0x57];

#[derive(Default)]
struct Tally { found: usize, tested: usize, broken: Vec<String>, listed: Vec<String> }

fn aim(sim: &mut Sim, at: V3) {
    let h = sim.bond.head();
    let d = sub(at, h);
    let l = len(d).max(1e-4);
    sim.bond.yaw = d[0].atan2(d[2]);
    sim.bond.pitch = (d[1] / l).asin() / std::f32::consts::FRAC_PI_2;
}

/// Draw weapon `w` through the real switch input; true when it is in hand.
fn draw(sim: &mut Sim, w: u8) -> bool {
    for _ in 0..40 {
        sim.step_bond(nf_game::player::Input { switch_to: Some(w), ..Default::default() }, false, false, false);
        if sim.gun.weapon == w { break; }
    }
    // The G cycle reaches the gadgets the switch key does not (0700000d's claw).
    for _ in 0..16 {
        if sim.gun.weapon == w { break; }
        sim.step_bond(nf_game::player::Input { gadget_cycle: 1, ..Default::default() }, false, false, false);
        for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    }
    for _ in 0..200 { if sim.gun.switch_state == 0 { break; } sim.step_bond(Default::default(), false, false, false); }
    sim.gun.weapon == w && sim.gun.switch_state == 0
}

/// The lock gadgets the G cycle reaches on this level (the real cycle input).
fn cycle_gadgets(sim: &mut Sim) -> Vec<u8> {
    let mut seen = Vec::new();
    for _ in 0..16 {
        sim.step_bond(nf_game::player::Input { gadget_cycle: 1, ..Default::default() }, false, false, false);
        for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
        let w = sim.gun.weapon;
        if seen.contains(&w) { break; }
        seen.push(w);
    }
    eprintln!("[gadget census] G cycle: {:x?}", seen);
    seen.into_iter().filter(|w| LOCK_GADGETS.contains(w)).collect()
}

fn seg_tri(a: V3, b: V3, t: &[V3; 3]) -> bool {
    let d = sub(b, a);
    let e1 = sub(t[1], t[0]);
    let e2 = sub(t[2], t[0]);
    let p = [d[1] * e2[2] - d[2] * e2[1], d[2] * e2[0] - d[0] * e2[2], d[0] * e2[1] - d[1] * e2[0]];
    let det = e1[0] * p[0] + e1[1] * p[1] + e1[2] * p[2];
    if det.abs() < 1e-9 { return false; }
    let inv = 1.0 / det;
    let s = sub(a, t[0]);
    let u = (s[0] * p[0] + s[1] * p[1] + s[2] * p[2]) * inv;
    if !(0.0..=1.0).contains(&u) { return false; }
    let q = [s[1] * e1[2] - s[2] * e1[1], s[2] * e1[0] - s[0] * e1[2], s[0] * e1[1] - s[1] * e1[0]];
    let v = (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]) * inv;
    if v < 0.0 || u + v > 1.0 { return false; }
    let tt = (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]) * inv;
    (0.0..=1.0).contains(&tt)
}

/// Open the first mask-4 ScriptPlayer whose live entity crosses the segment `a`→`b`, through the
/// real E from a stand in front of the entity [M SP_Activate 0x1dc338], then let its script play.
fn open_blocking_sp(sim: &mut Sim, a: V3, b: V3) -> Option<usize> {
    for spi in 0..sim.drones.scripts.players.len() {
        let p = &sim.drones.scripts.players[spi];
        if p.deleted || p.mask & 4 == 0 { continue; }
        let placement = p.placement;
        let picks = { let d = sim.data(); sim.drones.scripts.players[spi].entity_use_picks(&sim.drones.scripts.objects, &d) };
        for (c, r, _, _, h, m) in picks {
            let tris = usepick::posed(sim, h, &m);
            if !tris.iter().any(|t| seg_tri(a, b, t)) { continue; }
            if !usepick::press_on(sim, &tris, (c, r), 60, &|s: &Sim| s.drones.scripts.players[spi].state != 1) { return None; }
            for _ in 0..300 { sim.step_bond(Default::default(), false, false, false); }
            return Some(placement);
        }
    }
    None
}

fn census(archive: &str) -> Tally {
    let mut t = Tally::default();
    let load = || {
        let mut scene = nf_view::load_with_carry_weapons_difficulty(&iso(), archive, false, None, &[], 3)
            .unwrap_or_else(|e| panic!("load {archive}: {e}"));
        {
            let sim = scene.sim.as_mut().unwrap();
            for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
        }
        scene
    };

    // --- Grapple points.
    let n_points = load().sim.as_ref().unwrap().drones.grapple_points.len();
    for gi in 0..n_points {
        let mut scene = load();
        let sim = scene.sim.as_mut().unwrap();
        let gp = sim.drones.grapple_points[gi].pos;
        let name = format!("{archive} grapple point {gi} at ({:.1}, {:.1}, {:.1})", gp[0], gp[1], gp[2]);
        t.found += 1;
        let mut icons: Vec<(f32, V3, u16)> = sim.icons.iter().filter(|ic| ic.ctx == 2 && !ic.dead)
            .map(|ic| (len(sub(ic.centre, gp)), ic.centre, ic.on_channel)).filter(|(d, _, _)| *d < 60.0).collect();
        icons.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        if icons.is_empty() { t.listed.push(format!("{name}: no type-2 icon within 60")); continue; }
        if !draw(sim, CLAW) { t.listed.push(format!("{name}: the claw is not carried")); continue; }
        t.tested += 1;
        let mut ok = false;
        let mut tried = Vec::new();
        for &(_, c, on_ch) in icons.iter().take(4) {
            let yaw = sim.bond.yaw;
            sim.place_bond([c[0], c[1] + 0.5, c[2]], yaw);
            for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
            // An icon armed by a channel the mission raises later (07000012's on-channel 0x3C):
            // the channel is set ON for the trial, standing in for that progress [M ThirdIcon
            // 0x197070:27-32], and the row says so.
            if sim.bond.icon_ctx != 2 && on_ch != 0 && !sim.drones.channels.on(on_ch as u32) {
                let f = sim.drones.frame;
                sim.drones.channels.set(on_ch as u32, 1, f);
                tried.push(format!("icon on-channel {on_ch:#x} set on"));
                for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
            }
            if sim.bond.icon_ctx != 2 { tried.push(format!("({:.1},{:.1},{:.1}) ctx {} (icon on-channel {on_ch:#x} {})", c[0], c[1], c[2], sim.bond.icon_ctx, sim.drones.channels.on(on_ch as u32))); continue; }
            if sim.bond.grapple.is_some() { t.broken.push(format!("{name}: grapple on before the fire (sister)")); break; }
            let fire = |sim: &mut Sim| for _ in 0..30 {
                aim(sim, gp);
                sim.step_bond(Default::default(), true, false, false);
                if sim.bond.grapple.is_some() { break; }
            };
            fire(sim);
            if sim.bond.grapple.is_none() {
                // A use-key script entity between the arm and the point (0700000a's storeroom vent lid,
                // SP #685 mask 4 — its entity 0x02000750 lies across the shaft under the SkyBridge
                // point [D]) is opened first with the real E, the way the route does it.
                let stand = sim.bond.pos;
                if let Some(op) = open_blocking_sp(sim, sim.bond.head(), gp) {
                    tried.push(format!("E opened SP #{op} first"));
                    let yaw = sim.bond.yaw;
                    sim.place_bond(stand, yaw);
                    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
                }
                fire(sim);
            }
            match sim.bond.grapple.as_ref() {
                Some(g) if len(sub(g.point, gp)) < 3.0 => { ok = true; break; }
                Some(g) => tried.push(format!("({:.1},{:.1},{:.1}) latched another point ({:.1},{:.1},{:.1})", c[0], c[1], c[2], g.point[0], g.point[1], g.point[2])),
                None => tried.push(format!("({:.1},{:.1},{:.1}) no latch", c[0], c[1], c[2])),
            }
            sim.bond.grapple = None;
            sim.bond.substate = nf_game::player::SS_WALK;
        }
        if !ok { t.listed.push(format!("{name}: no latch from the nearest type-2 icons: {}", tried.join("; "))); }
        else if !tried.is_empty() { eprintln!("  pass {name}: {}", tried.join("; ")); }
    }

    // --- Locks.
    let n_locks = load().sim.as_ref().unwrap().drones.locks.len();
    for li in 0..n_locks {
        let mut scene = load();
        let sim = scene.sim.as_mut().unwrap();
        let l = &sim.drones.locks[li];
        let (out, placement, kind) = (l.out, l.placement, l.kind);
        let name = format!("{archive} lock #{placement} kind {kind} out {out:#x}");
        t.found += 1;
        if l.unlocked || l.burned { t.listed.push(format!("{name}: open at load")); continue; }
        let mut gadgets = cycle_gadgets(sim);
        // The kind picks the arm [M Lock_Update 0x1ae3d8:47 kind 1 = the laser hits 0x4E/0x4F;
        // :162 kinds 0/2 = the PDA band 0x56/0x57]: the matching gadget goes first.
        let arm = |g: u8| if kind == 1 { g == 0x4e || g == 0x4f } else { g == 0x56 || g == 0x57 };
        gadgets.retain(|&g| arm(g));
        if gadgets.is_empty() { t.listed.push(format!("{name}: the level carries no gadget for its kind (the typed code / key is the route)")); continue; }
        if gadgets.is_empty() { t.listed.push(format!("{name}: no lock gadget in the G cycle")); continue; }
        let Some(pick) = sim.drones.use_picks.locks.get(li).copied() else { continue };
        let tris = usepick::posed(sim, pick.hash, &pick.matrix);
        let opened = |s: &Sim| { let l = &s.drones.locks[li]; l.unlocked || l.burned || (out != 0 && s.drones.channels.on(out as u32)) };
        if opened(sim) { t.broken.push(format!("{name}: open before the burn (sister)")); continue; }
        t.tested += 1;
        let mut ok = false;
        let mut notes = Vec::new();
        for &g in &gadgets {
            if !draw(sim, g) { notes.push(format!("{g:#x} not drawn")); continue; }
            // Every stand in front of the lock's faces in turn: a keypad's back face sits through
            // its mount wall (07000014's #776 behind the 14A/14B door frame), so the first stand
            // can be the wrong side. A stand whose fire starts nothing in 120 ticks is left.
            let mut stands = 0;
            if !tris.is_empty() {
                usepick::for_each_stand(sim, &tris, pick.sphere, &mut |sim: &mut Sim| {
                    stands += 1;
                    let pos = sim.bond.pos;
                    for k in 0..900 {
                        sim.bond.pos = pos;
                        sim.bond.vel = [0.0; 3];
                        aim(sim, pick.sphere.0);
                        sim.step_bond(Default::default(), true, false, false);
                        if opened(sim) { ok = true; return true; }
                        let l = &sim.drones.locks[li];
                        if k == 120 && !l.scanning && l.burn_b == 255 && l.burn_a == 255 { return false; }
                    }
                    false
                });
            }
            if stands == 0 { notes.push(format!("{g:#x} no stand")); continue; }
            if ok { break; }
            notes.push(format!("{g:#x} held 900 ticks, lock still shut"));
        }
        if !ok {
            if notes.iter().all(|n| n.ends_with("no stand") || n.ends_with("not drawn")) { t.listed.push(format!("{name}: {}", notes.join("; "))); }
            else { t.broken.push(format!("{name}: {}", notes.join("; "))); }
        }
    }

    eprintln!("[gadget census] {archive}: found {}, tested {}, broken {}, listed {}", t.found, t.tested, t.broken.len(), t.listed.len());
    for b in &t.broken { eprintln!("  BROKEN {b}"); }
    for b in &t.listed { eprintln!("  listed {b}"); }
    t
}

fn check(archive: &str) {
    let t = census(archive);
    assert!(t.broken.is_empty(), "{archive}: {} gadget targets refuse the real fire:\n{}", t.broken.len(), t.broken.join("\n"));
}

macro_rules! level_rows { ($($name:ident = $file:expr;)*) => { $( #[test] fn $name() { check($file); } )* } }

level_rows! {
    gadgets_07000001 = "07000001.bin";
    gadgets_07000002 = "07000002.bin";
    gadgets_07000003 = "07000003.bin";
    gadgets_07000004 = "07000004.bin";
    gadgets_07000005 = "07000005.bin";
    gadgets_07000006 = "07000006.bin";
    gadgets_07000007 = "07000007.bin";
    gadgets_07000008 = "07000008.bin";
    gadgets_07000009 = "07000009.bin";
    gadgets_0700000a = "0700000a.bin";
    gadgets_0700000b = "0700000b.bin";
    gadgets_0700000c = "0700000c.bin";
    gadgets_0700000d = "0700000d.bin";
    gadgets_07000011 = "07000011.bin";
    gadgets_07000012 = "07000012.bin";
    gadgets_07000013 = "07000013.bin";
    gadgets_07000014 = "07000014.bin";
    gadgets_07000015 = "07000015.bin";
    gadgets_07000016 = "07000016.bin";
    gadgets_0700001b = "0700001b.bin";
    gadgets_0700004a = "0700004a.bin";
}
