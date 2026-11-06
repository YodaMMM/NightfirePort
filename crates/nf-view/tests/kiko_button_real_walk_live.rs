//! kiko-button-real-walk: the KikoDeathButton #693 reached ON FOOT from the chaser's spawn — the
//! real walk (`Sim::step_bond` + `Input` only, NO `bond.pos` writes anywhere in the main walk),
//! the E press through the game's own dispatch, and the gate answer.
//!
//! What the walk proves (new evidence, this task):
//! - **the tower room is walkable**: from the silo plate the west lane (x 5.2..6.3) passes the
//!   tower-base wall — the diagonal collision ramp the fix-finale probe read as a "slab leak" at
//!   (7.41, −153.78) [D collision: the southward/eastward ray scans; the wall's blocked cells
//!   (6.5,−152.5)→(7,−153.5)→(7.5,−154.5)→(8,−155.5) shift east as z drops]. Bond covers the last
//!   metres on his own input.
//! - **the bridge fall into the pit is the game's own transition, not a leak**: the bridge deck
//!   holds to z ≈ −174 and the ch-0x81 slab's stretch (z −178.50..−174.32, y 0..2.07 [D trigger
//!   bbox]) is open — Bond drops through its airspace, the touch fires, and the pack cinema's
//!   stream-8 key re-enables him on the pit floor [D pack_cinema_16_live; M Player_Enable
//!   0x195498].
//! - **the gate ch 0x8d arms on the unavoidable corridor crossing and the disc CLEARS it in the
//!   07900016 pack**: touch #616 is a flags-0xEA TouchOnce (latching [M the class-dispatcher
//!   update + 0x1b1900's disassembly — Trigger_Calc updates type-2 only, the dispatcher all
//!   classes]), mask 8 (the player), volume x 0.36..4.54, y −0.20..1.88, z −146.65..−144.58 [D] =
//!   the full corridor width at the only route south. CORRECTED (this task): the older header
//!   claimed nothing on the disc clears the lock — its census decoded 07000016.bin + 07800016.bin
//!   and missed the 07900016.bin pack. The clear is the lift-out pack cinema **0x06000632**
//!   (07900016.bin [D]): its stream 16 writes ch 141 = 0x8d = 0 at **t1056**, and its stream 13's
//!   event 6 at **t1057** hands Bond back to stream 12's dummy at key 1868 = (5.39, 1.34,
//!   −158.31) — the shuttle-bay floor beside the button [D keys; M Script_EventHandler 0x1d94c0
//!   event 6 → Script_Free → Player_Enable 0x195498]. The cinema runs only after the group-7 pit
//!   squad dies and the launch script ends (mux #633 → ch 0x82 [D]). So the walked press is
//!   inhibited exactly until the lift-out cinema plays — the test now walks the whole chain
//!   (shelter pulses, the chaser (the test's own grenade, as before), the pit squad on Bond's
//!   fire — no pos writes) and pins the walked press COMPLETING after the clear: E →
//!   Player_Activate 0x1a5170's class-0 arm →
//!   SP_Activate 0x1dc338 (the facing dot) → Switch_Activate 0x1b02a8 toggles ch 0x90 (the gate
//!   now off [M 0x1b02a8:22]) → mux #618 → ch 100 → MissionState 6 → 0x7300017
//!   [M 0x182c60:111-112]. Sister: the same E from the documented gate-off reach (a fresh load
//!   teleported to the button room, #616 never crossed — the interactables shape) fires the same
//!   chain, pinning that the walked completion is the cinema's clear at work, not a port slack.
//!
//! Open live check (the one link machine evidence cannot close from here): whether
//! the GAME's contact pass registers #616 for the walking player at all — its Trigger_Hit is
//! contact-list-based on the trigger's glist bounds, the port's on the target object's collision
//! tris. Fresh-boot PCSX2, walk z −144..−147 in the mid-corridor, watch `switch_channels[141]`
//! (0x26fc90+141 = 0x26fd1d): 0→1 then a t1056-style 1→0 at the lift-out confirms the arm AND
//! the clear on the shipped game; a steady 0 means the game's #616 bounds differ from the port's
//! tris and the walked press would complete without the cinema — see the report.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, RamSave};
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

fn grenade_at(sim: &mut nf_view::scene::Sim, pos: [f32; 3]) {
    sim.drones.grenades.push(nf_game::grenade::Grenade {
        pos, velocity: [0.0, 0.0, 0.0], speed: 0.0,
        // One tick from its fuse (travelled 59 of range 60): a grenade explodes only on its fuse,
        // never on contact (fix-weapons-all, `Bullet_CollisionHandler` 0x12ce80).
        travelled: 59.0, range: 60.0, damage: 100.0, radius: 3.0, weapon_type: 0x34,
        mesh: 0x0200_0726, owner: usize::MAX, state: 0, bounces: 0, smoke_popped: false, resting: false,
        stuck: false, seek_target: None,
        phase: None,
    burn: 0, // [M Bullet_init 0x12c5a0:211 — the 0x3b family carries 100]
    defused: false,
    });
}

/// Ticks until `done`, or panics with `what` after `max` (the walkreal shape).
fn tick_until(w: &mut Walker, max: u32, what: &str, done: impl Fn(&nf_view::scene::Sim) -> bool) -> u32 {
    for t in 0..max {
        if done(w.scene.sim.as_ref().unwrap()) { return t; }
        w.tick(Input::default(), false);
    }
    panic!("{what}: not within {max} ticks");
}

/// Sits one ch-243 pulse out where Bond stands, with the walker's damage allowance OFF (no
/// top-up can hide a contact): every tick of the pulse Bond's own hurt capsule must miss
/// HurtPoly-ShuttleLaunch, and he ends it alive with the mission not failed (the walkreal shape).
fn sit_out_pulse(w: &mut Walker, label: &str) -> u32 {
    let allowance = w.top_up_health;
    // The allowance holds until the pulse rises (the guards' fire before it is not the volume's
    // contact), then is OFF for every tick of the pulse (walkreal_07000016_live's shape).
    tick_until(w, 8000, label, |s| s.drones.channels.on(243));
    w.top_up_health = false;
    let mut n = 0u32;
    while w.scene.sim.as_ref().unwrap().drones.channels.on(243) {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let h = sim.hurts.iter().find(|h| h.gate == 0xf3).expect("HurtPoly-ShuttleLaunch");
            assert!(!h.contains(p, [p[0], p[1] + 1.8, p[2]], 0.55), "{label}: the volume touches Bond at {p:?} (+{n})");
            assert!(sim.drones.health.health > 0.0, "{label}: Bond died at {p:?} (+{n})");
        }
        w.tick(Input::default(), false);
        n += 1;
        assert!(n < 1000, "{label}: the pulse never fell");
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.health.health > 0.0 && sim.drones.mission.status() != 3, "{label}: Bond lives, no fail");
    w.top_up_health = allowance;
    n
}

/// E beats aimed at `at` until `done` (the interactables press shape: the game's Player_Activate
/// dispatch, one edge per 6 ticks).
fn press_use_until(w: &mut Walker, at: [f32; 3], label: &str, max: u32, done: impl Fn(&nf_view::scene::Sim) -> bool) -> bool {
    for k in 0..max {
        if done(w.scene.sim.as_ref().unwrap()) { return true; }
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (at[0] - sim.bond.pos[0]).atan2(at[2] - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        let to = [at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]];
        let want_pitch = (to[1] / to[0].hypot(to[2]).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let input = Input {
            turn: err.clamp(-0.08, 0.08),
            look: (want_pitch - sim.bond.pitch).clamp(-0.06, 0.06),
            ..Default::default()
        };
        w.tick(input, k % 6 == 0);
    }
    eprintln!("[kiko-walk] {label}: the E never landed in {max} ticks");
    false
}

/// From the pit floor into the west blast room (blast_room_shelter_16_live: the room is outside
/// HurtPoly-ShuttleLaunch's polygon [D], its door leaves shut over both pulses [D 0600090a
/// streams 17/18]) — the walkreal route.
const INTO_THE_SHELTER: &[(&str, walker::Wp)] = &[
    ("the pit floor west", walker::Wp { at: [-8.5, -22.2, -173.5], act: walker::Act::None, arrive: 1.0 }),
    ("the west blast door", walker::Wp { at: [-11.0, -22.0, -170.0], act: walker::Act::None, arrive: 0.9 }),
    ("the west blast room", walker::Wp { at: [-15.2, -22.0, -170.4], act: walker::Act::None, arrive: 0.8 }),
];

/// The real walk from the chaser's spawn to the KikoDeathButton — through the whole disc chain
/// (the shelter pulses, the chaser, the pit squad, the lift-out cinema) — and the gate answer.
#[test]
fn the_07000016_real_walk_reaches_the_kiko_death_button_and_the_cinema_clears_the_gate() {
    let iso_path = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso_path, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(200);

    // The walkreal route (walkreal_07000016_live::walk_arrival_to_the_chasers_spawn, proven):
    // the corridor, the station detour, the descent, the doors, the silo plate, the south room.
    let route: &[(&str, walker::Wp)] = &[
        ("the corridor south", walker::Wp::new(4.3, 0.6, -72.5).r(1.2)),
        ("door[7] through", walker::Wp::new(4.9, -0.1, -78.0).r(1.2)),
        ("the lane east", walker::Wp::new(10.0, 0.0, -82.0).r(1.3)),
        ("the lane south", walker::Wp::new(12.0, 0.0, -88.0).r(1.3)),
        ("the ramp foot", walker::Wp::new(13.8, 0.0, -90.8).r(1.1)),
        ("the ramp 1", walker::Wp::new(14.5, 0.6, -87.5).r(1.2)),
        ("the ramp 2", walker::Wp::new(14.5, 2.4, -84.5).r(1.2)),
        ("the ramp 3", walker::Wp::new(14.5, 4.6, -81.5).r(1.2)),
        ("the ring east", walker::Wp::new(14.5, 5.9, -79.0).r(1.2)),
        ("the ring 1", walker::Wp::new(6.0, 5.9, -78.5).r(1.4)),
        ("the ring 2", walker::Wp::new(-4.0, 5.9, -79.0).r(1.4)),
        ("the ring 3", walker::Wp::new(-11.0, 5.9, -81.0).r(1.4)),
        ("the ring 4", walker::Wp::new(-13.0, 5.9, -86.0).r(1.4)),
        ("the station west", walker::Wp::new(-16.5, 5.9, -88.0).r(1.3)),
        ("the station south", walker::Wp::new(-16.3, 5.9, -93.5).r(1.2)),
        ("the station E", walker::Wp::at(-15.2, 5.9, -94.6, walker::Act::Use { use_ticks: 120 }).r(1.2)),
        ("the return 1", walker::Wp::new(-16.3, 5.9, -89.0).r(1.3)),
        ("the return 2", walker::Wp::new(-13.0, 5.9, -85.0).r(1.4)),
        ("the return 3", walker::Wp::new(-10.0, 5.9, -81.0).r(1.4)),
        ("the return 4", walker::Wp::new(-4.0, 5.9, -79.0).r(1.4)),
        ("the return 5", walker::Wp::new(6.0, 5.9, -78.5).r(1.4)),
        ("the return 6", walker::Wp::new(14.5, 5.9, -79.0).r(1.2)),
        ("the descent 1", walker::Wp::new(14.5, 4.6, -82.0).r(1.2)),
        ("the descent 2", walker::Wp::new(14.5, 2.4, -85.0).r(1.2)),
        ("the descent 3", walker::Wp::new(14.5, 0.6, -88.0).r(1.2)),
        ("the east lane south", walker::Wp::new(9.5, 0.0, -100.0).r(1.6)),
        ("the trench's west end", walker::Wp::new(4.0, 0.0, -108.0).r(1.6)),
        ("door[0]'s west approach", walker::Wp::new(2.3, 0.0, -110.0).r(0.7)),
        ("door[0]'s threshold", walker::Wp::new(2.3, 0.0, -113.2).r(0.7)),
        ("door[0] through", walker::Wp::new(2.0, -0.1, -119.0).r(1.2)),
        ("door[1] through", walker::Wp::new(2.0, -0.1, -132.0).r(1.2)),
        ("door[3]'s panel", walker::Wp::at(3.4, 0.8, -141.5, walker::Act::Use { use_ticks: 90 }).r(0.8)),
        ("door[3] off panel", walker::Wp::new(2.0, 0.0, -140.0).r(1.0)),
        ("door[3] through", walker::Wp::new(2.0, -0.1, -147.0).r(1.2)),
        ("door[2]'s panel", walker::Wp::at(3.4, 0.8, -151.9, walker::Act::Use { use_ticks: 90 }).r(0.8)),
        ("door[2] off panel", walker::Wp::new(2.2, 0.0, -150.4).r(1.0)),
        ("door[2] through", walker::Wp::new(2.2, -0.1, -151.5).r(1.2)),
        ("the silo plate", walker::Wp::new(2.5, 0.3, -153.9).r(1.0)),
        ("door[4]'s panel", walker::Wp::at(3.5, 1.0, -165.7, walker::Act::Use { use_ticks: 90 }).r(0.8)),
        ("door[4] through", walker::Wp::new(2.5, 1.0, -169.0).r(1.3)),
        ("the south room", walker::Wp::new(2.4, 1.0, -175.0).r(1.2)),
    ];
    w.walk_route(route);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x8f), "the silo plate fired [D #600]");
        assert!(sim.drones.channels.on(0x81), "the ch-0x81 slab fired on the bridge fall [D #602]");
        // THE GATE: the mid-corridor crossing latched ch 0x8d [D #616 flags-0xEA TouchOnce, mask 8;
        // M the dispatcher update + 0x1b1900 disasm]. It is ON until the lift-out cinema clears it.
        assert!(sim.drones.channels.on(0x8d),
            "the mid-corridor touch #616 latched the gate on the walked route [D; M 0x1b02a8:22]");
    }

    // The launch chain plays out (no skip, no pos writes): the pack cinema 0x06000631 hands Bond
    // back on the pit floor [D pack_cinema_16_live stream 8; M Player_Enable 0x195498] — the
    // upper-floor re-seat this test used before is GONE: the survival leg is the blast-room
    // shelter walked on Input (blast_room_shelter_16_live; the full walk is walkreal_07000016).
    w.skip_cutscenes = false;
    tick_until(&mut w, 3000, "the pack cinema's hand-back on the pit floor",
        |s| s.drones.channels.on(0x81) && !s.disabled && s.bond.pos[1] < -20.0);
    w.skip_cutscenes = true;
    let t0 = w.total_ticks;
    w.walk_route(INTO_THE_SHELTER);
    let inside_at = w.total_ticks - t0;
    assert!(inside_at < 1000, "Bond is in the room before its door starts to shut (+{inside_at}, the leaves move 1000..1350 [D])");
    let n1 = sit_out_pulse(&mut w, "pulse 1 in the shelter");
    assert!((550..=566).contains(&n1), "pulse 1 is 1611..2170 = 559 ticks [D stream 6], got {n1}");

    // The chaser: ch 240 @2180 wakes spawner #604 (group 6) [D]; Bond shoots her from the room.
    tick_until(&mut w, 2000, "the chaser spawns",
        |s| s.drones.list.iter().any(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && !matches!(d.death_channel, 23..=26)));
    let chaser = w.scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && !matches!(d.death_channel, 23..=26))
        .expect("the chaser is present");
    while w.total_ticks - t0 < 2600 { w.tick(Input::default(), false); }
    // The chaser's defeat through the game's explosive path (this test's own kill shape before
    // this task): a grenade explodes only on its fuse, never on contact [M Bullet_CollisionHandler
    // 0x12ce80], so the drop goes off where she stands — just after her spawn, in the open pit
    // and well outside the room. (Bond's fire needs the walkreal's ~1100-tick duel and missed the
    // 6310 door deadline here; the throw lands at +~1220 instead.)
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let before: Vec<bool> = (0..256).map(|c| sim.drones.channels.on(c)).collect();
        let mut died = false;
        'outer: for _ in 0..3 {
            let p = sim.drones.list[chaser].pos;
            eprintln!("[kiko-dbg] throw at {:?} gate_before={}", p, sim.drones.channels.on(0x8d));
            grenade_at(sim, p);
            for _ in 0..300u32 {
                sim.step_bond(Input::default(), false, false, false);
                let now8d = sim.drones.channels.on(0x8d);
                let now90 = sim.drones.channels.on(0x90);
                if now8d != before[0x8d as usize] || now90 != before[0x90 as usize] {
                    eprintln!("[kiko-dbg] GATE FLIPPED at tick {died} corpse={}", sim.drones.list[chaser].corpse);
                    for c in 0..256u32 {
                        if sim.drones.channels.on(c) != before[c as usize] { eprintln!("[kiko-dbg]   ch {c} {:#x} {} -> {}", c, before[c as usize] as u8, sim.drones.channels.on(c) as u8); }
                    }
                    for (i, d) in sim.drones.list.iter().enumerate() {
                        if d.corpse && d.present { eprintln!("[kiko-dbg]   corpse #{i} death_ch {} at {:?}", d.death_channel, d.pos); }
                    }
                    break 'outer;
                }
                if sim.drones.list[chaser].corpse { died = true; break; }
            }
            if died { break; }
        }
        assert!(died, "the chaser dies at the silo");
        let mut flipped = false;
        for t in 0..100u32 {
            w.tick(Input::default(), false);
            let sim = w.scene.sim.as_ref().unwrap();
            let now90 = sim.drones.channels.on(0x90);
            if now90 != before[0x90 as usize] && !flipped {
                flipped = true;
                eprintln!("[kiko-dbg] 0x90 ROSE at settle +{t}");
                for c in 0..256u32 {
                    if sim.drones.channels.on(c) != before[c as usize] { eprintln!("[kiko-dbg]   ch {c} {:#x} {} -> {}", c, before[c as usize] as u8, sim.drones.channels.on(c) as u8); }
                }
                for (i, d) in sim.drones.list.iter().enumerate() {
                    if d.corpse && d.present { eprintln!("[kiko-dbg]   corpse #{i} death_ch {} at {:?}", d.death_channel, d.pos); }
                }
            }
        }
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let mut died = false;
        for _ in 0..3 {
            let p = sim.drones.list[chaser].pos;
            grenade_at(sim, p);
            for _ in 0..300u32 {
                sim.step_bond(Input::default(), false, false, false);
                if sim.drones.list[chaser].corpse { died = true; break; }
            }
            if died { break; }
        }
        assert!(died, "the chaser dies at the silo");
    }
    for _ in 0..100 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.list[chaser].corpse, "the chaser is down");
        assert!(sim.drones.channels.on(0x8d), "the gate stays armed into the fight (the clear has not run yet) [D]");
        assert!(!sim.drones.channels.on(0x90), "sister: the chaser's death raises nothing (param 12 = 0 [D #261]); ch 0x90 is cold before the press [D]");
        assert!(sim.drones.mission.status() != 6, "sister: the mission does not complete on the kill [D AND #271 waits for the button]");
    }

    // Back into the shelter before the leaves shut again (6310..6660 [D]); pulse 2 out.
    w.walk_route(INTO_THE_SHELTER);
    assert!(w.total_ticks - t0 < 6310, "back in the room before its door shuts for pulse 2 (+{})", w.total_ticks - t0);
    let n2 = sit_out_pulse(&mut w, "pulse 2 in the shelter");
    assert!((500..=520).contains(&n2), "pulse 2 is 6810..7320 = 510 ticks [D stream 9], got {n2}");

    // The pit squad (group 7, ch 242 @7332): all four must fall for ch 0xbf [D mux #611].
    tick_until(&mut w, 600, "the pit squad spawns", |s| s.drones.list.iter().any(|d| d.present && !d.corpse && matches!(d.death_channel, 23..=26)));
    for _ in 0..4 {
        let next = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            sim.drones.list.iter().enumerate()
                .filter(|(_, d)| d.present && !d.corpse && d.health > 0.0 && matches!(d.death_channel, 23..=26))
                .min_by(|a, b| walker::dxz(a.1.pos, bp).total_cmp(&walker::dxz(b.1.pos, bp)))
                .map(|(i, _)| i)
        };
        let Some(i) = next else { break };
        // The fight cannot ride on a dropped gun landing at Bond's feet: with the guard-side fire
        // glass pass [M Collide_Update 0x1ea398:216-229 — the port's law since guard-batch], the
        // phase's drop pickup is not guaranteed (the green master run rode one member dying
        // ADJACENT to the shelter door, and the return-2 leg's stall alone shifts the phase's
        // entry state), and the held fists' 1.8-u hit bullet never reaches a member standing 4-6 u
        // off [L weapon_data range 1.8 — the guardroom-guard invulnerability, level-06-guardroom-
        // guard; the measured 10800-tick budget expiry with clip 1 spare 0 untouched]. Cycle the
        // wheel to an owned firearm (the real player's move, Input only) before every target.
        assert!(w.switch_to_armed("the pit squad"), "the pit squad: an owned firearm is on the wheel");
        assert!(w.shoot(i, "the pit squad"), "the pit squad member #{i} dies to Bond's fire");
    }
    tick_until(&mut w, 600, "the pit squad's deaths raise ch 0xbf [D mux #611]", |s| s.drones.channels.on(0xbf));

    // THE LIFT-OUT: ch 0xc0 @7808 (stream 20) → mux #633 → ch 0x82 → Script Player #610 → pack
    // cinema 0x06000632 (07900016.bin) [D]. No skip: it disables Bond at t0, stream 16 clears
    // ch 141 = 0x8d at t1056, and stream 13's event 6 at t1057 hands Bond back to stream 12's
    // dummy at key 1868 — beside the button [D; M 0x1d94c0 event 6 → Script_Free → Player_Enable].
    w.skip_cutscenes = false;
    tick_until(&mut w, 3000, "ch 0x82 starts the lift-out cinema [D mux #633]", |s| s.drones.channels.on(0x82) && s.disabled);
    tick_until(&mut w, 1500, "the lift-out cinema hands Bond back", |s| !s.disabled);
    w.skip_cutscenes = true;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 5.39).abs() < 0.5 && (p[1] - 1.34).abs() < 1.0 && (p[2] + 158.31).abs() < 0.5,
            "0x06000632 hands Bond back at its key 1868 (5.39, 1.34, −158.31) [D], got {p:?} — the upper-floor re-seat is gone");
        assert!(!sim.drones.channels.on(0x8d),
            "0x06000632 stream 16 cleared the walked route's gate ch 0x8d at t1056 [D 07900016.bin] — the disc DOES clear the button lock");
    }

    // THE WALKED PRESS, now unblocked by the cinema's clear: the last metres on Input (the west
    // lane past the tower-base wall), then the E through the game's dispatch → ch 0x90.
    w.walk_to(walker::Wp::new(6.3, 1.0, -155.45).r(0.6), "the button's side");
    {
        let pressed = press_use_until(&mut w, [7.02, 1.46, -155.61], "the walked press", 240,
            |sim| sim.drones.channels.on(0x90));
        assert!(pressed, "the walked E on the KikoDeathButton fires ch 0x90 once 0x06000632 cleared the gate [D t1056; M Switch_Activate 0x1b02a8]");
    }
    tick_until(&mut w, 900, "the mission completes", |s| s.drones.mission.status() == 6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x90), "the KikoDeathButton raised ch 0x90 [D #693 p0]");
        assert!(sim.drones.channels.on(100), "mux #618 (out 0x64 ← 0x90) fires ch 100 [D #618]");
        assert_eq!(sim.drones.mission.to_end, 0x0730_0017, "the driving hand-off [M 0x182c60:111-112]");
    }

    // SISTER (the gate-off completion, the interactables reach): a fresh load never crosses #616,
    // the same E on the same button fires the chain — ch 0x90 → mux #618 → ch 100 → MissionState 6
    // → 0x7300017 [M 0x182c60:111-112]. Pins that the button itself (and everything downstream) is
    // live without the cinema, and the walked completion is the clear's doing alone.
    let mut w2 = Walker::new(nf_view::load_with_carry(&iso_path, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    w2.wait_enabled(3000, "the fresh arrival");
    {
        let sim = w2.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x8d), "sister: the gate is off on the teleport reach (#616 not crossed) [D]");
        let sim = w2.scene.sim.as_mut().unwrap();
        sim.bond.pos = [7.02, 1.2, -154.0];
        sim.bond.vel = [0.0; 3];
    }
    w2.wait(10);
    {
        let pressed = press_use_until(&mut w2, [7.02, 1.46, -155.61], "the gate-off press", 240,
            |sim| sim.drones.channels.on(0x90));
        assert!(pressed, "the gate-off E on the KikoDeathButton fires ch 0x90 [D #346 p0; M 0x1b02a8]");
    }
    let mut complete = false;
    for _ in 0..900u32 {
        w2.tick(Input::default(), false);
        if w2.scene.sim.as_ref().unwrap().drones.mission.status() == 6 { complete = true; break; }
    }
    {
        let sim = w2.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(100), "mux #618 (out 0x64 ← 0x90) fires ch 100 [D #618]");
        assert!(complete, "the mission completes (MissionState 6 [M 0x182c60:85-116])");
        assert_eq!(sim.drones.mission.to_end, 0x0730_0017, "the driving hand-off [M 0x182c60:111-112]");
    }
}
