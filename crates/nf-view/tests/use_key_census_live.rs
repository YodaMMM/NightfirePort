//! use-key census: every story level, one placement of each use-key class present, operated
//! through the real path (`nf_view::load` → `Sim::step_bond`'s use key = the fresh press of
//! action 0xE [M `Input_Action` 0x17f208 bit 0x4 → `Player_WeaponFiring` 0x1a5460 →
//! `Player_Activate` 0x1a5170]). The target picks come from the live sim lists (the parsed
//! placements [D]); the per-level expectations come from docs/levels/*.md and are tabulated in
//! docs/systems/use-key.md §4.
//!
//! Sisters: every operated target records its effect flag OFF immediately before the use press,
//! so deleting the press (or the operation) fails the test. Negative rows pin the game's
//! refusals: a lock without the key must not unlock [M Lock_Activate 0x1aed00], a switch on
//! channel 0x61 must not answer a class-3 activator [M Switch_Activate 0x1b02a8], an SS without
//! mask bit 4 must ignore E [M SS_Activate 0x1a9ea8], and levels with no E-usable placements
//! assert exactly that.
//!
//! The teleport beside each target is the documented place-once stand-in (walkreal tests); the
//! operation itself runs through the real aim + use input. Difficulty 3 so Operative (1) never
//! auto-unlocks a lock under us [M Lock_Create 0x1ae1f8:67-69].

use nf_view::scene::{Scene, Sim};

type V3 = [f32; 3];

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn at_origin(p: V3) -> bool { p[0].abs() < 1e-3 && p[1].abs() < 1e-3 && p[2].abs() < 1e-3 }

fn load(level: &str) -> Scene {
    nf_view::load_with_carry_weapons_difficulty(&iso(), level, false, None, &[], 3)
        .unwrap_or_else(|e| panic!("load {level}: {e}"))
}

/// Place Bond once beside the target (the documented stand-in) and let him ground. Candidate
/// offsets settle in order; the landing nearest the target (x z, then y) wins — a door leaf's
/// centre sits ~2 above its floor and some placements hang over a drop, so one blind offset
/// lands Bond on the wrong storey.
fn place_near(sim: &mut Sim, pos: V3) -> V3 {
    // The first offset is the historic one (the walkreal stand-in) — kept when it lands close;
    // the rest are fallbacks for placements hanging over a drop.
    let offs: [V3; 5] = [[0.0, 1.2, 1.4], [0.0, -0.5, 1.0], [0.0, -0.5, -1.0], [1.0, -0.5, 0.0], [-1.0, -0.5, 0.0]];
    let mut best = sim.bond.pos;
    let mut best_score = f32::MAX;
    for o in offs {
        sim.bond.pos = [pos[0] + o[0], pos[1] + o[1], pos[2] + o[2]];
        for _ in 0..100 { sim.step_bond(Default::default(), false, false, false); }
        let score = ((sim.bond.pos[0] - pos[0]).powi(2) + (sim.bond.pos[2] - pos[2]).powi(2)).sqrt()
            + (sim.bond.pos[1] - pos[1]).abs();
        if score < best_score { best_score = score; best = sim.bond.pos; }
        let dxz = ((sim.bond.pos[0] - pos[0]).powi(2) + (sim.bond.pos[2] - pos[2]).powi(2)).sqrt();
        // Reach-aware early accept: the tightest pick reach in the use paths is 2.2 (the SS /
        // monitor arm) from the HEAD — the settled head is ~0.7 above the feet.
        let hd = ((sim.bond.pos[0] - pos[0]).powi(2) + (sim.bond.pos[1] + 0.7 - pos[1]).powi(2)
            + (sim.bond.pos[2] - pos[2]).powi(2)).sqrt();
        if o == offs[0] && dxz <= 3.0 && hd <= 2.0 && sim.drones.level_exit.is_none() { return sim.bond.pos; }
    }
    sim.bond.pos = best;
    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    sim.bond.pos
}

/// Aim the view at the target and pulse E one tick in six (the fresh-press edge) until `done`
/// says the effect landed. Returns whether it did. The operation is the real input only.
fn aim_use(sim: &mut Sim, target: V3, done: &dyn Fn(&Sim) -> bool) -> bool {
    for k in 0..360 {
        if sim.drones.level_exit.is_some() { return false; } // the exit-hold freezes the world
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        let (dx, dz, dy) = (target[0] - eye[0], target[2] - eye[2], target[1] - eye[1]);
        let err = dx.atan2(dz) - sim.bond.yaw;
        let want_pitch = (dy / (dx * dx + dz * dz).sqrt().max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let input = nf_game::player::Input {
            turn: err.clamp(-0.08, 0.08),
            look: (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        };
        sim.step_bond(input, false, false, k % 6 == 0);
        if done(sim) { return true; }
    }
    false
}

/// Just pulse E while looking the way the caller left the view (for negative rows).
fn press_use(sim: &mut Sim, ticks: usize) {
    for k in 0..ticks { sim.step_bond(Default::default(), false, false, k % 6 == 0); }
}

/// Is the point within `r` of a LOAD_LEVEL trigger's box? Those fire with no mask gate
/// [M Trigger_Activate 0x1b2318 — type 10 has no mask&4 gate] and freeze the level in the
/// exit hold — operation rows must stand away from them.
fn near_load_trigger(sim: &Sim, p: V3, r: f32) -> bool {
    sim.drones.triggers.iter().any(|t| t.kind == nf_game::triggers::kind::LOAD_LEVEL
        && !(0..3).any(|k| p[k] + r < t.lo[k] || p[k] - r > t.hi[k]))
}

/// Operate the first candidate door that answers (the census needs ONE door of the level; a
/// candidate whose floor is elsewhere — Bond lands > 3 away in x z — or that stands next to a
/// LOAD_LEVEL trigger volume is skipped).
fn use_any_door(sim: &mut Sim, cands: &[usize]) -> Option<usize> {
    // Operate from each offset as it is tried (a best-score pick can land inside the wall
    // reveal beside a door whose own floor is on one side only).
    let offs: [V3; 4] = [[0.0, -0.5, 1.0], [0.0, -0.5, -1.0], [1.0, -0.5, 0.0], [-1.0, -0.5, 0.0]];
    for &di in cands {
        let dpos = sim.drones.doors.list[di].pos;
        if near_load_trigger(sim, dpos, 0.0) { continue; }
        for o in offs {
            sim.bond.pos = [dpos[0] + o[0], dpos[1] + o[1], dpos[2] + o[2]];
            for _ in 0..100 { sim.step_bond(Default::default(), false, false, false); }
            if sim.drones.level_exit.is_some() { return None; }
            let dxz = ((sim.bond.pos[0] - dpos[0]).powi(2) + (sim.bond.pos[2] - dpos[2]).powi(2)).sqrt();
            if dxz > 2.0 { continue; }
            let (st0, p0) = (sim.drones.doors.list[di].state, sim.drones.doors.list[di].pos01);
            let target = [dpos[0], dpos[1] - 0.5, dpos[2]];
            if aim_use(sim, target, &|s| {
                let d = &s.drones.doors.list[di];
                d.state != st0 || (d.pos01 - p0).abs() > 0.01
            }) { return Some(di); }
            if sim.drones.level_exit.is_some() { return None; }
        }
    }
    None
}

fn door_candidates(sim: &Sim) -> Vec<usize> {
    // Nearest Bond's load position first — the start floor's doors are the reachable ones.
    let mut cands: Vec<usize> = (0..sim.drones.doors.list.len())
        .filter(|&i| sim.drones.doors.list[i].ch_lock == 0 && sim.drones.doors.list[i].locked_flag == 0
            && sim.drones.doors.list[i].keys.is_some() && !at_origin(sim.drones.doors.list[i].pos))
        .collect();
    let b = sim.bond.pos;
    // Exit-volume-adjacent doors last: an E near a LOAD_LEVEL trigger fires the exit (no mask
    // gate [M 0x1b2318]) and the exit hold freezes the world for every later row.
    cands.sort_by_key(|&i| (near_load_trigger(sim, sim.drones.doors.list[i].pos, 2.0) as u32,
        ((sim.drones.doors.list[i].pos[0] - b[0]).powi(2)
            + (sim.drones.doors.list[i].pos[2] - b[2]).powi(2)) as u64));
    cands
}

fn trigger_centre(lo: V3, hi: V3) -> V3 {
    [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5]
}

// --- 07000001: monitor #105 (group 1, camera sensor #280) + doors ×5 [D 01.md:61-67].
#[test]
fn use_census_07000001() {
    let mut scene = load("07000001.bin");
    let sim = scene.sim.as_mut().unwrap();
    let mi = sim.drones.monitors.list.iter().position(|m| m.state == 0 && !at_origin(m.pos))
        .expect("07000001 carries a usable monitor [D 01.md]");
    let mpos = sim.drones.monitors.list[mi].pos;
    assert!(sim.drones.monitors.list[mi].state == 0, "sister: monitor starts idle");
    place_near(sim, mpos);
    assert!(aim_use(sim, mpos, &|s| s.drones.monitors.list[mi].state == 2),
        "E on the monitor must enter the camera view [M Monitor_Activate 0x1b0058]");
    assert!(sim.drones.monitors.list[mi].watched.is_some(), "the monitor must watch a camera");

    let cands = door_candidates(sim);
    assert!(!cands.is_empty(), "07000001 carries player doors [D 01.md]");
    assert!(use_any_door(sim, &cands).is_some(),
        "E on a door must move it [M Door_Activate 0x135be0]");
}

// --- 07000002: monitor #323 (param 0 = 2) + cameras #406/#423 [D 02.md:39,55].
#[test]
fn use_census_07000002() {
    let mut scene = load("07000002.bin");
    let sim = scene.sim.as_mut().unwrap();
    let mi = sim.drones.monitors.list.iter().position(|m| m.state == 0 && !at_origin(m.pos))
        .expect("07000002 carries a usable monitor [D 02.md]");
    let mpos = sim.drones.monitors.list[mi].pos;
    assert!(sim.drones.monitors.list[mi].state == 0, "sister: monitor starts idle");
    place_near(sim, mpos);
    assert!(aim_use(sim, mpos, &|s| s.drones.monitors.list[mi].state == 2),
        "E on the monitor must enter the camera view [M Monitor_Activate 0x1b0058]");
}

// --- 07000003: no E-usable placement at all (ch 0x96 is script-owned) [D 03.md].
#[test]
fn use_census_07000003_absence() {
    let mut scene = load("07000003.bin");
    let sim = scene.sim.as_mut().unwrap();
    assert!(!sim.drones.useobjs.switches.iter().any(|s| s.ch != 0 && !at_origin(s.pos)),
        "07000003 carries no player switch [D 03.md]");
    assert!(!sim.drones.useobjs.pcs.iter().any(|p| p.e_usable != 0 && p.ch != 0 && !at_origin(p.pos)),
        "07000003 carries no E-usable PC [D 03.md]");
    assert!(!sim.drones.monitors.list.iter().any(|m| !at_origin(m.pos)),
        "07000003 carries no monitor [D 03.md]");
}

// --- 07000004: none — mux #46 only, the switch/lock template data is dead at origin [D 04.md:61-68].
#[test]
fn use_census_07000004_absence() {
    let mut scene = load("07000004.bin");
    let sim = scene.sim.as_mut().unwrap();
    assert!(!sim.drones.useobjs.switches.iter().any(|s| s.ch != 0 && !at_origin(s.pos)),
        "07000004 carries no live switch [D 04.md]");
    assert!(!sim.drones.useobjs.pcs.iter().any(|p| p.e_usable != 0 && p.ch != 0 && !at_origin(p.pos)),
        "07000004 carries no E-usable PC [D 04.md]");
}

// --- 07000005: zero switches / locks / monitors / PCQWorm [D 05.md:108-110].
#[test]
fn use_census_07000005_absence() {
    let mut scene = load("07000005.bin");
    let sim = scene.sim.as_mut().unwrap();
    assert!(!sim.drones.useobjs.switches.iter().any(|s| s.ch != 0 && !at_origin(s.pos)),
        "07000005 carries no player switch [D 05.md]");
    assert!(!sim.drones.useobjs.pcs.iter().any(|p| p.e_usable != 0 && p.ch != 0 && !at_origin(p.pos)),
        "07000005 carries no E-usable PC [D 05.md]");
    assert!(!sim.drones.monitors.list.iter().any(|m| !at_origin(m.pos)),
        "07000005 carries no monitor [D 05.md]");
}

// --- 07000006: SimpleScripts ×28 but every one carries activate mask 0x1/0x2 — touch/bullet
// driven, none E-usable (mask bit 4) [D the live parse]; no switches/PCs/monitors.
#[test]
fn use_census_07000006_absence() {
    let mut scene = load("07000006.bin");
    let sim = scene.sim.as_mut().unwrap();
    assert!(!sim.drones.sss.list.iter().any(|s| s.mask & 4 != 0 && s.channel != 0),
        "07000006 carries no E-usable SS [D 06.md — all masks 0x1/0x2]");
    assert!(!sim.drones.useobjs.switches.iter().any(|s| s.ch != 0 && !at_origin(s.pos)),
        "07000006 carries no player switch [D 06.md]");
    assert!(!sim.drones.useobjs.pcs.iter().any(|p| p.e_usable != 0 && p.ch != 0),
        "07000006 carries no E-usable PC [D 06.md]");
}

// --- 07000007: no explicit switch/lock; doors ×15 [D 07.md:123-127].
#[test]
fn use_census_07000007() {
    let mut scene = load("07000007.bin");
    let sim = scene.sim.as_mut().unwrap();
    let cands = door_candidates(sim);
    assert!(!cands.is_empty(), "07000007 carries player doors [D 07.md]");
    assert!(use_any_door(sim, &cands).is_some(),
        "E on a door must move it [M Door_Activate 0x135be0]");
}

// --- 07000008: the safe — locks #207/#208 (out 134/135), SS #205 safe door (ch 136) [D 08.md:130-142].
#[test]
fn use_census_07000008() {
    let mut scene = load("07000008.bin");
    let sim = scene.sim.as_mut().unwrap();
    // Negative row: E without the key must NOT unlock the safe lock [M Lock_Activate 0x1aed00].
    let li = sim.drones.locks.iter().position(|l| !l.unlocked && !at_origin(l.pos))
        .expect("07000008 carries the safe locks [D 08.md]");
    let (out, lpos) = (sim.drones.locks[li].out, sim.drones.locks[li].pos);
    assert!(!sim.drones.channels.on(out as u32), "sister: the safe out channel starts off");
    place_near(sim, lpos);
    press_use(sim, 60);
    assert!(!sim.drones.locks[li].unlocked,
        "E without the key must not unlock the safe [M 0x1aed00 — no key, keypad menu instead]");
    assert!(!sim.drones.channels.on(out as u32),
        "the safe out channel must stay off after a bare E");
}

// --- 07000009: alarm switches ×11 (0x7C/0x7D, busy one-shots that never fired — latch 0);
// the elevator SP is elevator_buttons_09_live [D 09.md].
#[test]
fn use_census_07000009() {
    let mut scene = load("07000009.bin");
    let sim = scene.sim.as_mut().unwrap();
    let si = sim.drones.useobjs.switches.iter().position(|s| s.ch != 0 && s.ch != 0x61
            && !at_origin(s.pos) && !sim.drones.channels.on(s.ch as u32) && (s.latch == 0 || !s.busy))
        .expect("07000009 carries the alarm switches [D 09.md]");
    let (ch, spos) = (sim.drones.useobjs.switches[si].ch, sim.drones.useobjs.switches[si].pos);
    assert!(!sim.drones.channels.on(ch as u32), "sister: the switch channel starts off");
    place_near(sim, spos);
    assert!(aim_use(sim, spos, &|s| s.drones.channels.on(ch as u32)),
        "E on the switch must toggle its channel [M Switch_Activate 0x1b02a8]");
}

// --- 0700000a: PC #688 is armed=true but e_usable=0 — the gadget/bullet target, NOT an E target
// [D the live parse; PCQWorm_Activate 0x1b0840 needs param 2 != 0]; switches ch-0x61 are
// E-rejected [D 0a.md:99-110].
#[test]
fn use_census_0700000a() {
    let mut scene = load("0700000a.bin");
    let sim = scene.sim.as_mut().unwrap();
    let pi = sim.drones.useobjs.pcs.iter().position(|p| p.ch == 0x48)
        .expect("0700000a carries PC #688 (ch 0x48) [D 0a.md]");
    assert!(sim.drones.useobjs.pcs[pi].e_usable == 0,
        "PC #688 must not be E-usable (param 2 = 0) [D 0a.md]");

    // Negative row: a switch on channel 0x61 refuses a class-3 activator [M 0x1b02a8].
    let xi = sim.drones.useobjs.switches.iter().position(|s| s.ch == 0x61 && !at_origin(s.pos))
        .expect("0700000a carries the ch-0x61 switches [D 0a.md]");
    let xpos = sim.drones.useobjs.switches[xi].pos;
    let before = sim.drones.channels.on(0x61);
    place_near(sim, xpos);
    press_use(sim, 60);
    assert_eq!(sim.drones.channels.on(0x61), before,
        "a ch-0x61 switch must not answer the player [M 0x1b02a8]");
}

// --- 0700000b: switch #249 (ch 9), PC #297 (0x4C), lock #178 (out 47) [D 0b.md:161-172].
#[test]
fn use_census_0700000b() {
    let mut scene = load("0700000b.bin");
    let sim = scene.sim.as_mut().unwrap();
    let li = sim.drones.locks.iter().position(|l| !l.unlocked && !at_origin(l.pos))
        .expect("0700000b carries lock #178 [D 0b.md]");
    let (out, lpos) = (sim.drones.locks[li].out, sim.drones.locks[li].pos);
    place_near(sim, lpos);
    press_use(sim, 60);
    assert!(!sim.drones.locks[li].unlocked && !sim.drones.channels.on(out as u32),
        "E without the key must not unlock [M 0x1aed00]");

    if let Some(pi) = sim.drones.useobjs.pcs.iter().position(|p| p.e_usable != 0 && p.ch != 0 && !at_origin(p.pos)) {
        let (ch, ppos) = (sim.drones.useobjs.pcs[pi].ch, sim.drones.useobjs.pcs[pi].pos);
        assert!(!sim.drones.channels.on(ch as u32), "sister: the PC channel starts off");
        place_near(sim, ppos);
        assert!(aim_use(sim, ppos, &|s| s.drones.channels.on(ch as u32)),
            "E on the PC must raise its channel [M PCQWorm_Update 0x1aee20:34-37 — the channel write; 0x1b0840 is the Activate gate]");
    }

    if let Some(si) = sim.drones.useobjs.switches.iter().position(|s| s.ch != 0 && s.ch != 0x61
            && !at_origin(s.pos) && !sim.drones.channels.on(s.ch as u32) && (s.latch == 0 || !s.busy)) {
        let (ch, spos) = (sim.drones.useobjs.switches[si].ch, sim.drones.useobjs.switches[si].pos);
        place_near(sim, spos);
        assert!(aim_use(sim, spos, &|s| s.drones.channels.on(ch as u32)),
            "E on the switch must toggle its channel [M 0x1b02a8]");
    }
}

// --- 0700000c: SS ×8 (vents ch 18, #544 secret door ch 8), locks ×5 [D 0c.md:77-103].
#[test]
fn use_census_0700000c() {
    let mut scene = load("0700000c.bin");
    let sim = scene.sim.as_mut().unwrap();
    if let Some(si) = sim.drones.sss.list.iter().position(|s| s.mask & 4 != 0 && s.channel != 0
            && s.state != 0 && !sim.drones.channels.on(s.channel as u32)) {
        let (ch, spos) = (sim.drones.sss.list[si].channel, sim.drones.sss.list[si].pos());
        place_near(sim, spos);
        assert!(aim_use(sim, spos, &|s| s.drones.channels.on(ch as u32)),
            "E on the mask-4 SS must raise its drive channel [M 0x1a9ea8]");
    }
    let li = sim.drones.locks.iter().position(|l| !l.unlocked && !at_origin(l.pos))
        .expect("0700000c carries the kind-0x2E locks [D 0c.md]");
    let (out, lpos) = (sim.drones.locks[li].out, sim.drones.locks[li].pos);
    place_near(sim, lpos);
    press_use(sim, 60);
    assert!(!sim.drones.locks[li].unlocked && !sim.drones.channels.on(out as u32),
        "E without the key must not unlock [M 0x1aed00]");
}

// --- 0700000d: doors ×6 (2 E-swing); SS #112 is mask 1 — E must leave the SS alone; the
// E-panel trigger (TouchOnce out 43, mask 4) DOES answer E [D 0d.md:89-106 + the live parse].
#[test]
fn use_census_0700000d() {
    let mut scene = load("0700000d.bin");
    let sim = scene.sim.as_mut().unwrap();
    let cands = door_candidates(sim);
    assert!(!cands.is_empty(), "0700000d carries E-swing doors [D 0d.md]");
    assert!(use_any_door(sim, &cands).is_some(),
        "E on a door must move it [M 0x135be0]");

    // (No SS-negative row here: 0d's mask-1 SS #112 shares channel 0x2b with the E-panel
    // trigger — pressing E near it drives the channel through the trigger and the SS's own
    // playhead advances as the channel's CONSUMER [M SS_Update 0x1a9948], so the refusal is
    // not observable in isolation on this level. 07000013 carries the class rule row.)

    // E-panel trigger (mask 4, out != 0, type TOUCH_ONCE/TOUCH) writes its out channel: only
    // trigger types 1 and 2 write `+0x20` [M 0x1b2318:23-49]; MOVIE (0xd) pushes game states 6/7
    // instead (:51-62) and AND/SEQ_AND/OUT/OR do nothing there (daily audit 2026-10-19).
    let ti = sim.drones.triggers.iter().position(|t| t.mask & 4 != 0 && t.out != 0
            && matches!(t.kind, nf_game::triggers::kind::TOUCH_ONCE | nf_game::triggers::kind::TOUCH)
            && !t.gone && t.script.is_none()
            && !near_load_trigger(sim, trigger_centre(t.lo, t.hi), 2.5));
    if let Some(ti) = ti {
        let (out, tpos) = (sim.drones.triggers[ti].out,
            trigger_centre(sim.drones.triggers[ti].lo, sim.drones.triggers[ti].hi));
        assert!(!sim.drones.channels.on(out as u32), "sister: panel out {out} starts off");
        place_near(sim, tpos);
        assert!(aim_use(sim, tpos, &|s| s.drones.channels.on(out as u32)),
            "E on the panel trigger must write its out channel [M 0x1b2318]");
    }
}

// --- 07000011: PC #216 "Computer to hack" (out 0x38), SS vents (mask 4), lock #177 [D 11.md:136-158].
#[test]
fn use_census_07000011() {
    let mut scene = load("07000011.bin");
    let sim = scene.sim.as_mut().unwrap();
    let pi = sim.drones.useobjs.pcs.iter().position(|p| p.e_usable != 0 && p.ch != 0 && !at_origin(p.pos))
        .expect("07000011 carries the hackable PC [D 11.md]");
    let (ch, ppos) = (sim.drones.useobjs.pcs[pi].ch, sim.drones.useobjs.pcs[pi].pos);
    assert!(!sim.drones.channels.on(ch as u32), "sister: the PC channel starts off");
    place_near(sim, ppos);
    assert!(aim_use(sim, ppos, &|s| s.drones.channels.on(ch as u32)),
        "E on the PC must raise its channel [M PCQWorm_Update 0x1aee20:34-37 — the channel write; 0x1b0840 is the Activate gate]");

    if let Some(li) = sim.drones.locks.iter().position(|l| !l.unlocked && !at_origin(l.pos)) {
        let (out, lpos) = (sim.drones.locks[li].out, sim.drones.locks[li].pos);
        place_near(sim, lpos);
        press_use(sim, 60);
        assert!(!sim.drones.locks[li].unlocked && !sim.drones.channels.on(out as u32),
            "E without the key must not unlock [M 0x1aed00]");
    }
}

// --- 07000012: PC (out 0x34), monitor (grp 10, 5 sensors), locks ×2 — decryptor pair; the
// ch-0x58/0x59 E-switches #962/#965 sit on another map and the sim attaches use-objects from
// the main map only [S] unverified (the port's Drones::attach_level / UseObjects::create main-map scoping; the game's per-map attach is not pinned here) — the
// census asserts the sim truth (no main-map switch) and the gap is in the report [D 12.md:133].
#[test]
fn use_census_07000012() {
    let mut scene = load("07000012.bin");
    let sim = scene.sim.as_mut().unwrap();
    assert!(!sim.drones.useobjs.switches.iter().any(|s| s.ch == 0x58 || s.ch == 0x59),
        "the ch-0x58/0x59 switches are not main-map placements [D 12.md:133 — multi-map gap]");

    let pi = sim.drones.useobjs.pcs.iter().position(|p| p.e_usable != 0 && p.ch != 0 && !at_origin(p.pos))
        .expect("07000012 carries the E-usable PC [D 12.md]");
    let (pch, ppos) = (sim.drones.useobjs.pcs[pi].ch, sim.drones.useobjs.pcs[pi].pos);
    assert!(!sim.drones.channels.on(pch as u32), "sister: the PC channel starts off");
    place_near(sim, ppos);
    assert!(aim_use(sim, ppos, &|s| s.drones.channels.on(pch as u32)),
        "E on the PC must raise its channel [M PCQWorm_Update 0x1aee20:34-37 — the channel write; 0x1b0840 is the Activate gate]");

    if let Some(mi) = sim.drones.monitors.list.iter().position(|m| m.state == 0 && !at_origin(m.pos)) {
        let mpos = sim.drones.monitors.list[mi].pos;
        place_near(sim, mpos);
        assert!(aim_use(sim, mpos, &|s| s.drones.monitors.list[mi].state == 2),
            "E on the monitor must enter the camera view [M 0x1b0058]");
    }

    let li = sim.drones.locks.iter().position(|l| !l.unlocked && !at_origin(l.pos))
        .expect("07000012 carries the decryptor locks [D 12.md]");
    let (out, lpos) = (sim.drones.locks[li].out, sim.drones.locks[li].pos);
    place_near(sim, lpos);
    press_use(sim, 60);
    assert!(!sim.drones.locks[li].unlocked && !sim.drones.channels.on(out as u32),
        "E without the key must not unlock [M 0x1aed00]");
}

// --- 07000013: SS #149 Shutters2 is mask 1 — E no-op; nothing else E-usable [D 13.md:84-87].
#[test]
fn use_census_07000013() {
    let mut scene = load("07000013.bin");
    let sim = scene.sim.as_mut().unwrap();
    assert!(!sim.drones.useobjs.switches.iter().any(|s| s.ch != 0 && !at_origin(s.pos)),
        "07000013 carries no player switch [D 13.md]");
    let xi = sim.drones.sss.list.iter().position(|s| s.mask & 4 == 0 && s.channel != 0)
        .expect("07000013 carries the mask-1 SS [D 13.md]");
    let (st0, t0) = (sim.drones.sss.list[xi].state, sim.drones.sss.list[xi].t);
    let xpos = sim.drones.sss.list[xi].pos();
    if !at_origin(xpos) { place_near(sim, xpos); }
    press_use(sim, 60);
    assert!(sim.drones.sss.list[xi].state == st0 && sim.drones.sss.list[xi].t == t0,
        "a mask-without-4 SS must ignore E [M 0x1a9ea8]");
}

// --- 07000014: switches ×4 (#605→0xDE, #632→0x4F — busy one-shots, latch 0), doors ×55 [D 14.md:50-64].
#[test]
fn use_census_07000014() {
    let mut scene = load("07000014.bin");
    let sim = scene.sim.as_mut().unwrap();
    let si = sim.drones.useobjs.switches.iter().position(|s| s.ch != 0 && s.ch != 0x61
            && !at_origin(s.pos) && !sim.drones.channels.on(s.ch as u32) && (s.latch == 0 || !s.busy))
        .expect("07000014 carries the switches [D 14.md]");
    let (ch, spos) = (sim.drones.useobjs.switches[si].ch, sim.drones.useobjs.switches[si].pos);
    assert!(!sim.drones.channels.on(ch as u32), "sister: the switch channel starts off");
    place_near(sim, spos);
    assert!(aim_use(sim, spos, &|s| s.drones.channels.on(ch as u32)),
        "E on the switch must toggle its channel [M 0x1b02a8]");

    let cands = door_candidates(sim);
    if !cands.is_empty() {
        assert!(use_any_door(sim, &cands).is_some(),
            "E on a door must move it [M 0x135be0]");
    }
}

// --- 07000015: KeypadLocked ×3 (#788→0x95, #765→0xf7, #711→0xf9) [D 15.md:89-95].
#[test]
fn use_census_07000015() {
    let mut scene = load("07000015.bin");
    let sim = scene.sim.as_mut().unwrap();
    let n = sim.drones.locks.iter().filter(|l| !l.unlocked && !at_origin(l.pos)).count();
    assert!(n >= 3, "07000015 carries three keypad locks, got {n} [D 15.md]");
    // Every one of them: E without the key/code must not unlock [M 0x1aed00].
    for li in 0..sim.drones.locks.len() {
        if sim.drones.locks[li].unlocked || at_origin(sim.drones.locks[li].pos) { continue; }
        let (out, lpos) = (sim.drones.locks[li].out, sim.drones.locks[li].pos);
        assert!(!sim.drones.channels.on(out as u32), "sister: lock {li} out starts off");
        place_near(sim, lpos);
        press_use(sim, 60);
        assert!(!sim.drones.locks[li].unlocked,
            "E without the key must not unlock lock {li} [M 0x1aed00]");
        assert!(!sim.drones.channels.on(out as u32), "lock {li} out must stay off");
    }
}

// --- 07000016: switch #693 KikoDeathButton (0x90, gate 0x8d), E-panel triggers (mask 4) [D 16.md:119-131].
#[test]
fn use_census_07000016() {
    let mut scene = load("07000016.bin");
    let sim = scene.sim.as_mut().unwrap();
    let si = sim.drones.useobjs.switches.iter().position(|s| s.ch != 0 && s.ch != 0x61 && !at_origin(s.pos))
        .expect("07000016 carries the KikoDeathButton [D 16.md]");
    let (ch, gate, spos) = (sim.drones.useobjs.switches[si].ch, sim.drones.useobjs.switches[si].gate,
        sim.drones.useobjs.switches[si].pos);
    let before = sim.drones.channels.on(ch as u32);
    place_near(sim, spos);
    press_use(sim, 30);
    let after_press = sim.drones.channels.on(ch as u32);
    if gate != 0 && sim.drones.channels.on(gate as u32) {
        assert_eq!(after_press, before,
            "the gate {gate:#x} is on — the switch must be inhibited [M 0x1b02a8]");
    } else {
        // Gate off: finish the activation with the aimed loop (the first pulses may still be aiming).
        let (sbusy, slatch, svalue) = {
            let s = &sim.drones.useobjs.switches[si];
            (s.busy, s.latch, s.value)
        };
        let b = sim.bond.pos;
        let dxz = ((b[0] - spos[0]).powi(2) + (b[2] - spos[2]).powi(2)).sqrt();
        assert!(aim_use(sim, spos, &|s| s.drones.channels.on(ch as u32)) || after_press != before,
            "E on the switch must toggle its channel [M 0x1b02a8] — ch {ch:#04x} gate {gate:#04x} busy {sbusy} latch {slatch} value {svalue} bond_dxz {dxz:.2}");
    }

    // E-panel trigger (mask 4, out != 0, type TOUCH_ONCE/TOUCH) writes its out channel: only
    // trigger types 1 and 2 write `+0x20` [M 0x1b2318:23-49]; MOVIE (0xd) pushes game states 6/7
    // instead (:51-62) and AND/SEQ_AND/OUT/OR do nothing there (daily audit 2026-10-19).
    let ti = sim.drones.triggers.iter().position(|t| t.mask & 4 != 0 && t.out != 0
            && matches!(t.kind, nf_game::triggers::kind::TOUCH_ONCE | nf_game::triggers::kind::TOUCH)
            && !t.gone && t.script.is_none()
            && !near_load_trigger(sim, trigger_centre(t.lo, t.hi), 2.5));
    if let Some(ti) = ti {
        let (out, tpos) = (sim.drones.triggers[ti].out,
            trigger_centre(sim.drones.triggers[ti].lo, sim.drones.triggers[ti].hi));
        assert!(!sim.drones.channels.on(out as u32), "sister: panel out {out} starts off");
        place_near(sim, tpos);
        assert!(aim_use(sim, tpos, &|s| s.drones.channels.on(out as u32)),
            "E on the panel trigger must write its out channel [M 0x1b2318]");
    }
}

// --- 0700001b: ScriptPlayers ×78 (the Drake chain), no switches / PCs / monitors [D 1b.md:51,82].
#[test]
fn use_census_0700001b_absence() {
    let mut scene = load("0700001b.bin");
    let sim = scene.sim.as_mut().unwrap();
    assert!(!sim.drones.useobjs.switches.iter().any(|s| s.ch != 0 && !at_origin(s.pos)),
        "0700001b carries no player switch [D 1b.md]");
    assert!(!sim.drones.useobjs.pcs.iter().any(|p| p.e_usable != 0 && p.ch != 0 && !at_origin(p.pos)),
        "0700001b carries no E-usable PC [D 1b.md]");
    assert!(!sim.drones.monitors.list.iter().any(|m| !at_origin(m.pos)),
        "0700001b carries no monitor [D 1b.md]");
}
