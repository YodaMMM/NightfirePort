//! interactables-0d: one live test per KIND of player-touchable placement on 0700000d (the power
//! station, part 2), driven the way a player drives it (the walk, the use key, the trigger pull,
//! the walk-over), each asserting the game's effect with its sister. The census with the type
//! table lives in docs/levels/0700000d.md §7; the end-to-end route is
//! walkreal_0700000d_live.rs.
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] the EE snapshot. A teleport
//! to reach a spot is used only where the walk there is a separately documented blocker (the
//! crane sequence's 0xE2/SP ride gap — walkreal_0700000d_live's pinned stand-in); the operation
//! itself always runs through the real input.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The warm load the real run hands this level: record 11's channels ride the save [M 0x182270:92-95].
/// `bits` = the `bonus[0]` word in play (0 = the fresh-boot state; the kind-6 gate reads it
/// [M 0x18dda0:93-95]).
fn load_warm_bits(bits: u64) -> Walker {
    let cold = nf_view::load_with_carry_medals(&iso(), "0700000d.bin", false, None, 1, bits).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry_medals(&iso(), "0700000d.bin", false, Some(&save), 1, bits).expect("load warm"));
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    w
}

fn load_warm() -> Walker { load_warm_bits(0) }

/// The yard stand-in (walkreal's documented teleport past the unported crane ride), then settle.
fn to_yard(w: &mut Walker) {
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [140.0, 5.5, -276.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
}

/// Fire the held weapon at a static point through the real trigger path (the walkreal's aim loop).
fn shoot_at(w: &mut Walker, target: [f32; 3], range: f32, max_ticks: u32, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> bool {
    let start = w.total_ticks;
    let mut last_tap = 0u32;
    loop {
        if done(&w.scene.sim.as_ref().unwrap()) { return true; }
        if w.total_ticks - start >= max_ticks { return false; }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
            let dx = target[0] - bp[0];
            let dz = target[2] - bp[2];
            let dy = target[1] - eye;
            if (dx * dx + dy * dy + dz * dz).sqrt() > range { return false; }
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on_target = err.abs() < 0.01 && perr.abs() < 0.015 && w.total_ticks - last_tap > 4;
            let trigger = on_target;
            if trigger { last_tap = w.total_ticks; }
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

/// THE HEADLINE — the level completes through the real input: the ST-3 plate walk-in (ch 0x1d,
/// objective 4 [D #378; M 0x1827d8:88-99]), the camera photograph of the laser prototype (ch 0x1e,
/// objective 5 [M CamSubject_Update 0x1bc668:14-24]), the C5 corridor plate (ch 0x28 [D #433]),
/// then the C5 door `bigdoor02` #96 SHOT TO DEATH — `Destroy_CollisionHandler` 0x12bd48 subtracts
/// every class-5 bullet's damage (the 0x67 special case only dims the lights, :31-33) and the
/// death writes switch_channels[0x1f] [M :35] — objective 6; mux #434 [0x1e & 0x1f & 0x28] →
/// ch 100 [D list] → `Mission_Update` state 6 → 0x730000d [M 0x182c60:102-103]. Sister: ch 0x1f
/// and ch 100 read off before the shooting while everything else is already in.
#[test]
fn c5_shot_open_completes_the_level() {
    let mut w = load_warm();
    w.top_up_health = true; // the door's death blast (kind 6, damage 1000 [D p5]) may reach Bond
    to_yard(&mut w);
    w.walk_to(Wp::new(136.5, 4.5, -290.0).r(2.0), "the yard south 1");
    w.walk_to(Wp::new(136.0, 4.5, -308.5).r(1.2), "the plate");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x1d), "the ST-3 plate #378 → ch 0x1d [D]");
        assert_eq!(sim.drones.mission.states[12][0], 4, "objective 4 completes [M 0x1827d8:88-99]");
    }
    // The photograph: the camera into subject #494's volume [D #494; M 0x1bc668].
    let subject = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.props.subjects.iter().find(|s| s.channel == 0x1e && s.placement == 494).expect("subject #494").clone()
    };
    let sc = [(subject.lo[0] + subject.hi[0]) / 2.0, (subject.lo[1] + subject.hi[1]) / 2.0, (subject.lo[2] + subject.hi[2]) / 2.0];
    w.walk_to(Wp::new(137.0, 5.2, -320.0).r(1.5), "the display trench");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.gun.switch_request = 0x54;
    }
    w.wait(40);
    assert!(shoot_at(&mut w, sc, 40.0, 900, &|sim| sim.drones.channels.on(0x1e)), "the photograph writes ch 0x1e [M 0x1bc668]");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for _ in 0..2 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        assert_eq!(sim.drones.mission.states[12][1], 4, "objective 5 completes [M 0x1827d8:88-99]");
    }
    // The shooting stand FIRST (the corridor plate #433 is a re-firing Touch whose exit clears
    // ch 0x28 [M triggers.rs] — the corridor walk must come after the kill): the yard face of
    // bigdoor02 #96 (bounds 131.78..137.08, 3.99..7.35, −269.85..−269.15 [D placed gfx]), 11+ m
    // out — outside the death blast's reach (blast damage 1000 [D p5]; top_up_health guards).
    // The weapon is the level's own MOUNTED LASER 0x67 (the catwalk gun [D hint 0x4000066]): the
    // mounted class sustains fire with no ammo pool [L probe: clip 0/spare 0 keeps firing, 50
    // damage a hit; M Destroy_CollisionHandler subtracts every class-5 contact] — 200 hits for
    // the 10000 health. The carried loadout cannot (0x1e: 35 × 40 = 1400 [D ammo caps]).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [135.0, 4.5, -281.0];
        sim.bond.vel = [0.0; 3];
        let wd = sim.weapons.clone();
        let rounds = wd.clip_size(0x67).max(1);
        let _ = sim.drones.inventory.equip_weapon(&wd, 0x67, rounds, true);
        sim.gun.switch_request = 0x67;
    }
    w.wait(30);
    let door = |w: &Walker| w.scene.sim.as_ref().unwrap().drones.props.destroy.iter()
        .find(|d| d.placement == 96).cloned().unwrap();
    let d0 = door(&w);
    assert!(!d0.dead && d0.health > 9000.0, "sister: bigdoor02 stands at ~10000 health [D p3], got {}", d0.health);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x1f), "sister: ch 0x1f off before the shooting");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x1e) && !w.scene.sim.as_ref().unwrap().drones.channels.on(0x28),
        "the photograph input is latched; the corridor input comes after the kill [D #434 list]");
    let target = [134.5, 5.6, -269.4];
    let killed = shoot_at(&mut w, target, 25.0, 30000, &|sim| sim.drones.channels.on(0x1f));
    assert!(killed, "shooting bigdoor02 (health 10000 [D p3]) must write ch 0x1f [M 0x12bd48:35]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(door(&w).dead, "the leaf is dead");
        for _ in 0..2 {}
        assert!(sim.drones.channels.on(0x1f), "ch 0x1f latched [M 0x12bd48:35]");
    }
    // Now the corridor walk: through the door plane to the slope — the re-firing plate #433's
    // band raises ch 0x28 on the low crossing (the walkreal's pin) and leaves it ON while the
    // crossing continues [D; M triggers.rs].
    w.walk_to(Wp::new(136.0, 4.2, -278.0).r(2.0), "the yard north");
    w.walk_to(Wp::new(136.0, 5.2, -267.5).r(1.5), "the corridor slope");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x28), "the corridor plate #433 fires ch 0x28 [D]");
    // The mux → ch 100 → mission end → 0x730000d [M 0x182c60:102-103]. The end sequence runs
    // `Mission_Update`'s state-6 timeout (VIDEO_FRAME_RATE << 2 = 240 ticks [M :88-91]) before
    // `LevelEnd` → `ResetMap_LevelToLoad` [M :65-76] — wait it out.
    let mut ended = false;
    for _ in 0..120 {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(100) { ended = true; break; }
    }
    assert!(ended, "mux #434 [0x1e & 0x1f & 0x28] must raise ch 100 [D; M Trigger_Update's AND arm]");
    let mut exit = None;
    for _ in 0..400 {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.mission.status() == 6 { exit = sim.drones.level_exit.or(sim.drones.mission_end); if exit.is_some() { break; } }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.mission.status(), 6, "Mission_Update state 6 [M 0x182c60]");
    assert_eq!(exit, Some(0x0730_000d), "the level hands off to the driving level [D record; M 0x182c60:102-103], got {exit:?}");
}

/// The use-key plate kind (0xEA mask 4 [M Trigger_Activate 0x1b2318:23-33]): E the Overhead
/// Crane touchplate #460 → ch 0x2b; the channel arms the whole crane chain — the shipping_handle
/// SS #112 (state 0x40 re-sweeps on SS_Operate's channel half [M 0x1a9df8:47-51]) and the
/// ScriptPlayer #115 (mask 1 = its out-channel start bit [M SP_Update 0x1dbda8 flags&1]). Sisters:
/// ch 0x2b off, the SS at its key-0 pose and SP #115 unstarted before the E.
#[test]
fn crane_e_plate_arms_the_handle_ss_and_the_shipping_sp() {
    let mut w = load_warm();
    let mut held_delay = 0i32;
    // The plate stands on the crane dock (189.02, 9.72, −288.92 [D]; the dock floor y ≈ 9.7).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [189.0, 9.9, -287.6];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x2b), "sister: ch 0x2b off before the E");
        let ss = &sim.drones.sss.list[0];
        assert_eq!(ss.placement, 112, "the shipping_handle SS [D #112]");
        assert_eq!(ss.state, 0x40, "the handle rests armed [D p0 = 0x40]");
        // SP #115 is mode 8 — the create-time load beat primes it HELD in state 6 [M
        // SP_LoadScript's mode dispatch; Player::update state-6 arm]: script loaded, delay held.
        let sp = &sim.drones.scripts.players.iter().find(|p| p.placement == 115).unwrap();
        assert_eq!(sp.state, 6, "sister: SP #115 primed-held in state 6 [M 0x1dbda8 mode 8]");
        assert!(sp.info[sp.cur].is_some(), "the 0x060009EF script is preloaded [D #115 p0]");
        let d0 = sp.info[sp.cur].as_ref().unwrap().delay;
        assert!(d0 != 0, "sister: the held script waits (delay {d0})");
        held_delay = d0;
    }
    let start_z = w.scene.sim.as_ref().unwrap().drones.sss.list[0].pos()[2];
    w.walk_to(Wp::at(189.0, 9.9, -288.7, Act::Use { use_ticks: 240 }), "E the crane touchplate");
    w.wait(10);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x2b),
        "the E fires the plate's out ch 0x2b [D #460 p0; M Trigger_Activate 0x1b2318:23-33]");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The SS swept off its key-0 pose (2 keys, rate 75 [D]; the wrap p1 = 1 disarms it at
        // state 0 holding the end pose [M 0x1a9948:121-126]).
        let ss = &sim.drones.sss.list[0];
        let moved = (ss.pos()[2] - start_z).abs();
        assert!(moved > 0.01 || ss.state == 0, "the handle SS swept (dz {moved}, state {:#x})", ss.state);
        assert_eq!(ss.state, 0, "the wrap holds the end pose disarmed [M 0x1a9948:121-126]");
        // The ch-0x2b edge releases the held script: the state-6 arm zeroes the delay [M
        // 0x1dbda8 state-6 `s.delay = 0`] and the 3000-tick shipping-crane script plays.
        let sp = &sim.drones.scripts.players.iter().find(|p| p.placement == 115).unwrap();
        assert!(sp.info[sp.cur].is_some(), "the 0x060009EF script is loaded [D #115 p0]");
        let d1 = sp.info[sp.cur].as_ref().unwrap().delay;
        assert!(d1 == 0 || sp.state == 2 || sp.state == 3,
            "SP #115's held script must release on the ch-0x2b edge [M SP_Update state-6 arm], delay {held_delay} → {d1} state {}", sp.state);
    }
}

/// The crane-rotate chain: E the cab touchplate #450 (199.53, 13.31, −263.81 [D], the crane cab
/// level) → ch 0xa → ScriptPlayer_CraneRotate #492 (mode 64, mask 1, out 0xa [D]) plays
/// 0x06000A08 (750 ticks [D]) — the crane arm's script entities move. Sister: ch 0xa off and
/// SP #492 unstarted before the E.
#[test]
fn crane_rotate_e_plate_starts_sp492() {
    let mut w = load_warm();
    // The cab platform (y ≈ 13.3 [D plate box 12.1..14.5]) — the crane route's own level; the
    // walk there runs through the unported crane ride (walkreal's pinned gap) — the stand-in
    // reaches the cab, the E is the real input.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.5, 13.5, -264.3];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0xa), "sister: ch 0xa off before the E");
        let sp = &sim.drones.scripts.players.iter().find(|p| p.placement == 492).unwrap();
        assert_eq!(sp.state, 1, "sister: SP #492 primed-waiting in state 1 [M 0x1dbda8 mode 64 load beat]");
    }
    // The cab walk is furniture-wedged — the E holds from the stand (the rooftop-vent pattern):
    // face the plate (its box z −264.9..−262.7, y 12.1..14.5 [D]) and hold the use key. The stand
    // is on the touchplate [D crane_touchplate's own sliver sheets z −263.94..−262.73, x
    // 198.33..200.72].
    // The plate's sliver sheets slant (the B vertex at (199.00, −262.73) is the northmost
    // point); from the settled stand the +z look's use point lands north of the B→C edge — the
    // west look puts it across the sliver's west sheet [D crane_touchplate tris].
    w.scene.sim.as_mut().unwrap().bond.yaw = -std::f32::consts::FRAC_PI_2;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), w.total_ticks % 6 == 0);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xa) { break; }
    }
    w.wait(10);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0xa),
        "the E fires ch 0xa [D #450 p0; M Trigger_Activate 0x1b2318:23-33]");
    w.wait(90);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let sp = &sim.drones.scripts.players.iter().find(|p| p.placement == 492).unwrap();
        assert!(sp.state == 2 || sp.state == 3 || sp.state == 4,
            "SP #492 plays the crane-rotate script on the ch-0xa edge [M SP_Update 0x1dbda8 state-1 arm], state {}", sp.state);
        assert!(sp.info[sp.cur].is_some(), "the 0x06000A08 script is loaded [D #492 p0]");
    }
}

/// The walk-in TouchOnce kind: the office plate #265 (194.97, 10.38, −278.76 [D], box y
/// 9.9..10.9) fires ch 1 on the walk-through — `Trigger_Activate`'s auto-fire [M triggers.rs].
/// Sister: ch 1 off before the crossing.
#[test]
fn walkin_plate_265_fires_ch1() {
    let mut w = load_warm();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(1), "sister: ch 1 off before the walk");
    }
    // The office sits on the crane dock (the dock floor y ≈ 10.4 there) — walked from the dock
    // stand (the start's own floor level).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [195.0, 10.2, -276.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    w.walk_to(Wp::new(194.97, 10.2, -278.76).r(0.8), "through the office plate");
    w.wait(10);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(1),
        "the walk-through fires ch 1 [D #265 p0; M Trigger_Update]");
}

/// The door kinds: the plain swing `obj_doortop` #100 (no channels [D]) opens on the E
/// [M Door_Activate]; the locked `fakedoor` #517 (ch_lock 200, which has no writer anywhere in
/// the level's data [D script scan]) refuses the same E.
#[test]
fn doors_open_on_use_and_locked_refuse() {
    let mut w = load_warm();
    to_yard(&mut w);
    // The doortop pair stands at the ST-3 yard's north-east door (120.62/119.84, ~9.3/5.3, −280.5 [D]).
    let (di_unlock, dpos) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let di = sim.drones.doors.list.iter().position(|d| d.placement == 100).expect("doortop #100");
        (di, sim.drones.doors.list[di].pos)
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [dpos[0], dpos[1] - 0.6, dpos[2] + 1.4];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let before = w.scene.sim.as_ref().unwrap().drones.doors.list[di_unlock].state;
    w.use_door(di_unlock, "the yard doortop");
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di_unlock].state;
    assert!(st & 0xc != 0, "the channel-free doortop opens on the E [M Door_Activate], state {before:#x} → {st:#x}");
    // The locked fakedoor #517 on the TR-2 roof walkway (147.12, 22.94, −258.12 [D]; the guards
    // stand that roof at y 22.7 [D drones]) — the stand-in reaches the roof (the guard spawn
    // catwalk has no walked approach on this level's nav web), the E is the real input.
    let di_lock = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.doors.list.iter().position(|d| d.placement == 517).expect("fakedoor #517")
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let dp = sim.drones.doors.list[di_lock].pos;
        sim.bond.pos = [dp[0], dp[1] - 0.6, dp[2] + 1.4];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let before = w.scene.sim.as_ref().unwrap().drones.doors.list[di_lock].state;
    w.use_door(di_lock, "the locked fakedoor");
    w.wait(10);
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[di_lock].state, before,
        "the fakedoor locked on ch 200 refuses the E while the channel has no writer [D p3; M Door_Activate's lock arm]");
}

/// The CamSubject kind, second placement: the big ST-3 display #495 (bounds x
/// 120.99..121.78, y 8.55..13.77, z −328.68..−318.16 [D placed gfx]) photographs from the yard's
/// west edge — the weapon-0x54 bullet writes ch 0x1e [M CamSubject_Update 0x1bc668:14-24].
/// Sister: ch 0x1e off before the shot.
#[test]
fn photo_subject_495_writes_ch1e() {
    let mut w = load_warm();
    to_yard(&mut w);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x1e), "sister: ch 0x1e off before the shot");
        assert_eq!(sim.drones.props.subjects.len(), 2, "both display subjects exist [D #494/#495]");
    }
    // The shot stands in the yard's trench mouth facing the tall thin display 16 m west.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [137.0, 5.2, -323.0];
        sim.bond.vel = [0.0; 3];
        sim.gun.switch_request = 0x54;
    }
    w.wait(40);
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x54, "the camera is held [M 0x1a7ef8]");
    let target = [121.4, 10.5, -323.4];
    let shot = shoot_at(&mut w, target, 30.0, 1500, &|sim| sim.drones.channels.on(0x1e));
    assert!(shot, "photographing the big display writes ch 0x1e [D #495 p0 = 0x1e; M 0x1bc668]");
}

/// The destroyable kind, the oil-barrel family: shooting barrel #311 (health 20, kind 4, blast
/// 500 [D]) kills it and `Destroy_Smash` queues the blast script 0x0600052 [M 0x12bd48 death path;
/// props.rs]. Sister: the barrel stands at health 20 before the shot.
#[test]
fn oil_barrel_shoot_smashes_with_blast() {
    let mut w = load_warm();
    to_yard(&mut w);
    let b0 = w.scene.sim.as_ref().unwrap().drones.props.destroy.iter()
        .find(|d| d.placement == 311).cloned().expect("barrel #311");
    assert!(!b0.dead && (b0.health - 20.0).abs() < 0.5, "sister: the barrel stands at 20 [D p3]");
    let target = [(b0.lo[0] + b0.hi[0]) / 2.0, (b0.lo[1] + b0.hi[1]) / 2.0, (b0.lo[2] + b0.hi[2]) / 2.0];
    // The shot stands 4 m north-west of the barrel (142.68, 4.7, −308.15 [D]), out of the 500
    // blast's core; top_up_health guards the splash.
    w.top_up_health = true;
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [139.5, 4.5, -305.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let killed = shoot_at(&mut w, target, 15.0, 3000, &|sim| {
        sim.drones.props.destroy.iter().find(|d| d.placement == 311).map(|d| d.dead).unwrap_or(false)
    });
    assert!(killed, "the shot kills the 20-health barrel [M 0x12bd48 every class-5 contact]");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.props.destroy.iter().find(|d| d.placement == 311).unwrap().dead, "the leaf is dead");
}

/// The pickup kinds on the walk-over [M Pickup_Handler 0x18e650]: the BodyArmourHeavy #442
/// (kind 3, 50 [D]) sets armour 50; the sniper-ammo pickup #522 (kind 1, weapon 0x1f, 5 rounds
/// [D]) grants weapon 0x1f; a bond bonus #439 (kind 6 [D]) is consumed. Sisters: armour 0 / the
/// rifle un-owned / the bonus sitting before.
#[test]
fn pickups_taken_by_walking_over() {
    // The kind-6 row (#439) creates only with the level's gold medal recorded [M Pickup_Create
    // 0x18dda0:93-95]: 0700000d plays under BaseMap 0x0700000c (the mission table [L ee_start_05
    // 0x2a4350 row 12], the gold bit = the 0x070000c row's counter-3 bit 40 [L 0x2bc920 row 6,
    // desc[3] = (40, 1, 0x4000c) — kind 1, a plain bit, no group]) — the medaled load.
    let mut p = nf_game::progress::Progress::default();
    p.set_level_bonus(0x0700_000c, 3, 0); // [M Menu_SetLevelBonus 0x1ffa10]
    let mut w = load_warm_bits(p.bonus[0]);
    w.top_up_health = true;
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.health.armour, 0.0, "sister: no armour before");
    assert!(!w.scene.sim.as_ref().unwrap().drones.inventory.owned[0x1f], "sister: weapon 0x1f un-owned before");
    // The armour on the crane dock (192.82, 10.79, −272.03 [D]).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [192.8, 10.4, -270.0];
        sim.bond.vel = [0.0; 3];
    }
    w.walk_to(Wp::new(192.82, 10.4, -272.03).r(1.4), "onto the BodyArmourHeavy");
    w.wait(20);
    assert!((w.scene.sim.as_ref().unwrap().drones.health.armour - 50.0).abs() < 1.0,
        "the walk-over grants armour 50 [M 0x18e650 case 3], got {}",
        w.scene.sim.as_ref().unwrap().drones.health.armour);
    // The sniper-ammo pair (214.2, 11.09, −289.96 [D #522/#545]) PASS the distance+LOS gates
    // [NF_PICKUPLOG: dist 1.04 vs r 1.50, los true] but the take refuses: weapon 0x1f's
    // first-person model is one of the session merge's 28 still-missing hashes, and
    // `equip_weapon` refuses a weapon with no first-person model [M pickup.rs
    // `first_person(id) == 0` gate] — the models lane's finding, in the report's list.
    // The west-yard bond bonus (95.98, 6.79, −276.41 [D #439]) is consumed by the same path.
    let taken = |w: &Walker| w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .filter(|p| (p.m[3][0] - 95.98).abs() < 0.5 && (p.m[3][2] + 276.41).abs() < 0.5)
        .all(|p| p.gone || p.hidden);
    assert!(!taken(&w), "sister: the bonus sits before the walk");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [96.0, 6.3, -276.4];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    assert!(taken(&w), "the bond bonus is consumed on the walk-over [M 0x18e650:234-236]");
    // The no-medal arm [M 0x18dda0:93-95]: the same warm load with the fresh-boot zero word
    // creates no kind-6 at all — #439's spot holds nothing.
    let mut bare = load_warm_bits(0);
    bare.top_up_health = true;
    assert!(bare.scene.sim.as_ref().unwrap().drones.pickups.iter().all(|p| p.kind != nf_game::pickup::kind::BOND_BONUS),
        "no gold medal → the kind-6 bonus is not created [M 0x18dda0:93-95]");
}

/// The zip-wire kind: the three wires created with the disc's spheres [D #271/#484/#487;
/// Wire_Create 0x1981d8]. The grab+ride mechanism itself is fix-wire's system pin (green on
/// 07000005/07000002); the 0d instance's grab is a live check (the walk-in grab did not latch
/// from any probed stand on this level — the report's list).
#[test]
fn zipline_wires_created_with_disc_spheres() {
    let mut w = load_warm();
    let wires: Vec<([f32; 3], f32, f32)> = w.scene.sim.as_ref().unwrap().wires.iter()
        .map(|x| (x.centre, x.radius, x.yaw)).collect();
    assert_eq!(wires.len(), 3, "the three wires [D #271/#484/#487]");
    assert!(wires.iter().any(|(c, r, _)| (c[0] - 205.61).abs() < 0.5 && (c[2] + 248.52).abs() < 0.5 && (*r - 14.02).abs() < 0.1),
        "wire_crane2crane's sphere [D #271], got {wires:?}");
    assert!(wires.iter().any(|(c, r, _)| (c[0] - 187.97).abs() < 0.5 && (c[2] + 219.12).abs() < 0.5 && (*r - 13.12).abs() < 0.1),
        "the Zipline's sphere [D #484], got {wires:?}");
    assert!(wires.iter().any(|(c, r, _)| (c[0] - 171.52).abs() < 0.5 && (c[2] + 236.30).abs() < 0.5 && (*r - 18.98).abs() < 0.1),
        "the Zipline_2's sphere [D #487], got {wires:?}");
    assert!(w.scene.sim.as_ref().unwrap().bond.wire.is_none(), "sister: not riding at load");
}

/// The ladder kind: the crane-leg ladder #435 (centre 212.47, 8.14, −234.51, r 8.2 [D]) —
/// walking into its sphere grabs it and the forward input climbs [M Player_CollLadder 0x196118].
/// Sister: on the ground before the grab.
#[test]
fn crane_ladder_grab_and_climb() {
    let mut w = load_warm();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [211.5, 8.4, -233.2];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let y0 = w.scene.sim.as_ref().unwrap().bond.pos[1];
    let mut climbing = false;
    for _ in 0..300 {
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.pos[1] > y0 + 2.0 { climbing = true; break; }
    }
    assert!(climbing, "the ladder grab + climb raises Bond [M Player_CollLadder], y {:.2} → {:.2}",
        y0, w.scene.sim.as_ref().unwrap().bond.pos[1]);
}

/// The grapple kind: the Q-Claw (owned on this loadout [L]) reels Bond to pulley #259
/// (213.62, 15.77, −234.77 [D]) through the real fire path [M 0x192860]. Sister: on foot
/// (substate walk) before the shot.
#[test]
fn qclaw_reels_to_the_crane_pulley() {
    let mut w = load_warm();
    // Stand on the dock under the pulley, ~4 m away.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [212.4, 11.0, -236.6];
        sim.bond.vel = [0.0; 3];
        sim.gun.switch_request = 0x50;
    }
    w.wait(60);
    let target = [213.62f32, 15.77, -234.77];
    for _ in 0..200 {
        let (turn, look) = aim_at(&w, target);
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { turn, look, ..Default::default() }, false, false, false);
    }
    let mut hung = false;
    for _ in 0..1200 {
        let (turn, look) = aim_at(&w, target);
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { turn, look, ..Default::default() }, true, false, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grapple.as_ref().map(|g| g.phase) == Some(2) { hung = true; break; }
    }
    assert!(hung, "the claw hangs Bond at the pulley [D #259; M 0x192860], phase {:?} pos {:?}",
        w.scene.sim.as_ref().unwrap().bond.grapple.as_ref().map(|g| g.phase),
        w.scene.sim.as_ref().unwrap().bond.pos);
}

fn aim_at(w: &Walker, target: [f32; 3]) -> (f32, f32) {
    let sim = w.scene.sim.as_ref().unwrap();
    let bp = sim.bond.pos;
    let dx = target[0] - bp[0];
    let dz = target[2] - bp[2];
    let dy = target[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
    let desired = dx.atan2(dz);
    let err = walker::wrap_pi(desired - sim.bond.yaw);
    let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
    (err.clamp(-0.08, 0.08), (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08))
}
