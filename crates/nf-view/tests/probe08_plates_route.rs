//! PROBE (level-08-music-plate-ride): the game's route to the roofline music plates #99/#100/#101
//! (TouchOnces out 91/90/92 [D #99/#100/#101]) over x 117-131, z ≈ −101 [D volumes]. #[ignore]d:
//! a probe is a route record, not a pass/fail pin. Prints:
//!   1. the wires (class 0x2C) with their analytic grab/travel/zipline segments [M wire.rs's ports
//!      of Player_CollWire 0x1971f8 / Player_Wire 0x1976f8 / Player_Zipline 0x197ff0],
//!   2. the third icons (0xFA, `ThirdIcon_Create`) — the jump-grab contexts [M 0x1d04f0 case 0xfa],
//!   3. the plate trigger AABBs + the mux #102 decode,
//!   4. the world's floor_below answers along the roofline (z −101) and the station's east edge.
#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free route probe; run with --ignored"]
fn probe_the_roofline_route() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
    w.wait_enabled(3000, "the level start");
    let sim = w.scene.sim.as_ref().unwrap();

    println!("== wires (class 0x2C) ==");
    for (i, wire) in sim.wires.iter().enumerate() {
        let (ga, gb) = wire.grab_segment();
        let (ta, tb) = wire.travel_segment();
        let (za, zb) = wire.zipline_segment();
        println!("wire[{i}] centre {:?} radius {:.3} yaw {:.4} pitch {:.4} type {} ch {}",
            wire.centre, wire.radius, wire.yaw, wire.pitch, wire.wire_type, wire.channel);
        println!("  grab    ({:.2},{:.2},{:.2}) .. ({:.2},{:.2},{:.2})", ga[0], ga[1], ga[2], gb[0], gb[1], gb[2]);
        println!("  travel  ({:.2},{:.2},{:.2}) .. ({:.2},{:.2},{:.2})", ta[0], ta[1], ta[2], tb[0], tb[1], tb[2]);
        println!("  zipline ({:.2},{:.2},{:.2}) .. ({:.2},{:.2},{:.2})", za[0], za[1], za[2], zb[0], zb[1], zb[2]);
    }
    println!("== third icons (0xFA) ==");
    for ic in &sim.icons {
        println!("icon {:?} centre {:?} radius {:.3} ctx {} on_ch {}", ic.pos, ic.centre, ic.radius, ic.ctx, ic.on_channel);
    }
    println!("== plate triggers + mux + sound ==");
    for t in &sim.drones.triggers {
        if t.out == 90 || t.out == 91 || t.out == 92 || t.out == 0x68 {
            println!("trigger out {} kind {} lo {:?} hi {:?} centre {:?}", t.out, t.kind, t.lo, t.hi, t.centre());
        }
    }
    println!("== floor_below along the roofline (z -101.1, from y 50, window 14) ==");
    for x in [110.0f32, 112.0, 114.0, 115.5, 116.5, 117.8, 119.0, 121.0, 123.0, 125.0, 127.0, 128.5, 129.5, 130.8, 132.0, 133.4, 135.0] {
        let f = sim.world.floor_below([x, 50.0, -101.1], 14.0);
        println!("floor_below x {x:6.1} z -101.1 -> {:?}", f);
    }
    println!("== floor_below along the station's east edge (z -98.5) ==");
    for x in [130.0f32, 132.0, 133.4, 134.5, 136.0, 139.0] {
        let f = sim.world.floor_below([x, 52.0, -98.5], 14.0);
        println!("floor_below x {x:6.1} z -98.5 -> {:?}", f);
    }
    println!("== floor_below over the plate volumes (the stand check) ==");
    for (label, p) in [
        ("#100 platform", [130.78f32, 50.5, -101.22]),
        ("between #100/#99", [128.5, 50.5, -101.1]),
        ("#99", [124.98, 50.5, -101.07]),
        ("west of #99", [121.0, 50.5, -101.1]),
        ("#101", [117.81, 50.5, -101.02]),
        ("west of #101", [114.0, 50.5, -101.1]),
        ("icon #105 spot", [133.43, 50.5, -101.21]),
    ] {
        let f = sim.world.floor_below(p, 14.0);
        println!("floor_below {label} {:?} -> {:?}", p, f);
    }
}

/// PROBE 2: the real-input walk from the spawn to the boarding spot (the icon #105's sloped-roof
/// stand) and the jump-grab attempt, with the ride trace. #[ignore]d route record.
#[test]
#[ignore = "assertion-free route probe; run with --ignored"]
fn probe_the_walk_to_the_boarding_and_the_ride() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
    w.wait_enabled(3000, "the level start");
    eprintln!("[probe2] spawn pos {:?}", w.scene.sim.as_ref().unwrap().bond.pos);

    // The proven station legs (walkreal_07000008_live's route family).
    w.walk_nav_to([139.2, 49.5, -81.2], "to the intro plate", 1.3);
    eprintln!("[probe2] ch 0x49 {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0x49));
    w.wait_enabled(4500, "the station-intro scene");
    w.wait(60);
    w.walk_nav_to([139.0, 49.5, -93.0], "the station hall", 1.5);
    eprintln!("[probe2] the station hall -> pos {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    // The south shutters (#49-52 [D] at (138.35/140.72, 49.5, −96.75)): approach + E.
    w.walk_to(walker::Wp::at(139.5, 49.5, -95.9, walker::Act::Use { use_ticks: 150 }).r(1.1), "the shutters");
    eprintln!("[probe2] the shutters -> pos {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    // Raw south walk through the doorway onto the y 48.0 floor (the 1.5 step down).
    for (name, target, cap) in [
        ("through the doorway", [139.5f32, 48.0, -98.6], 400),
        ("the floor west 1", [136.5, 48.0, -98.9], 500),
        ("the floor west 2", [134.6, 48.0, -99.0], 500),
        ("the roof head", [133.6, 47.7, -100.6], 500),
        ("the icon stand", [133.4, 47.6, -101.2], 400),
    ] {
        let mut last = [0.0f32; 2];
        let mut still = 0u32;
        let mut ok = false;
        for t in 0..cap {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let dxz2 = ((p[0] - target[0]).powi(2) + (p[2] - target[2]).powi(2)).sqrt();
            if dxz2 < 0.55 { ok = true; break; }
            let use_key = if (p[0] - last[0]).abs() < 0.01 && (p[2] - last[1]).abs() < 0.01 {
                still += 1; still % 30 == 0
            } else { still = 0; last = [p[0], p[2]]; false };
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = nf_game::player::Input {
                turn: err.clamp(-0.08, 0.08),
                forward: if err.abs() > 1.0 { 0.0 } else { 1.0 },
                ..Default::default()
            };
            w.tick(input, use_key);
        }
        let s = w.scene.sim.as_ref().unwrap();
        eprintln!("[probe2] {name} -> pos {:?} ok {ok} grounded {} icon_ctx {:#x}",
            s.bond.pos, s.bond.grounded, s.bond.icon_ctx);
    }
    // The jump-grab: the game grabs instead of jumping [M Player_HandleJump 0x1916b0:118-147].
    let mut grabbed = false;
    for t in 0..240 {
        let press = t % 20 == 0;
        w.tick(nf_game::player::Input { jump: press, ..Default::default() }, false);
        let s = w.scene.sim.as_ref().unwrap();
        if s.bond.substate == nf_game::player::SS_ZIPLINE || s.bond.substate == nf_game::player::SS_WIRE {
            grabbed = true;
            eprintln!("[probe2] GRABBED at t{t} substate {:#x} wire {:?}", s.bond.substate, s.bond.wire);
            break;
        }
        if t % 40 == 0 {
            eprintln!("[probe2] jump try t{t} pos {:?} ctx {:#x} grounded {}", s.bond.pos, s.bond.icon_ctx, s.bond.grounded);
        }
    }
    eprintln!("[probe2] grabbed {grabbed}");
    if !grabbed { return; }

    // The ride trace: per-tick channels and position.
    let mut seen90 = None; let mut seen91 = None; let mut seen92 = None; let mut seen68 = None;
    for t in 0..1200u32 {
        w.tick(nf_game::player::Input::default(), false);
        let s = w.scene.sim.as_ref().unwrap();
        let (c90, c91, c92, c68) = (s.drones.channels.on(90), s.drones.channels.on(91),
            s.drones.channels.on(92), s.drones.channels.on(0x68));
        if c90 && seen90.is_none() { seen90 = Some((t, s.bond.pos)); eprintln!("[probe2] ch90 t{t} pos {:?}", s.bond.pos); }
        if c91 && seen91.is_none() { seen91 = Some((t, s.bond.pos)); eprintln!("[probe2] ch91 t{t} pos {:?}", s.bond.pos); }
        if c92 && seen92.is_none() { seen92 = Some((t, s.bond.pos)); eprintln!("[probe2] ch92 t{t} pos {:?}", s.bond.pos); }
        if c68 && seen68.is_none() { seen68 = Some(t); eprintln!("[probe2] ch0x68 t{t}"); }
        if seen68.is_some() { break; }
        if t % 60 == 0 {
            eprintln!("[probe2] ride t{t} pos {:?} sub {:#x}", s.bond.pos, s.bond.substate);
        }
    }
    let s = w.scene.sim.as_ref().unwrap();
    eprintln!("[probe2] end pos {:?} sub {:#x} 90/91/92/0x68 = {}/{}/{}/{}",
        s.bond.pos, s.bond.substate, seen90.is_some() as u8, seen91.is_some() as u8,
        seen92.is_some() as u8, seen68.is_some() as u8);
    eprintln!("[probe2] sound 0x14D playing {}", s.drones.sound_triggers.playing(0x14d));
}
