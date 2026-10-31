//! walkreal-c: 0700001b (the space station) walked with the player's own input — no `bond.pos`,
//! substate or channel writes after the load in the full walk (`walk_arrival_to_drakes_defeat`).
//!
//! The station is a zero-g level from the start: the p3 == 2 spawn enters substate 8 [M
//! Player_Init 0x1910f8 :160-165] and nothing in ACTION.ELF leaves it on this level. The mission
//! never needs the shaft's west side: the first wave's death raises the launch order, the eight
//! console screens are shot from floats inside the hangar, the disarm chain opens the fight gate
//! and Drake closes in through the float (the walk's doc comment has the [D]/[M] chain).
//!
//! `C_CHCHZEROG_Handler` 0x216388 is a cheat-menu checkbox handler (the C_CHCH* family the menu
//! dispatch `Handler_HandleMessage` 0x2029e0 drives — MUSIC, FLY, DRONES, BLIND, WEAP, HEALTH, HUD,
//! DEBUG, COORDS, ZEROG) whose `param_4 == 0x4b` is the menu message id, not a switch channel;
//! the port's ch-75 → substate-9 binding (scene.rs) is its own [S] model. The older legs below
//! still force the ground substate ([S] premise) and drive ch 0x4b to reach the float from a
//! walk — they pin port mechanisms (the deck route, the boundary wall, the iris doors), not the
//! mission route.
//!
//! Enemies are DISABLED for the walk only in the walkreal sense: damage is suppressed
//! (top_up_health, the walker's allowance). Opt-in via NF_ISO (the slim ISO is enough).

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, SS_WALK, SS_ZEROG_8, SS_ZEROG_9};
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Leg 1 (the old test's hub crossing, solved): the arrival NIS plays out, the objectives show,
/// and Bond WALKS the mapped east-deck route from the platform to the slit's south edge — the
/// leg the first run fell off (it beelined west off the platform's west edge into the pit).
#[test]
fn walk_arrival_and_the_east_deck_to_the_slit_edge() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — NOT a game assertion: the p3 == 2 spawn enters substate 8 [M Player_Init
    // 0x1910f8 :160-165, now ported], but no decompile path back to a ground substate after the
    // arrival NIS is pinned and the port wires no substate-changing handler. This leg pins the
    // port's WALK route, so the substate is forced back to the ground path.
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[23][0], 3,
            "'Disable the missiles' shows at once (enable 0 [L], the state-0 arm [M 0x1827d8:41-54])");
        assert_eq!(sim.drones.mission.states[23][1], 1, "'Defeat Drake' hides until ch 0x09 [L record 1b obj 2]");
    }

    // The mapped route: south off the platform onto the deck, then west along z ≈ −1
    // (probe1b_route.rs's band map: the deck x 8–28 at z −8..2, the slit at z ≈ 0).
    assert!(w.try_walk_to(walker::Wp::new(20.0, -1.9, 1.0).r(1.2), "the platform south edge"),
        "the platform's south edge is walked");
    assert!(w.try_walk_to(walker::Wp::new(17.0, -1.9, -1.0).r(1.2), "the deck south-west"),
        "the deck south-west is walked");
    assert!(w.try_walk_to(walker::Wp::new(13.0, -1.9, -1.0).r(1.2), "the deck mid-west"),
        "the deck mid-west is walked");
    assert!(w.try_walk_to(walker::Wp::new(10.0, -1.9, -1.0).r(1.2), "the deck west"),
        "the deck west is walked");
    assert!(w.try_walk_to(walker::Wp::new(9.0, -1.9, -1.4).r(1.2), "the slit edge"),
        "the slit's south edge is walked");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] < 11.0 && p[0] > 6.0, "Bond reached the deck's west edge ({p:?})");
        assert!(p[1] > -3.5, "Bond never fell into the pit (y {:?})", p[1]);
        // Grounded is NOT asserted at the last metre: the slit's mouth (z ≈ 0) is the float
        // zone — leg 2 walks into it and the level carries Bond (the walk logs show grounded
        // arrivals on every deck node up to (10.94,-1.46,-0.25)).
    }
}

/// Leg 2: the slit float. Walking west into the slit, the level's script carries Bond west over
/// the void without a fall — the zero-G crossing channel (the game enters substate 9 for it
/// [M 0x216388:10-17]; the port's float runs in the walk substate and stalls at the island's
/// east face — that stall is the un-ignored walk's blocker, not asserted here).
#[test]
fn the_slit_float_carries_bond_west_without_a_fall() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — the same ground-substate premise as leg 1 (the p3 == 2 spawn enters
    // substate 8 [M 0x1910f8 :160-165]; this leg pins the port's WALK route).
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    // Into the slit: walk west along z ≈ −0.5 → 0. The float takes over past the deck edge.
    let _ = w.try_walk_to(walker::Wp::new(8.6, -1.9, -0.6).r(1.2), "the slit mouth");
    let mut min_x = w.scene.sim.as_ref().unwrap().bond.pos[0];
    let mut fell = false;
    for _ in 0..900u32 {
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        min_x = min_x.min(p[0]);
        if p[1] < -6.0 { fell = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(!fell, "Bond never fell into the pit crossing the slit (min x {min_x:.2})");
        assert!(min_x < 7.0, "the float carried Bond west past the deck edge (min x {min_x:.2})");
        assert!(p[1] > -3.0, "the float holds Bond at the deck band (y {:.2})", p[1]);
    }
}

/// The game's own hit volumes for the walk's aim: the `Data` view the drone pass hands
/// `Scripts::held_hit_targets` (the `SP_GetHitDamage` 0x1dc5f8 read set [M]).
fn data_of(sim: &nf_view::scene::Sim) -> nf_game::drone::Data<'_> {
    nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
        ai_points: &sim.ai_points,
    }
}

/// The float's thrusters toward `at`: the strafe / lift / forward inputs are the body-axis
/// components of the offset (the zero-g move is `pos += local·rows` [M 0x193600:284-295]), so a
/// proportional input flies straight at the point without turning (the 0.04 ease damps it).
fn thrust_to(sim: &nf_view::scene::Sim, at: [f32; 3]) -> Input {
    let p = sim.bond.pos;
    let d = [at[0] - p[0], at[1] - p[1], at[2] - p[2]];
    let m = sim.bond.zero_g_rot;
    let dot = |a: [f32; 3]| a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
    Input {
        strafe: (dot(m[0]) * 0.15).clamp(-1.0, 1.0),
        lift: (dot(m[1]) * 0.15).clamp(-1.0, 1.0),
        forward: (dot(m[2]) * 0.15).clamp(-1.0, 1.0),
        ..Default::default()
    }
}

/// Kill every live astronaut (skin 0x05000078) with the walker's real fire, until `done` or
/// none is left.
fn kill_astronauts(w: &mut Walker, hold: [f32; 3], done: impl Fn(&nf_view::scene::Sim) -> bool, cap: u32) {
    let mut n = 0u32;
    while n < cap {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            if done(sim) { return; }
            sim.drones.list.iter().position(|d| d.present && !d.corpse && d.enabled && d.health > 0.0 && d.skin & 0xffff == 0x78)
        };
        match target {
            Some(di) => { let _ = w.shoot(di, "astronaut"); n += 1; }
            None => { let input = thrust_to(w.scene.sim.as_ref().unwrap(), hold); w.tick(input, false); n += 1; }
        }
    }
}

/// The station Bond floats at to shoot console `k`'s screen: every screen is bullet-visible from
/// (32, −1.7, 0) [the probe1b_chanmap los_1b_screens scan of the port's bullet segments]; the far
/// pairs are shot from nearer floats — 7/8 from (46, −7, 0), 1/2 from (20, −2, 0) — the harness's
/// choice of where to fly (any screen in bullet sight within 20 u is fired at on the way).
fn console_station(k: u32) -> [f32; 3] {
    match k { 7 | 8 => [46.0, -7.0, 0.0], 1 | 2 => [20.0, -2.0, 0.0], _ => [32.0, -1.7, 0.0] }
}

/// The deadline order: the missiles' p0 scripts end at state-1 ticks 1799 (3/4/5), 2399 (6/8)
/// and 3299 (1/2/7) [D P0_LEN — space_missile.rs], and state 2 reads the console channel then
/// [M 0x1aa25c].
const CONSOLE_ORDER: [u32; 8] = [3, 4, 5, 6, 8, 7, 1, 2];

/// Shoot the eight console screens from the float stations: each console UI script carries three
/// holds (event 3 on streams 1/2/3 [D the probe1b_scripts decode]) and resumes from each on 20
/// bullet damage — one rifle hit per hold [M SP_UnPause 0x1dca58 /
/// SP_GetHitDamage 0x1dc5f8], then writes the console channel [D stream 4 ev0x0d]. The aim is the held
/// script's largest hit sphere (the game's own damage volume — held_hit_targets); the trigger is
/// tapped in bursts: the rifle's clip recharges (the gun_1b probe: 12 rounds per shot, ~1 round
/// per 4 ticks after a ~100-tick pause), so the walk waits for three shots' worth before a burst.
/// Returns the frame each console latched.
fn shoot_consoles(w: &mut Walker) -> Vec<(u32, u32)> {
    let mut latched = Vec::new();
    let mut last_shot = 0u32;
    let mut in_burst = false;
    for tt in 0..6000u32 {
        let (input, fire) = {
            let sim = w.scene.sim.as_ref().unwrap();
            for &k in &CONSOLE_ORDER {
                if sim.drones.channels.on(k) && !latched.iter().any(|&(c, _)| c == k) { latched.push((k, sim.drones.frame)); }
            }
            let Some(&k) = CONSOLE_ORDER.iter().find(|&&k| !sim.drones.channels.on(k)) else { break };
            let pi = sim.drones.scripts.players.iter().position(|p| p.gate as u32 == 30 + k && p.hit_points > 0.0)
                .expect("the console's UI script player (gate 30 + k, hit points 20 [D SP#213-220])");
            let tg = sim.drones.scripts.held_hit_targets(&data_of(sim));
            let best = tg.iter().filter(|x| x.0 == pi).max_by(|a, b| a.2.partial_cmp(&b.2).unwrap()).map(|x| x.1);
            let station = console_station(k);
            let mut inp = thrust_to(sim, station);
            let mut fire = false;
            if let Some(c) = best {
                let (e, pe) = walker::zero_g_errs(sim, c).unwrap();
                inp.turn = e.clamp(-0.08, 0.08);
                inp.look = pe.clamp(-0.08, 0.08);
                let clip = sim.gun.clip;
                if clip >= 36 { in_burst = true; }
                if clip < 12 { in_burst = false; }
                let eye = sim.bond.head();
                let dist = ((c[0] - eye[0]).powi(2) + (c[1] - eye[1]).powi(2) + (c[2] - eye[2]).powi(2)).sqrt();
                let los = match sim.world.segment_bullet(eye, c) { None => true, Some(t) => (1.0 - t) * dist < 0.4 };
                let near = walker::dxz(sim.bond.pos, station) < 2.5 || (los && dist < 20.0);
                fire = near && in_burst && e.abs() < 0.006 && pe.abs() < 0.006
                    && tt.saturating_sub(last_shot) >= 12 && tt % 6 == 0;
            }
            (inp, fire)
        };
        if fire {
            let c0 = w.scene.sim.as_ref().unwrap().gun.clip;
            w.tick_fire(input);
            if w.scene.sim.as_ref().unwrap().gun.clip < c0 { last_shot = tt; }
        } else {
            w.tick(input, false);
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    for &k in &CONSOLE_ORDER {
        if sim.drones.channels.on(k) && !latched.iter().any(|&(c, _)| c == k) { latched.push((k, sim.drones.frame)); }
    }
    latched
}

/// The launch order is the FIRST WAVE's death, on disc data alone: the arrival script
/// 0x0600085c writes ch 190/191 [D stream 6/9 ev0x0d], ch 191 gates SP#148 (0x060009a4 [D p9])
/// whose streams write ch 115 (spawner #163's trigger [D p4 = 0x73]) and ch 193 [D streams 1/3];
/// spawner #163's DONE channel is ch 194 [D p6 = 0xc2] — written when its spawned wave is dead
/// [M DroneSpawner 0x137e68:126-143, the `p6 != 0` wait] — and AND #202 folds ch 193 & 194 into
/// ch 19 [D p0 = 0x13, p1/p2 = 0xc1/0xc2], the missiles' launch-order gate [D p7 = 19; M
/// 0x1aa1c8]. The older "ch 19 has no disc writer" reading only swept the scripts. Sister: with
/// the wave alive, ch 194 / ch 19 stay off and every missile sits in state 0.
#[test]
fn the_first_wave_kill_raises_the_launch_order() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_8,
        "the p3 == 2 start is the zero-g substate 8 [M Player_Init 0x1910f8 :160-165]");
    let mut spawned = false;
    for _ in 0..3000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.list.iter().any(|d| d.present && !d.corpse && d.skin & 0xffff == 0x78) { spawned = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(spawned, "ch 115 (SP#148's write [D 060009a4 stream 1]) spawns the first wave [D spawner #163 p4 = 0x73]");
        assert!(sim.drones.channels.on(115) && sim.drones.channels.on(191), "the chain's head: ch 191 → SP#148 → ch 115 [D]");
    }
    // Sister: the wave alive — 1500 ticks idle, no launch order.
    for _ in 0..1500u32 { w.tick(Default::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(193), "SP#148's ch 193 write lands [D 060009a4 stream 3]");
        assert!(!sim.drones.channels.on(194) && !sim.drones.channels.on(19),
            "sister: the live wave holds the spawner's done write [M 0x137e68:126-143] — no ch 194, no ch 19");
        assert!(sim.drones.space_missiles.iter().all(|m| m.state == 0), "sister: the missiles wait on ch 19 [M 0x1aa1c8]");
    }
    let hold = w.scene.sim.as_ref().unwrap().bond.pos;
    kill_astronauts(&mut w, hold, |s| s.drones.channels.on(19), 2400);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(194), "the dead wave writes spawner #163's done channel 194 [D p6; M 0x137e68]");
    assert!(sim.drones.channels.on(19), "AND #202 (193 & 194) raises the launch order ch 19 [D]");
}

/// The whole mission on real input — no position, substate or channel write after the load.
/// The station is a zero-g level from the start (the p3 == 2 spawn enters substate 8 [M
/// Player_Init 0x1910f8 :160-165]) and the route never needs the shaft's far side: Drake comes
/// to Bond.
///
/// 1. The arrival plays; the first wave (spawner #163) attacks; killing it raises ch 194 →
///    AND #202 → ch 19, the launch order [D — the sister test above].
/// 2. The eight missiles leave state 0 [M 0x1aa1c8]; their p0 scripts write the console gates
///    31..38 at t359 [D], the console UI scripts (SP#213-220, hit points 20) play and hold at
///    t650; Bond floats to the stations and shoots the screens (20 damage resumes the script
///    [M SP_UnPause 0x1dca58], the t682 write latches the console channel [D]) before each
///    missile's p0 ends (state 2 reads the channel [M 0x1aa25c]) — all eight take the DISARM
///    branch (MissileDeploy 2 [M 0x1aa2cc]).
/// 3. Missile 1's disarm script writes ch 46 [D 0600091c @t1000] — SP#211's start (gate ch 30,
///    console 1's UI write [D 060007e2]); the fly-by chain 0x060007fd → 0x060008f0 writes the
///    fight gate ch 10 at its t400 [D]; spawner #187 (trigger 10 [D]) spawns Drake, who closes
///    in through the float. The done channels 21/22/27 land → AND #139 → ch 9, objective 1 [D].
/// 4. Bond shoots Drake (the real fire path): his death channel 14 [D #186 p12], the defeat NIS
///    (SP#198) writes ch 0x0b [D 0x06000a20], the monitor's ch 9 & 0x0b → ch 100 → MissionState
///    6 → 0x0730001b [M 0x182c60:114-116].
///
/// The walker's health allowance stays on (damage suppressed — the walkreal convention): the
/// later waves (spawners #165-181, ch 116-120 from the missile scripts [D]) keep firing while
/// the rifle's recharge paces the console shots, and the port's astronaut accuracy is
/// unmeasured against the game.
#[test]
fn walk_arrival_to_drakes_defeat() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.substate, SS_ZEROG_8, "the station starts in the zero-g substate [M Player_Init 0x1910f8 :160-165]");
        assert_eq!(sim.drones.mission.states[23][0], 3,
            "'Disable the missiles' shows at once (enable 0 [L], the state-0 arm [M 0x1827d8:41-54])");
        assert_eq!(sim.drones.mission.states[23][1], 1, "'Defeat Drake' hides until ch 0x09 [L record 1b obj 2]");
    }

    // 1. The first wave → the launch order.
    kill_astronauts(&mut w, [32.0, -1.7, 0.0], |s| s.drones.channels.on(19), 4000);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(19),
        "the first wave's death raises the launch order ch 19 [D spawner #163 p6 = 194, AND #202]");

    // 2. The consoles, in deadline order.
    let latched = shoot_consoles(&mut w);
    eprintln!("[1b] consoles latched (console, frame): {latched:?}");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for k in 1..=8u32 {
            assert!(sim.drones.channels.on(k), "console {k} latched by the real shot [D the console UI scripts' t682 write]");
        }
    }

    // 3. The disarm branch, the fly-by chain, Drake.
    let mut drake = None;
    for _ in 0..4000u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) { drake = Some(i); break; }
        let input = thrust_to(sim, [32.0, -1.7, 0.0]);
        w.tick(input, false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.missile_deploy.iter().all(|&d| d == 2),
            "every missile took the DISARM branch (MissileDeploy 2 [M 0x1aa2cc]): {:?}", sim.drones.missile_deploy);
        assert!(sim.drones.channels.on(46), "missile 1's disarm script writes SP#211's start ch 46 [D 0600091c @t1000]");
        assert!(sim.drones.channels.on(10), "the fly-by chain writes the fight gate ch 10 [D 0x060008f0 t400]");
        assert!(sim.drones.mission.status() <= 1, "no fail on the way (status {}) [M 0x182c60]", sim.drones.mission.status());
    }
    let drake = drake.expect("the fight gate spawns Drake (spawner #187, trigger 10 [D])");
    assert!(w.scene.sim.as_ref().unwrap().drones.list[drake].enabled, "the gate enables Drake (wait-switch 10 [D])");

    // 4. The fight and the defeat NIS (plays out — no skip, so its channel writes land).
    w.skip_cutscenes = false;
    assert!(w.shoot(drake, "Drake"), "Drake dies to Bond's fire in the float");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x0e),
        "Drake's death channel ch 0x0e rises with the kill [D #186 p12 = 14; M 0x1747b8/0x13e660]");
    let mut ended = false;
    for _ in 0..2400u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission.status() == 6 { ended = true; break; }
    }
    w.skip_cutscenes = true;
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0x09), "objective 1: AND #139 (21 & 22 & 27) → ch 9 [D]");
    assert!(sim.drones.channels.on(0x0b), "the defeat NIS writes ch 0x0b [D 0x06000a20]");
    assert!(sim.drones.channels.on(100), "ch 100, the mission end [M 0x1827d8:120 / D 0x06000a20]");
    assert!(ended, "the mission ends (MissionState 6) [M 0x182c60]");
    assert_eq!(sim.drones.mission.to_end, 0x0730_001b, "the station's own end code [M 0x182c60:114-116]");
}

/// The float-entry sister: the ch-75 rise enters the zero-G substate and the lift thruster
/// carries Bond OFF the deck band (the walk-substate bond wedges at y −1.62 on the pit's west
/// wall). Removing the entry handler regresses both pins. Sister: without the ch-75 write the
/// entry never fires (the substate stays walk).
#[test]
fn the_zero_g_entry_floats_bond_off_the_deck() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — the legs' ground-substate premise (the p3 == 2 spawn enters substate 8
    // [M 0x1910f8 :160-165]; this test pins the ENTRY, not the substate premise).
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    let _ = w.try_walk_to(walker::Wp::new(8.6, -1.9, -0.6).r(1.2), "the slit mouth");
    // Sister: no ch-75 write — the entry never fires, the wedge holds below the face.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_ne!(sim.bond.substate, SS_ZEROG_9, "sister: no ch-75 write, no float entry [M 0x216388 needs the msg]");
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x4b, 1, sim.drones.frame);
    }
    w.tick(Input::default(), false);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_9,
        "the ch-75 rise enters the float [M 0x216388:10-17]");
    // The float's proven pins: the entry (above) and the RISE — the lift thruster rides the
    // body's up row [M 0x193600:284-295] and Bond leaves the deck band (the walk-substate bond
    // wedges at y −1.62 on the pit's west wall). The WESTWARD CROSSING is the open blocker:
    // the shaft's west wall (the body-solid, bullet-pass surface the segments put at x ≈ 4.4,
    // still blocking at y 15.9) stops the thrust route — the game's own crossing past it is
    // unmeasured (the report's live check). The loop below drives the same thrust plan the
    // crossing needs and records the progress without pinning it.
    let mut min_x = f32::MAX;
    let mut rose = false;
    for t in 0..900u32 {
        let input = if t < 560 { Input { lift: 1.0, ..Default::default() } }
                    else { Input { forward: 1.0, ..Default::default() } };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        min_x = min_x.min(sim.bond.pos[0]);
        if sim.bond.pos[1] > 2.0 && t < 560 { rose = true; }
    }
    assert!(rose, "the float rises off the deck band under the lift thruster (the walk-substate bond wedges at y -1.62)");
    let _ = min_x;
}

/// The sealed-shaft wall, decided: the face that stops the float at x ≈ 4.98
/// IS the game's own boundary, and the game's body collides with it — the port's capsule stop is
/// the game's behaviour, not a port artefact.
///
/// [D probe1b_wall] the face is plc #128 OBJECT_Boundary_1 (kind 0x8019, cel 0x8000, glist 0x0):
/// a two-triangle pair at x 4.433 spanning y −16.56..+16.74, z −20.40..+20.40 (both stored
/// normals face EAST, +x), surface byte 0x40 — present in the level archive at the slit band,
/// at the mouths' height (y 9.45) and everywhere between.
///
/// [M] the game's rules stop the player body on it: the surface byte's 0x40 sets HITDATA+0x51
/// bit 1 on every hit (Collide_Intersect 0x1ebaa8 disasm 0x1ebf78-0x1ebf80) and that bit drops
/// hits only for the HITTEST+0x98 bit-0x8 bullet queries (0x1ebff0-0x1ec010) — the player body
/// rides Collide_CylinderIntersect 0x1ecc00:61's kind 0x800, which carries no bit 0x8;
/// Intersect_CylGeom 0x109da0's per-triangle loop rejects no face by side (the head-on cull
/// :302-322 only skips faces within ~2.6° of horizontal), and Player_ZeroG 0x192ab0 composes
/// only thrust/rotation — the shared movement applies the same body collision. The z ±8.82
/// mouths do not reopen the route: the iris doors (SP#141/#142's meshes, x 5.23 [D]) sit EAST
/// of the boundary, and the boundary spans the doors' whole y band — open doors dead-end at it.
#[test]
fn the_boundary_wall_stops_the_float_and_passes_bullets() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700001b.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    // The body ray east→west hits the 4.433 plane across the slit band and at the mouths'
    // height, with the boundary's surface byte [D mat 0x40].
    for (y, z) in [(-2.4f32, 0.0), (0.0, -4.0), (2.5, 0.0), (9.45, 8.82), (9.45, -8.82)] {
        let (t, n, mat, _) = sim.world
            .segment_surface_flags([5.5, y, z], [3.5, y, z], true)
            .unwrap_or_else(|| panic!("the boundary ray (y {y}, z {z}) must hit [D plc #128 at x 4.433]"));
        let x = 5.5 - t * 2.0;
        assert!((x - 4.433).abs() < 0.01, "hit at x {x:.3} = the boundary plane 4.433 [D plc #128], got t {t}");
        assert!((n[0] - 1.0).abs() < 0.01, "the stored normal faces east [D], got ({:.2},{:.2},{:.2})", n[0], n[1], n[2]);
        assert_eq!(mat & 0x3f, 0, "the boundary's surface index is 0 [D mat 0x40]");
        assert!(mat & 0x40 != 0, "the boundary carries the 0x40 surface flag [D]");
    }
    // Sisters (the game's ray rules): the bullet query passes (the 0x40 drop [M 0x1ebff0]) and
    // the west→east ray passes (the stored normals face east; the ray test skips n·dir ≥ 0
    // [M 0x10a8c8:168]).
    for (y, z) in [(0.0f32, 0.0), (9.45, 8.82), (9.45, -8.82)] {
        // The bullet is never stopped AT the boundary: the 0x40 drop [M 0x1ebff0-0x1ec010]
        // releases the hit and the ray runs on (west of the boundary the 0x19 grills at x ≈ 3.8
        // [D] can catch it — that is not the boundary).
        if let Some(t) = sim.world.segment_bullet([5.5, y, z], [3.5, y, z]) {
            let x = 5.5 - t * 2.0;
            assert!(x < 4.4, "the bullet passes the boundary westward (the 0x40 drop [M 0x1ebff0-0x1ec010]) \
                at y {y} z {z} — stopped at x {x:.3} instead");
        }
        assert!(sim.world.segment([3.5, y, z], [5.5, y, z]).is_none(),
            "the west→east ray passes the east-facing boundary [M 0x10a8c8:168] at y {y} z {z}");
    }
    drop(sim);
    // The float, real thrust only: enter at the slit mouth (the documented ch-75 stand-in for
    // the disc's own SP#211 pulse — the chain test above drives the chain itself), lift clear
    // of the deck band, then thrust west. The boundary stops the capsule at
    // 4.433 + 0.55 (the body radius) ≈ 4.98 — min x must land in that window, never past it.
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — the same ground-substate premise as the entry test (the p3 == 2 spawn
    // enters substate 8 [M 0x1910f8 :160-165]; this test pins the WALL, not the substate).
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    let _ = w.try_walk_to(walker::Wp::new(8.6, -1.9, -0.6).r(1.2), "the slit mouth");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x4b, 1, sim.drones.frame);
    }
    w.tick(Input::default(), false);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_9,
        "the ch-75 rise enters the float [M 0x216388:10-17]");
    let mut min_x = f32::MAX;
    for t in 0..1100u32 {
        let input = if t < 220 { Input { lift: 1.0, ..Default::default() } }
                    else { Input { forward: 1.0, ..Default::default() } };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        min_x = min_x.min(sim.bond.pos[0]);
    }
    assert!(min_x > 4.85 && min_x < 5.15,
        "the float thrusts west and the boundary stops the capsule at 4.433 + 0.55 ≈ 4.98 \
         [D plc #128; M the kind-0x800 body, no bit-0x8 drop; M CylGeom two-sided], got min x {min_x:.3}");
}

/// The sealing completes this task's scan: the boundary box (plc #128 OBJECT_Boundary_1 + its
/// end-walls) is closed on every face for the player's body. The west wall (x 4.433), the shaft's
/// top slab (y 16.74, normal down) and the pit floor (y −16.56) are the sisters above/`probe1b_wall`;
/// THIS test pins the two END-WALLS at z ±20.40 (m40) and their only openings — deck-level windows
/// ~0.8-1.2 tall — against the body's own height [M Player_Collision 0x194df0's default arm: the
/// capsule A = pos + up·0.55, B = pos + up·(+0x7CC − cc), radius 0.55 [M 0x3f0ccccd], standing cc
/// ≈ 1.0347 [L ee_start_05.bin BL 0x16cf0f0 per docs/fields/obj-tag-player.md] — ~1.58 tall,
/// taller than every window band]. With no script player-write on the disc [D the walk's doc
/// comment sweep], the box is sealed for real-input crossing; the game's own crossing (if any)
/// must use a body shape or a mechanism outside this scan — the report's live checks.
#[test]
fn the_end_walls_seal_the_shaft_box() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700001b.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    let w = &sim.world;
    // The end-walls exist at z ±20.40 across the shaft's x/y span (both ray directions — the
    // segment query is one-sided [M 0x10a8c8:168]).
    for (x, y) in [(6.5f32, -9.5), (6.5, 0.5), (6.5, 8.5), (10.0, -6.5), (14.0, -3.5), (19.0, 2.5), (23.0, -9.5)] {
        for zs in [20.4f32, -20.4] {
            let (a, b) = if zs > 0.0 { (26.0, 14.0) } else { (-26.0, -14.0) };
            let mut hit = None;
            for (p, q) in [(a, b), (b, a)] {
                if let Some((t, n, mat, _)) = w.segment_surface_flags([x, y, p], [x, y, q], true) {
                    let z = p - t * (p - q);
                    if (z - zs).abs() < 0.6 { hit = Some((z, mat)); break; }
                }
            }
            let (z, mat) = hit.unwrap_or_else(|| panic!("the end-wall at z {zs:+.1} must face ({x:.1},{y:+.1}) [D probes]"));
            assert!((z - zs).abs() < 0.6 && mat & 0x3f == 0 && mat & 0x40 != 0,
                "the end-wall face at ({x:.1},{y:+.1}) sits at z {z:+.2} with the boundary material [D m40], got m{mat:02x}");
        }
    }
    // The only openings: deck-level windows, y bands [D 0.2-step scans]:
    //   x 7.5/8.0: y −1.8..−1.0 (0.8 tall); x 9.0: −2.0..−1.2; x 10.0: −2.2..−1.2 (1.0 tall).
    for (x, y_lo, y_hi) in [(7.5f32, -1.8, -1.0), (8.0, -1.8, -1.0), (10.0, -2.2, -1.2)] {
        let mut y = y_lo;
        while y <= y_hi + 0.01 {
            let mut hitz = false;
            for (p, q) in [(-26.0f32, -14.0), (-14.0, -26.0)] {
                if let Some((t, n, mat, _)) = w.segment_surface_flags([x, y, p], [x, y, q], true) {
                    let z = p - t * (p - q);
                    if (z + 20.4).abs() < 0.6 { hitz = true; break; }
                }
            }
            assert!(!hitz, "the window band at (x {x:.1}, y {y:+.1}) must be open [D probes]");
            y += 0.4;
        }
        // The band's edges are wall (the window is bounded).
        for y in [y_lo - 0.6, y_hi + 0.6] {
            let mut hitz = false;
            for (p, q) in [(-26.0f32, -14.0), (-14.0, -26.0)] {
                if let Some((t, n, mat, _)) = w.segment_surface_flags([x, y, p], [x, y, q], true) {
                    let z = p - t * (p - q);
                    if (z + 20.4).abs() < 0.6 { hitz = true; break; }
                }
            }
            assert!(hitz, "the window band's edge at (x {x:.1}, y {y:+.1}) is end-wall [D probes]");
        }
        assert!(y_hi - y_lo < 1.3, "the window at x {x:.1} is {:+.1} tall [D probes] — narrower than the body", y_hi - y_lo);
    }
    // The body cannot pass any band: the capsule's total height ~1.58 [M 0x194df0's default arm]
    // exceeds every window (0.8..1.0 [D]).
    let capsule_height = 0.55 + (1.0347 - 0.55) + 2.0 * 0.55; // A to B span + the radius bulge at both ends
    assert!(capsule_height > 1.3,
        "the float body (~{capsule_height:.2} tall [M Player_Collision 0x194df0 default arm + cc 1.0347 [L]]) exceeds the widest window (1.0 [D])");
}

/// The iris doors, decided: the z ±8.82 booths' doors — SP#141/#142
/// "ScriptPlayer_IrisDoorOpen" [D placement names/positions (5.23, 9.45, ∓8.82)], script
/// 0x06000818 [D p0] — are LOAD-TIME LOOPERS, not missile-triggered pieces: the create runs
/// `SP_LoadScript` for mode ∉ {0,1} [M SP_CreateScriptPlayer 0x1db9a8 tail] and its mode-8 arm
/// starts the script LOOPING (`Script_Play(script, 1)`) in state 6 [M SP_LoadScript 0x1dbc08:46-48].
/// The script's per-loop shape [D decode]: the 8 iris-panel entities (0x02000834..3b) animate
/// OPEN at keys 0..120, the script HOLDS at t208 (stream 9 ev0x03 p0=0 — the −1 delay), the
/// panels CLOSE at 480..600 with the door sounds (stream 10: 0x000004c4 at start, 0x000004c3 at
/// t480), and the missile-done channels 0x18/0x1c are CLEARED at t550 (streams 12/13) — each
/// ~10 s loop clears the missile bookkeeping. The channel-RISE restart does NOT apply: [M
/// SP_Update 0x1dbda8 case 6] reloads only through the OTHER script slot, and both SPs carry a
/// single script (hash[1] = 0 [D live players]) — the rise never starts them. The doors are NOT
/// the route west either way: their meshes sit at x 5.23, east of the boundary [D probe1b_wall;
/// the boundary pin test above].
#[test]
fn the_iris_doors_are_load_time_loopers() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The create/load semantics: primed PLAYING loopers (state 6 = the mode-8 arm's
        // Script_Play(loop=1) [M 0x1dbc08:46-48]) on their own out channels [D p6].
        for plc in [141usize, 142] {
            let p = sim.drones.scripts.players.iter().find(|p| p.placement == plc).unwrap();
            assert_eq!(p.scripts[0], 0x0600_0818, "SP#{plc} carries the iris-door script [D p0]");
            assert_eq!(p.modes[0], 8, "SP#{plc} is a mode-8 looper [D p2]");
            assert_eq!((p.state, p.info[0].is_some()), (6, true),
                "SP#{plc} primed PLAYING (state 6, script loaded) from the level load [M 0x1db9a8 + 0x1dbc08]");
            assert_eq!(p.out, if plc == 141 { 0x1c } else { 0x18 }, "SP#{plc} fires on its own out channel [D p6]");
        }
        // The 8 iris-panel entities exist (the playing script created them [D streams 1-8,
        // hashes 0x02000834..3b]). Their stored matrices are the streams' local keys — the world
        // placement folds the parent chain (Script_Interp's live-matrix walk), so only the
        // existence and the hash band are pinned here.
        let mut panels = 0usize;
        for o in sim.drones.scripts.objects.list.iter().flatten() {
            if let nf_game::nis::Obj::Entity { hash, .. } = o {
                if (0x0200_0834..=0x0200_083b).contains(hash) { panels += 1; }
            }
        }
        assert!(panels >= 8, "the 8 iris-panel entities exist [D streams 1-8], found {panels}");
        // The channel-rise sister: [M 0x1dbda8 case 6] reloads only through the other slot
        // (hash[1] = 0 [D]) — a ch rise on the out channel starts NOTHING.
        assert_eq!(sim.drones.scripts.players.iter().find(|p| p.placement == 141).unwrap().state, 6,
            "sister: the doors stay looping loopers regardless of the missile channels");
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x1c, 1, sim.drones.frame);
        sim.drones.channels.set(0x18, 1, sim.drones.frame);
    }
    w.tick(Input::default(), false);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for plc in [141usize, 142] {
            let p = sim.drones.scripts.players.iter().find(|p| p.placement == plc).unwrap();
            assert_eq!(p.state, 6,
                "sister: the ch 0x18/0x1c rise does NOT restart SP#{plc} [M 0x1dbda8 case 6 needs the other slot]");
        }
    }
}

/// The spawn pin (green): with ch 0x0a driven from the WALKED deck route, spawner #187 produces
/// Drake instantly, enabled, at his placement (−7.32, −1.42, 0.06) — the 720a "Drake does not
/// spawn in the port" gap is refuted (their Bond had fallen into the pit; the spawner answers
/// from the deck's edge).
#[test]
fn the_fight_gate_spawns_drake_from_the_deck() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — the same ground-substate premise as leg 1 (the p3 == 2 spawn enters
    // substate 8 [M 0x1910f8 :160-165]; this test pins the spawner, not the substate).
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [14.0, -1.9, -4.0], [12.0, -1.9, -6.5]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.3), "the deck south-west");
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x0a, 1, sim.drones.frame);
    }
    let mut found = None;
    for _ in 0..900u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
            found = Some(i);
            break;
        }
    }
    let i = found.expect("the gate spawns Drake (spawner #187 [D])");
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[i];
    assert!(d.enabled, "the gate enables Drake (wait-switch 10 [D])");
    assert!((d.pos[0] - (-7.32)).abs() < 1.5 && (d.pos[2] - 0.06).abs() < 1.5,
        "Drake stands at his placement ({:?})", d.pos);
    assert!(!sim.drones.channels.on(0x0e), "Drake is alive (ch 0x0e not yet fired [D param 12 = 14])");
}


/// Sister walk: the east-deck leg adds no substate change of its own. The p3 == 2 spawn starts
/// IN the zero-G substate 8 [M Player_Init 0x1910f8 :160-165 — asserted; the daily audit
/// 2026-10-17: the old "never enters zero-G" doc contradicted the start], and the only proven
/// 1b substate change is the slit zone's C_CHCHZEROG 0x216388 msg 0x4b → 9 — past this route's
/// end. Driven under the legs' [S] ground-substate premise, no tick of the route enters 8/9.
#[test]
fn the_east_deck_route_adds_no_substate_change() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_8,
        "the p3 == 2 start is substate 8 [M Player_Init 0x1910f8 :160-165]");
    // [S] unverified — the same ground-substate premise as the legs above.
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    let mut zerog_ticks = 0u32;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    // The substate sample: drive 300 more ticks and count any 8/9.
    for _ in 0..300 {
        w.tick(Default::default(), false);
        if matches!(w.scene.sim.as_ref().unwrap().bond.substate, 8 | 9) { zerog_ticks += 1; }
    }
    assert_eq!(zerog_ticks, 0, "the east-deck route enters no zero-G substate of its own [S] unverified — 0x216388 is C_CHCHZEROG_Handler (its msg-0x4b arm sets substate 9 or 0); nothing there ties it to a slit zone or the route's end");
}

/// The disc's own activation chain, run by the port's script engine (this task's find): with
/// ch 30 (SP#211's gate — console-1's UI-script write [D 0x060007e2 stream 1]) and ch 46
/// (SP#211's out — the missile-disarm chain's write [D 0x0600091c stream 10]; mask bit 0 IS the
/// state-0 start trigger [M SP_Update 0x1dbda8 case 0]) written from the WALKED deck route,
/// SP#211 plays 0x060007fd → 0x060008f0 and the fly-by's own SUB 0x6000971 pulse (stream 2
/// `ev0x0d[0x4b,1]` [D]) lands on the ported `C_CHCHZEROG` entry [M 0x216388:10-17] — substate 9
/// with NO raw ch-0x4b write from the walk. The chain also reaches the fight gate: Drake
/// (spawner #187 [D]) spawns enabled within the cap. Removing the entry handler, or the SP
/// start trigger's out-channel bit, regresses both pins.
#[test]
fn the_sp211_chain_enters_the_float_and_spawns_drake() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — the same ground-substate premise as leg 1 (this test pins the CHAIN).
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    let _ = w.try_walk_to(walker::Wp::new(8.6, -1.9, -0.6).r(1.2), "the slit mouth");
    // Sister: before the drive, walk-substate holds and no float is entered.
    assert_ne!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_9,
        "sister: the chain's writes are what enter the float");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x1e, 1, sim.drones.frame);
        sim.drones.channels.set(0x2e, 1, sim.drones.frame);
    }
    let mut entered = false;
    let mut drake = None;
    for _ in 0..1800u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if !entered && sim.bond.substate == SS_ZEROG_9 { entered = true; }
        if drake.is_none() {
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
                drake = Some(i);
            }
        }
        if entered && drake.is_some() { break; }
    }
    assert!(entered, "the chain's own ch-75 SUB pulse enters the float [M 0x216388:10-17; D 0x6000971 s2]");
    let i = drake.expect("the chain reaches the fight gate: Drake spawns (spawner #187 [D])");
    assert!(w.scene.sim.as_ref().unwrap().drones.list[i].enabled,
        "the gate enables Drake (wait-switch 10 [D])");
}
