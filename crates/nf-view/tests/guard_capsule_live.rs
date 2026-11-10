//! guard-standing-capsule repro: a guard in his fire animation must present the game's standing
//! body for Bond to collide with. The game's bot capsule (`NDrone2_Collision` 0x140e90:91-121:
//! top = pos + up·r, bottom = pos + up·(r − hip), r 0.4 / 0.55 at kind 0xc) is ON within 2.0 of
//! the player (`DroneFunc_NearPlayerCollision` 0x14b190) in EVERY live state (`NDrone2_DoCollision`
//! 0x149ec8 checks only the dead bit 0x200 and the anim-info OFF bit — not the state), and
//! `Collide_Jointy` 0x1ec180:229-281 resolves the player's kind-0x800 query against it on every
//! contact tick — grounded included. The port pushed a GROUNDED Bond off door guards only.
//!
//! The pin: 07000002's geisha-1 captor group (enabled by the ch-0x17 plate [D tr[003] out 0x17])
//! stands its post and fires (the combat states set +0x4f8|0x20 [M DroneFunc_SetAsAttacking
//! 0x14aed8]); Bond walks the real route to the room and then straight INTO the nearest live
//! captor — real steering input every tick, no teleport after the spawn. On the game the two
//! capsule spines keep ~(0.55 + 0.4) apart; through the pre-fix port Bond walked to < 0.4 of the
//! guard's origin (no standing body).
//!
//! Opt-in via NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp, wrap_pi};
use nf_game::player::Input;

fn ch_on(w: &Walker, c: u8) -> bool { w.scene.sim.as_ref().unwrap().drones.channels.on(c as u32) }

/// The walkreal real-health fight helper: kill every armed guard in radius through the real fire
/// path (the fights keep the legs from wedging under return fire).
fn clear_pursuers(w: &mut Walker, radius: f32, label: &str) {
    for _ in 0..8 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().enumerate()
                .filter(|(_, d)| d.team == 1 && d.health > 0.0 && !d.corpse && d.enabled
                    && (d.pos[0] - sim.bond.pos[0]).powi(2) + (d.pos[2] - sim.bond.pos[2]).powi(2) < radius * radius)
                .map(|(i, _)| i).next()
        };
        let Some(di) = target else { return };
        if !w.shoot(di, label) { eprintln!("[gc] {label}: drone {di} survived the clip"); }
    }
}

#[test]
fn a_firing_guard_stands_between_bond_and_his_post() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&iso, "07000002.bin", false).expect("load 07000002"));
    w.lenient = true;

    // The level-start cut-scene plays out (real, not skipped).
    w.skip_cutscenes = false;
    for _ in 0..4200 {
        if !w.scene.sim.as_ref().unwrap().disabled { break; }
        w.tick(Default::default(), false);
    }
    w.skip_cutscenes = true;

    // The proven real route: spawn room → ladder #114 → the garden → the west wing → the
    // ch-0x17 plate crossed from its north side (the real-health walkreal's legs).
    for (name, at) in [
        ("bunker room south", [-27.2f32, 1.05, 31.5]),
        ("bunker corridor", [-27.2, 1.05, 36.0]),
        ("corridor south", [-27.2, 1.05, 39.5]),
        ("alcove mouth", [-27.2, 1.05, 41.0]),
    ] {
        w.walk_to(Wp { at, act: Act::None, arrive: 1.0 }, name);
    }
    w.walk_to(Wp { at: [-26.87, 2.54, 42.23], act: Act::Ladder { top_y: 6.0 }, arrive: 0.9 }, "ladder #114");
    w.walk_to(Wp { at: [-26.0, 5.9, 43.4], act: Act::None, arrive: 1.4 }, "the garden");
    w.walk_to(Wp { at: [-24.6, 5.9, 42.8], act: Act::None, arrive: 1.4 }, "the garden east");
    clear_pursuers(&mut w, 18.0, "garden");
    clear_pursuers(&mut w, 18.0, "garden second pass");
    for (name, at) in [
        ("garden path", [-22.2f32, 5.9, 34.1]),
        ("south hall", [-23.3, 5.9, 27.6]),
        ("south hall west", [-31.2, 5.9, 22.1]),
        ("north of the ch-0x17 plate", [-35.2, 6.5, 8.2]),
    ] {
        w.walk_nav_to(at, name, 1.6);
        let r = if name == "south hall west" { 24.0 } else { 14.0 };
        clear_pursuers(&mut w, r, name);
        if name == "south hall west" { clear_pursuers(&mut w, 24.0, "south hall west second pass"); }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.level_exit.is_some() {
                eprintln!("[gc] level_exit after {name}: mission state {} internal {} fail_label {:#x} health {:.1}",
                    sim.drones.mission.state, sim.drones.mission.internal, sim.drones.mission.fail_label, sim.drones.health.health);
            }
        }
    }
    w.lenient = true;
    w.walk_to(Wp { at: [-35.2, 6.5, 4.6], act: Act::Wait(10), arrive: 0.9 }, "the ch-0x17 plate");
    w.lenient = false;
    if !ch_on(&w, 0x17) {
        // The walkreal's own fallback (the west-wing slope wedges the direct leg; the TouchOnce
        // machinery itself is proven on this level by #341): drive the raise, keep every Bond
        // position real.
        eprintln!("[gc] the ch-0x17 plate raise driven (the walker can't cross its quad)");
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x17, 1, sim.drones.frame);
        w.wait(30);
    }
    assert!(ch_on(&w, 0x17), "the geisha group is enabled [D tr[003] out 0x17]");

    // The nearest live captor of the enabled group (the geisha-1 room, around the geisha).
    let sim = w.scene.sim.as_ref().unwrap();
    if sim.drones.level_exit.is_some() {
        eprintln!("[gc] level_exit before the bump: mission state {} internal {} fail_label {:#x}",
            sim.drones.mission.state, sim.drones.mission.internal, sim.drones.mission.fail_label);
    }
    let g1 = sim.drones.list.iter().position(|d| d.second_channel == 0x49).expect("geisha 1");
    let around = sim.drones.list[g1].pos;
    let target = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.team == 1 && d.health > 0.0 && !d.corpse && d.enabled
                && d.pos.iter().zip(around).map(|(a, b)| (a - b).powi(2)).sum::<f32>() < 64.0)
            .min_by(|a, b| {
                let da = a.1.pos.iter().zip(around).map(|(x, y)| (x - y).powi(2)).sum::<f32>();
                let db = b.1.pos.iter().zip(around).map(|(x, y)| (x - y).powi(2)).sum::<f32>();
                da.total_cmp(&db)
            })
            .map(|(i, _)| i)
            .expect("the enabled geisha-1 group has a live captor")
    };
    eprintln!("[gc] bump target = drone {target} at {:?}", w.scene.sim.as_ref().unwrap().drones.list[target].pos);

    // The real nav route into the captor's room (the walkreal's approach), then the close steer.
    w.walk_nav_to(around, "the geisha-1 room approach", 3.0);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[gc] approached: bond {:?} captor {:?}", sim.bond.pos, sim.drones.list[target].pos);
    }

    // The bump leg: steer Bond at the captor's LIVE position every tick (real movement input —
    // the game's own walk; no pos writes, no teleports), 480 ticks, tracking the closest XZ
    // approach while the captor lives.
    let mut min_d = f32::MAX;
    for t in 0..480 {
        let (tp, alive) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[target];
            (d.pos, d.health > 0.0 && !d.corpse)
        };
        if !alive { eprintln!("[gc] the target died at tick {t}"); break; }
        let (pos, yaw) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.pos, sim.bond.yaw) };
        let dx = tp[0] - pos[0];
        let dz = tp[2] - pos[2];
        let dist = (dx * dx + dz * dz).sqrt();
        min_d = min_d.min(dist);
        if t % 60 == 0 {
            let st = w.scene.sim.as_ref().unwrap().drones.list[target].state;
            eprintln!("[gc] tick {t}: dist {dist:.3} guard state {st:#x} bond ({:.2},{:.2},{:.2})", pos[0], pos[1], pos[2]);
        }
        if dist > 0.6 {
            let desired = dx.atan2(dz);
            let err = wrap_pi(desired - yaw);
            let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 0.2 { 0.3 } else { 1.0 }, ..Default::default() };
            w.tick(input, false);
        } else {
            // At the body: hold position (the game's body holds Bond here); the tick still runs.
            w.tick(Input::default(), false);
        }
    }
    eprintln!("[gc] min XZ distance reached: {min_d:.3}");
    // The game's two spines (Bond 0.55, the guard's standing capsule 0.4 [M 0x140e90:91-121])
    // keep Bond's spine ~0.95 from the guard's; slack for the guard's own repositioning.
    assert!(min_d >= 0.8, "Bond walked into the firing guard's body: min XZ distance {min_d:.3} — the game's standing capsule [M 0x140e90:91-121] holds ~0.95");
}

/// The kind-0xc body (capsule-join): `NDrone2_Collision` 0x140e90:91-94 widens the standing
/// capsule to r 0.55 when the voice set +0xD8 is 0xc (`Drone::kind`; on the disc only
/// 07000008's skin 0x500003b, Zoe, carries it [D census over nf_view::dev::LEVELS]); every other
/// standing bot is r 0.4, top = pos + r, bottom = pos + r − hip [M :91-121]. Zoe's start seat is
/// boxed in by the gondola geometry on every side (a walk-in stops at 1.34-2.5 on the world, not
/// on her), so the shape is pinned on the body the Sim hands the player's push.
#[test]
fn zoe_presents_the_wider_kind_0xc_body() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
    for _ in 0..300 { w.tick(Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let zoe = sim.drones.list.iter().find(|d| d.kind == 0xc).expect("the kind-0xc ally (Zoe) is placed");
    assert_eq!(zoe.skin, 0x0500_003b, "the kind-0xc voice set is Zoe's skin [D]");
    let (top, bottom, r) = nf_view::scene::bot_player_body(zoe);
    assert_eq!(r, 0.55, "kind 0xc → r 0.55 [M 0x140e90:91-94]");
    assert!((top[1] - (zoe.pos[1] + 0.55)).abs() < 1e-5 && (bottom[1] - (zoe.pos[1] + 0.55 - zoe.hip_anim)).abs() < 1e-5,
        "top = pos + r, bottom = pos + r − hip [M 0x140e90:102-166]: {top:?} {bottom:?} pos {:?} hip {}", zoe.pos, zoe.hip_anim);
    let guard = sim.drones.list.iter().find(|d| d.kind != 0xc && !d.down && d.team == 1).expect("an 08 guard");
    let (gt, _, gr) = nf_view::scene::bot_player_body(guard);
    assert_eq!(gr, 0.4, "every other standing bot → r 0.4 [M 0x140e90:91]");
    assert!((gt[1] - (guard.pos[1] + 0.4)).abs() < 1e-5);
}
