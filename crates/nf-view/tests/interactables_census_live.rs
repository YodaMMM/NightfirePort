//! Interactables census, per object: every on-foot level's switches, use-key ScriptPlayers, E-panel
//! triggers and doors (lock-channel doors included) operated one by one through the real use path
//! (`Sim::step_bond`'s use key = the fresh press of action 0xE [M Input_Action 0x17f208 →
//! Player_WeaponFiring 0x1a5460 → Player_Activate 0x1a5170]).
//!
//! use_pick_census_live covers the SimpleScripts, free doors, keypad locks, Q-Worm PCs and monitors
//! face by face; use_key_census_live operates one object per class per level. This binary takes the
//! classes neither walks object by object:
//!
//! - switches (`Switch_Activate` 0x1b02a8, reached through the switch's embedded ScriptPlayer
//!   entity and its facing dot [M SP_Activate 0x1dc338:44-61]): every switch whose gates pass
//!   (channel set and not 0x61, gate channel off, not a spent one-shot) must toggle its channel;
//! - use-key ScriptPlayers (`SP_Activate` 0x1dc338: mask & 4, the gate channel on, the out channel
//!   on when mask & 1): every one with a live entity must take the press (`activated` latch or a
//!   state move). A gate that is off at load is set ON for the trial: it stands in for the mission
//!   reaching that point, and the row says so;
//! - E-panel triggers (`Trigger_Activate` 0x1b2318 types 1/2: mask & 4, out set): every one must
//!   write its out channel;
//! - doors with a lock channel (`Door_Activate` 0x135be0: lock channel set and OFF refuses): the lock
//!   channel is set ON for the trial (the unlock stands in for the mission) and E must move the door.
//!   Free doors are walked by use_pick_census_live already and are not repeated here.
//!
//! Every stand is in front of one of the object's collision faces (common/usepick.rs: the game's pick
//! needs the use point inside the world sphere and within 1.0 in front of a face [M Collide_PickObj
//! 0x1eae80, Intersect_PointGeom 0x1e92d8]). Sister on every row: the effect is read OFF on the tick
//! before the press. An object with no stand anywhere is listed (not failed): the stand search is a
//! stand-in for a walk and can miss a nook, so those rows are read by hand in the report.

use nf_view::scene::Sim;

type V3 = [f32; 3];

#[path = "common/usepick.rs"]
mod usepick;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn len(a: V3) -> f32 { dot(a, a).sqrt() }

#[derive(Default)]
struct Tally {
    found: usize,
    tested: usize,
    broken: Vec<String>,
    nostand: Vec<String>,
    skipped: Vec<String>,
    forced: Vec<String>,
    notes: Vec<String>,
}

/// The state the trials touch, restored before every press: an E on one object (or the toggle it
/// writes) moves its neighbours too — a switch's channel drives its twins' mirrors [M Switch_Update
/// 0x1adea8], so one trial left the next switch on the channel looking like a spent one-shot.
struct Snap {
    ch: nf_game::triggers::Channels,
    sw: Vec<(u8, u8, u16)>,
    players: Vec<nf_game::nis::Player>,
    triggers: Vec<nf_game::triggers::Trigger>,
    doors: nf_game::door::Doors,
    mission: (u32, u32, u32),
    // Bond, the guards and the monitors: trials advance the clock, so a guard that sees Bond
    // shoots him, and a monitor view disables him — either would refuse every later press.
    bond: nf_game::player::Bond,
    health: nf_game::bullet::PlayerHealth,
    guards: Vec<nf_game::drone::Drone>,
    monitors: Vec<nf_game::monitor::Monitor>,
    // A keypad lock beside the target (07000014's armoury door) opens the keypad menu on the
    // press [M Lock_Activate 0x1aed00 mode 0], and an open menu refuses every later use.
    locks: Vec<nf_game::lock::Lock>,
}

impl Snap {
    fn take(sim: &Sim) -> Snap {
        Snap {
            ch: sim.drones.channels.clone(),
            sw: sim.drones.useobjs.switches.iter().map(|w| (w.value, w.latch, w.auto_ct)).collect(),
            players: sim.drones.scripts.players.clone(),
            triggers: sim.drones.triggers.clone(),
            doors: sim.drones.doors.clone(),
            mission: (sim.drones.mission.internal, sim.drones.mission.state, sim.drones.mission.fail_label),
            bond: sim.bond.clone(),
            health: sim.drones.health,
            guards: sim.drones.list.clone(),
            monitors: sim.drones.monitors.list.clone(),
            locks: sim.drones.locks.clone(),
        }
    }
    fn put(&self, sim: &mut Sim) {
        sim.drones.channels = self.ch.clone();
        for (w, &(v, l, a)) in sim.drones.useobjs.switches.iter_mut().zip(&self.sw) { (w.value, w.latch, w.auto_ct) = (v, l, a); }
        sim.drones.scripts.players = self.players.clone();
        sim.drones.triggers = self.triggers.clone();
        sim.drones.doors = self.doors.clone();
        (sim.drones.mission.internal, sim.drones.mission.state, sim.drones.mission.fail_label) = self.mission;
        sim.drones.level_exit = None;
        sim.bond = self.bond.clone();
        sim.drones.health = self.health;
        sim.drones.list = self.guards.clone();
        sim.drones.monitors.list = self.monitors.clone();
        sim.disabled = false;
        sim.drones.locks = self.locks.clone();
        sim.drones.keypad = None;
    }
}

/// Set channel `c` ON for a trial, together with the inputs of any multiplexor that writes it:
/// a mux re-derives its out every tick [M Trigger_Calc 0x1b1900], so a bare write to an AND's out
/// is undone the next tick (07000014's armoury door lock ch 0x2E = AND(0x2C, 0x2D), mux #777 [D]).
fn force_on(sim: &Sim, chs: &mut nf_game::triggers::Channels, c: u32) {
    chs.set(c, 1, 0);
    for t in &sim.drones.triggers {
        if t.out as u32 == c && t.kind != nf_game::triggers::kind::TOUCH_ONCE && t.kind != nf_game::triggers::kind::TOUCH {
            for &k in &t.list { if k != 0 { chs.set(k, 1, 0); } }
        }
    }
}

fn near_load_trigger(sim: &Sim, p: V3, r: f32) -> bool {
    sim.drones.triggers.iter().any(|t| t.kind == nf_game::triggers::kind::LOAD_LEVEL
        && !(0..3).any(|k| p[k] + r < t.lo[k] || p[k] - r > t.hi[k]))
}

/// The twelve triangles of an axis box (a trigger without a parsed glist keeps its box).
fn box_tris(lo: V3, hi: V3) -> Vec<[V3; 3]> {
    let c = |x: usize, y: usize, z: usize| [if x == 0 { lo[0] } else { hi[0] }, if y == 0 { lo[1] } else { hi[1] }, if z == 0 { lo[2] } else { hi[2] }];
    // Each face wound outward.
    let quads = [
        [c(0, 0, 0), c(0, 0, 1), c(0, 1, 1), c(0, 1, 0)], // -x
        [c(1, 0, 0), c(1, 1, 0), c(1, 1, 1), c(1, 0, 1)], // +x
        [c(0, 0, 0), c(1, 0, 0), c(1, 0, 1), c(0, 0, 1)], // -y
        [c(0, 1, 0), c(0, 1, 1), c(1, 1, 1), c(1, 1, 0)], // +y
        [c(0, 0, 0), c(0, 1, 0), c(1, 1, 0), c(1, 0, 0)], // -z
        [c(0, 0, 1), c(1, 0, 1), c(1, 1, 1), c(0, 1, 1)], // +z
    ];
    quads.iter().flat_map(|q| [[q[0], q[1], q[2]], [q[0], q[2], q[3]]]).collect()
}

fn bound(tris: &[[V3; 3]]) -> (V3, f32) {
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in tris.iter().flatten() { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } }
    let c = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
    (c, len(sub(hi, c)))
}

/// One object: walk every stand in front of its faces; at each, `arm` restores the trial state,
/// a still tick proves the effect off (sister), one E press and four still ticks read it.
/// Returns Some(true) used, Some(false) stands found but none took, None no stand.
fn operate(sim: &mut Sim, tris: &[[V3; 3]], sph: (V3, f32), allowed: &dyn Fn(&Sim) -> bool,
           arm: &dyn Fn(&mut Sim), hit: &dyn Fn(&Sim) -> bool, sister_fail: &mut bool) -> Option<bool> {
    let mut stands = 0;
    let mut used = false;
    usepick::for_each_stand(sim, tris, sph, &mut |sim: &mut Sim| {
        if near_load_trigger(sim, sim.bond.pos, 2.5) { return false; }
        if !allowed(sim) { return false; }
        let (pos, yaw, pitch) = (sim.bond.pos, sim.bond.yaw, sim.bond.pitch);
        arm(sim);
        let hold = |sim: &mut Sim| { sim.bond.pos = pos; sim.bond.vel = [0.0; 3]; sim.bond.yaw = yaw; sim.bond.pitch = pitch; };
        hold(sim);
        sim.step_bond(Default::default(), false, false, false);
        if hit(sim) { *sister_fail = true; return false; }
        hold(sim);
        sim.step_bond(Default::default(), false, false, true);
        if len(sub(sim.bond.pos, pos)) > 0.2 { return false; }
        stands += 1;
        let mut got = hit(sim);
        for _ in 0..4 {
            if got { break; }
            hold(sim);
            sim.step_bond(Default::default(), false, false, false);
            got = hit(sim);
        }
        sim.drones.level_exit = None;
        if got { used = true; }
        got
    });
    arm(sim);
    if used { Some(true) } else if stands > 0 { Some(false) } else { None }
}

/// NF_CENSUS_ONLY=<class> runs one section (switch, sp, trigger, door, monitor) — a probe aid.
fn section(name: &str) -> bool { std::env::var("NF_CENSUS_ONLY").map_or(true, |v| v == name) }

/// NF_CENSUS_PL=<placement> keeps one door (a probe aid).
fn only_pl(p: usize) -> bool {
    std::env::var("NF_CENSUS_PL").map_or(true, |v| v.split(',').any(|x| x.parse::<usize>().ok() == Some(p)))
}

fn census(archive: &str) -> Tally {
    let mut scene = nf_view::load_with_carry_weapons_difficulty(&iso(), archive, false, None, &[], 3)
        .unwrap_or_else(|e| panic!("load {archive}: {e}"));
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    let home = sim.bond.pos;
    let ch0 = sim.drones.channels.clone();
    let base = Snap::take(sim);
    let mut t = Tally::default();
    let record = |t: &mut Tally, name: String, r: Option<bool>, sister: bool| {
        t.tested += 1;
        if sister { t.broken.push(format!("{name}: effect on before the press (sister)")); return; }
        match r {
            Some(true) => {}
            Some(false) => t.broken.push(format!("{name}: stands in front of its faces, E has no effect")),
            None => t.nostand.push(name),
        }
    };

    // --- Switches through their embedded SP entity [M Switch_Create 0x1adcc0:44-66, SP_Activate 0x1dc338:44-61].
    let switch_sps: Vec<usize> = sim.drones.useobjs.switches.iter().filter_map(|s| s.sp).collect();
    for si in 0..sim.drones.useobjs.switches.len() {
        if !section("switch") { break; }
        let s = &sim.drones.useobjs.switches[si];
        let (ch, gate, placement, busy, sp) = (s.ch, s.gate, s.placement, s.busy, s.sp);
        let (value0, latch0) = (s.value, s.latch);
        let name = format!("{archive} switch #{placement} ch {ch:#x}");
        t.found += 1;
        if ch == 0 { t.skipped.push(format!("{name}: no channel, never activates [M 0x1b02a8:24]")); continue; }
        if ch == 0x61 { t.skipped.push(format!("{name}: alarm channel, refuses the player [M 0x1b02a8:20-21]")); continue; }
        if busy && latch0 != 0 { t.skipped.push(format!("{name}: spent one-shot (latch {latch0}, value {value0}, on {})", ch0.on(ch as u32))); continue; }
        if gate != 0 && ch0.on(gate as u32) { t.skipped.push(format!("{name}: gate {gate:#x} on at load")); continue; }
        let Some(spi) = sp else { t.skipped.push(format!("{name}: no embedded SP")); continue };
        let picks = { let d = sim.data(); sim.drones.scripts.players[spi].entity_use_picks(&sim.drones.scripts.objects, &d) };
        if picks.is_empty() { t.nostand.push(format!("{name}: no live entity")); t.tested += 1; continue; }
        let on0 = ch0.on(ch as u32);
        let mut res = None;
        let mut sister = false;
        for (c, r, dir, epos, h, m) in picks {
            let tris = usepick::posed(sim, h, &m);
            if tris.is_empty() { continue; }
            let got = operate(sim, &tris, (c, r),
                &|s: &Sim| dot(dir, sub(s.bond.pos, epos)) >= 0.0,
                &|s: &mut Sim| base.put(s),
                &|s: &Sim| s.drones.channels.on(ch as u32) != on0, &mut sister);
            if got.is_some() { res = got; }
            if res == Some(true) { break; }
        }
        record(&mut t, name, res, sister);
    }

    // --- Use-key ScriptPlayers [M SP_Activate 0x1dc338].
    for spi in 0..sim.drones.scripts.players.len() {
        if !section("sp") { break; }
        if switch_sps.contains(&spi) { continue; }
        let p = &sim.drones.scripts.players[spi];
        if p.deleted || p.mask & 4 == 0 { continue; }
        t.found += 1;
        let (gate, out, mask, placement) = (p.gate, p.out, p.mask, p.placement);
        let name = format!("{archive} SP #{placement} mask {mask:#x} gate {gate:#x} out {out:#x} state {} delay {:?}", p.state, p.info[p.cur].as_ref().map(|s| s.delay));
        let mut chs = ch0.clone();
        if gate != 0 && !chs.on(gate as u32) { force_on(sim, &mut chs, gate as u32); t.forced.push(format!("{name}: gate set on")); }
        if mask & 1 != 0 && out != 0 && !chs.on(out as u32) { force_on(sim, &mut chs, out as u32); t.forced.push(format!("{name}: out set on (mask 1)")); }
        let picks = { let d = sim.data(); sim.drones.scripts.players[spi].entity_use_picks(&sim.drones.scripts.objects, &d) };
        if picks.is_empty() { t.skipped.push(format!("{name}: no live entity at load (not E-usable [M 0x1a5170:121-122])")); continue; }
        let sp0 = sim.drones.scripts.players[spi].clone();
        let (st0, cur0) = (sp0.state, sp0.cur);
        // A held script (`SCRIPTINFO+0x9C != 0`) resumes on the press: `SP_Activate` zeroes it
        // [M 0x1dc338:84-86] even where `SP_Update` folds the latch the same tick.
        let delay0 = sp0.info[sp0.cur].as_ref().map(|s| s.delay);
        let mut res = None;
        let mut sister = false;
        for (c, r, _dir, _epos, h, m) in picks {
            let tris = usepick::posed(sim, h, &m);
            if tris.is_empty() { continue; }
            let got = operate(sim, &tris, (c, r), &|_| true,
                &|s: &mut Sim| { base.put(s); s.drones.channels = chs.clone(); },
                &|s: &Sim| { let x = &s.drones.scripts.players[spi]; x.activated || x.state != st0 || x.cur != cur0
                    || (delay0.is_some_and(|d| d != 0) && x.info[x.cur].as_ref().map(|s| s.delay) == Some(0)) }, &mut sister);
            if got.is_some() { res = got; }
            if res == Some(true) { break; }
        }
        record(&mut t, name, res, sister);
    }
    base.put(sim);

    // --- E-panel triggers [M Trigger_Activate 0x1b2318:23-49].
    for ti in 0..sim.drones.triggers.len() {
        if !section("trigger") { break; }
        let tr = &sim.drones.triggers[ti];
        if tr.mask & 4 == 0 || tr.out == 0 || !matches!(tr.kind, nf_game::triggers::kind::TOUCH_ONCE | nf_game::triggers::kind::TOUCH) { continue; }
        t.found += 1;
        let (out, placement, lo, hi) = (tr.out, tr.placement, tr.lo, tr.hi);
        let name = format!("{archive} E-panel trigger #{placement} out {out:#x}");
        if tr.gone { t.skipped.push(format!("{name}: gone at load")); continue; }
        if ch0.on(out as u32) { t.skipped.push(format!("{name}: out already on at load")); continue; }
        let tris = if tr.tris.is_empty() { box_tris(lo, hi) } else { tr.tris.clone() };
        let sph = bound(&tris);
        let mut sister = false;
        let res = operate(sim, &tris, sph, &|_| true,
            &|s: &mut Sim| base.put(s),
            &|s: &Sim| s.drones.channels.on(out as u32), &mut sister);
        // A walk-in plate the stand itself fires is the walk's business, not the E's.
        if sister { t.skipped.push(format!("{name}: fires on the walk-in at the stand")); continue; }
        record(&mut t, name, res, false);
    }
    base.put(sim);

    // --- Doors [M Door_Activate 0x135be0]: free ones open on E; lock-channel ones refuse E while
    // the channel is off and open once it is on.
    for i in 0..sim.drones.doors.list.len() {
        if !section("door") { break; }
        let d0 = sim.drones.doors.list[i].clone();
        if !only_pl(d0.placement) { continue; }
        t.found += 1;
        let name = format!("{archive} door #{} lock ch {:#x}", d0.placement, d0.ch_lock);
        if d0.glist == 0 || d0.keys.is_none() { t.skipped.push(format!("{name}: not an E door (no keys/glist)")); continue; }
        if d0.locked_flag != 0 { t.skipped.push(format!("{name}: DOORINFO+0x77 locked byte")); continue; }
        if d0.state & 0xc != 0 { t.skipped.push(format!("{name}: open at load")); continue; }
        let tris = usepick::posed(sim, d0.glist, &d0.matrix);
        if tris.is_empty() { t.skipped.push(format!("{name}: no collision shape")); continue; }
        let mut chs = ch0.clone();
        let sph = d0.live_sph();
        let keep = d0.clone();
        let moved = |s: &Sim| { let x = &s.drones.doors.list[i]; x.state != keep.state || (x.pos01 - keep.pos01).abs() > 1e-4 };
        if d0.ch_lock != 0 && !chs.on(d0.ch_lock as u32) {
            // The refusal: lock channel set and OFF on a closed door → the hit sound only [M 0x135be0].
            // A move after the lock channel rose in the trial is an unlock by a plate at the door
            // (07000003's E plate → ch 0x31, 07000011's storeroom touch plate #283 → ch 0x5A,
            // 07000016's security-station plates [D]) — the game's own route, not a lock bypass.
            // The plate's write can be gone by the end of the tick (a touch plate's channel is
            // cleared by the next Trigger_Calc), so the test reads the channel's write stamp: every
            // nonzero write stamps `switch_channels_time` [M Trigger_Calc / Switch_Update writers].
            let lch = d0.ch_lock as usize;
            let t0 = ch0.time.get(lch).copied();
            let mut sister = false;
            let r = operate(sim, &tris, sph, &|_| true, &|s: &mut Sim| base.put(s),
                &|s: &Sim| moved(s) && !s.drones.channels.on(lch as u32) && s.drones.channels.time.get(lch).copied() == t0, &mut sister);
            t.tested += 1;
            if r == Some(true) || sister { t.broken.push(format!("{name}: moves while its lock channel is off")); }
            force_on(sim, &mut chs, d0.ch_lock as u32);
            t.forced.push(format!("{name}: lock channel set on"));
        }
        let mut sister = false;
        let chs2 = chs.clone();
        let res = operate(sim, &tris, sph,
            // `Door_Activate`'s side test on a direct activation with Bond's head as the activator.
            &|s: &Sim| {
                let mut dd = s.drones.doors.clone();
                dd.list[i] = keep.clone();
                let mut ch = chs2.clone();
                dd.activate(i, &[(s.bond.head(), 0.0, false)], true, &mut ch, s.drones.frame, &mut Vec::new())
            },
            &|s: &mut Sim| { base.put(s); s.drones.channels = chs.clone(); },
            &moved, &mut sister);
        // Moving with no press once unlocked: `Door_Update` activates the door on the lock
        // channel's rising edge [M 0x1363c8:101-107], so the unlock itself opens it — a pass.
        if sister && d0.ch_lock != 0 && res != Some(true) {
            t.tested += 1;
            t.notes.push(format!("{name}: opens by itself on the lock channel's rising edge [M Door_Update 0x1363c8:101-107]"));
            continue;
        }
        // A free door that moves before the press was opened by someone else in the trial (a
        // guard walking through [M Door_Update's proximity scan], the lift-shaft garrison on
        // 0700004a): the door works, the E row is not separable there.
        if sister && res != Some(true) && d0.ch_lock == 0 {
            t.tested += 1;
            t.notes.push(format!("{name}: opened by another actor before the press"));
            continue;
        }
        record(&mut t, name, res, sister);
    }
    base.put(sim);

    // --- Monitors with a camera to show [M Monitor_Activate 0x1b0058].
    for mi in 0..sim.drones.monitors.list.len() {
        if !section("monitor") { break; }
        let pi = sim.drones.monitors.list[mi].placement;
        let name = format!("{archive} monitor #{pi}");
        t.found += 1;
        let mut probe = sim.drones.monitors.clone();
        if !probe.activate(mi, &sim.drones.sensors) { t.skipped.push(format!("{name}: no camera to show")); continue; }
        let Some(pick) = sim.drones.use_picks.monitors.get(mi).copied() else { continue };
        let tris = usepick::posed(sim, pick.hash, &pick.matrix);
        if tris.is_empty() { t.skipped.push(format!("{name}: no collision shape")); continue; }
        let keep = sim.drones.monitors.list[mi].clone();
        let mut sister = false;
        let res = operate(sim, &tris, pick.sphere, &|_| true,
            &|s: &mut Sim| { base.put(s); s.drones.monitors.list[mi] = keep.clone(); s.disabled = false; },
            &|s: &Sim| s.drones.monitors.list[mi].state != keep.state, &mut sister);
        record(&mut t, name, res, sister);
        sim.drones.monitors.list[mi] = keep;
        sim.disabled = false;
    }
    base.put(sim);
    sim.bond.pos = home;

    eprintln!("[census] {archive}: found {}, tested {}, broken {}, no stand {}, skipped {}",
        t.found, t.tested, t.broken.len(), t.nostand.len(), t.skipped.len());
    for b in &t.broken { eprintln!("  BROKEN {b}"); }
    for b in &t.nostand { eprintln!("  NOSTAND {b}"); }
    for b in &t.skipped { eprintln!("  skip {b}"); }
    for b in &t.forced { eprintln!("  forced {b}"); }
    for b in &t.notes { eprintln!("  pass {b}"); }
    t
}

fn check(archive: &str) {
    let t = census(archive);
    assert!(t.broken.is_empty(), "{archive}: {} objects refuse the real E:\n{}", t.broken.len(), t.broken.join("\n"));
}

macro_rules! level_rows { ($($name:ident = $file:expr;)*) => { $( #[test] fn $name() { check($file); } )* } }

level_rows! {
    census_07000001 = "07000001.bin";
    census_07000002 = "07000002.bin";
    census_07000003 = "07000003.bin";
    census_07000004 = "07000004.bin";
    census_07000005 = "07000005.bin";
    census_07000006 = "07000006.bin";
    census_07000007 = "07000007.bin";
    census_07000008 = "07000008.bin";
    census_07000009 = "07000009.bin";
    census_0700000a = "0700000a.bin";
    census_0700000b = "0700000b.bin";
    census_0700000c = "0700000c.bin";
    census_0700000d = "0700000d.bin";
    census_07000011 = "07000011.bin";
    census_07000012 = "07000012.bin";
    census_07000013 = "07000013.bin";
    census_07000014 = "07000014.bin";
    census_07000015 = "07000015.bin";
    census_07000016 = "07000016.bin";
    census_0700001b = "0700001b.bin";
    census_0700004a = "0700004a.bin";
}
