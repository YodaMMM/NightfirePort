//! comms-chatter-0c: `Drone_InitComms` 0x139520's `switch_channels[0x98]` chatter gate, pinned
//! through real input on 0700000c (the power station).
//!
//! The game's one drone scan [M 0x139520:161-201] writes `switch_channels[0x98]` EVERY frame
//! (set or clear): the chatter latch — an enabled, living guard at alertness ≥ 1 (+0x50C)
//! within 30 units of its opponent (+0x1A0), not team 2 (+0x44 != 2), the dead `& 0x600` guards
//! skipped (`NDrone2_SetAsDead` 0x14a2f0:9 → 0x200, `NDrone2_PostDroneControl` 0x14aff8:9-12 →
//! 0x400). `NDrone2_PowerstationTalk` 0x13e770:39 reads 0x98 as the idle radio chatter's only
//! gate on 0x070000c/0d — on the old tree nothing wrote it, so the chatter could never fire.
//!
//! The route is real input from the spawn: the shared 0c prologue to the yard ladder base
//! (`walk_spawn_to_the_yard_ladder_base`), the ladder climb and the patrol-ring legs copied
//! verbatim from guard1_rounds_0700000c_live, then the hold at the ring west edge where guard
//! #1 (placement #3, weapon param 0x1e, list index 1 [D docs/levels/0700000c-audit.md]) closes
//! to ~4.6 u and fights — no teleport after the load. The pin samples every tick of the fight
//! window: the channel pair (0x97 latch, 0x98 gate) against the scan law recomputed in the
//! test from the raw drone fields, then Bond kills guard #1 through the real fire path and the
//! dead-guard skip is pinned the same way (the corpse must not hold the gate up).
#[path = "common/walkreal0c.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

struct Sample { tick: u32, ch98: u8, ch97: u8, rule: bool, g_alive: bool, g_qualified: bool, alert: f32, dist: f32 }

/// The scan's chatter arm, recomputed here from the raw drone fields — the decompile's
/// disjunction of [M 0x139520:173-179] over the living list (the `& 0x600` skip folded as
/// corpse / health, [M 0x14a2f0:9 / 0x14aff8:9-12]).
fn rule(w: &Walker) -> bool {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.list.iter().filter(|d| d.present).any(|d| {
        !(d.corpse || d.health <= 0.0) && d.enabled && d.alert >= 1.0 && d.dist < 30.0 && d.team != 2
    })
}

fn sample(w: &Walker, di: usize, out: &mut Vec<Sample>) {
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[di];
    out.push(Sample {
        tick: w.total_ticks,
        ch98: sim.drones.channels.v[0x98],
        ch97: sim.drones.channels.v[0x97],
        rule: rule(w),
        g_alive: !d.corpse && d.health > 0.0,
        g_qualified: !(d.corpse || d.health <= 0.0) && d.enabled && d.alert >= 1.0 && d.dist < 30.0 && d.team != 2,
        alert: d.alert,
        dist: d.dist,
    });
}

#[test]
fn chatter_gate_0x98_tracks_the_comms_scan_through_a_real_ring_fight() {
    let iso = iso();
    // Real input from the spawn to the yard ladder base (the shared prologue; Bond never fires
    // back on it, top-up on).
    let mut w = walker::walk_spawn_to_the_yard_ladder_base(&iso);
    let di = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter()
            .find(|d| d.placement == 3 && d.weapon == 0x1e)
            .expect("placement #3 (guard #1, weapon param 0x1e) must exist at load")
            .index
    };

    // Leg 7 (REAL, verbatim from guard1_rounds): the ladder walk-in grab + climb + mantle.
    w.walk_to(Wp::new(10.3, 1.05, -186.8).r(0.7), "the ladder approach");
    let mut climbed = false;
    for _ in 0..900 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.bond.substate == nf_game::player::SS_CLIMB { climbed = true; break; }
        let desired = (9.669741f32 - sim.bond.pos[0]).atan2(-187.3427f32 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input {
            turn: if err.abs() > 0.2 { err.clamp(-0.08, 0.08) } else { 0.0 },
            forward: if err.abs() > 0.2 { 0.3 } else { 1.0 },
            ..Default::default()
        };
        sim.step_bond(input, false, false, false);
        w.total_ticks += 1;
    }
    assert!(climbed, "walking into the yard ladder #164 never grabbed it");
    let mut mantled = false;
    for _ in 0..2400 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.bond.substate != nf_game::player::SS_CLIMB && sim.bond.pos[1] > 2.0 { mantled = true; break; }
        sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    assert!(mantled, "the yard climb never mantled past climb_top 12.139");
    for _ in 0..600 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && sim.bond.substate == nf_game::player::SS_WALK && sim.bond.pos[1] > 10.5 { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }

    // Leg 8 (REAL, verbatim): the patrol-ring legs to the west edge.
    w.walk_to(Wp::new(10.5, 11.1, -190.5).r(1.2), "the ring diagonal");
    w.walk_to(Wp::new(12.0, 11.1, -196.0).r(1.2), "the ring diagonal 2");
    w.walk_to(Wp::new(12.0, 11.1, -203.0).r(1.2), "the ring north-west corner");
    w.walk_to(Wp::new(4.0, 11.1, -207.5).r(1.2), "the ring west edge");

    let mut samples: Vec<Sample> = Vec::new();
    sample(&w, di, &mut samples);

    // The fight window: hold Bond still at the west edge (no stick — the west edge drops off),
    // sampling the channel pair every tick for 2400 ticks (~40 s of game time). Guard #1
    // closes and fights ([L the yard census: AimStandFire at ~4.6 u through the west-edge leg]).
    // The walker's own tick keeps Bond's top-up alive — a raw step_bond loop let the 0x1e
    // rounds kill him mid-window (the death sequence drops every opponent pointer).
    for _ in 0..2400 {
        w.tick(nf_game::player::Input::default(), false);
        sample(&w, di, &mut samples);
    }
    let fight = samples.len();

    // The kill leg: Bond arms the 0x1e (the fights' own idiom — the number-key press until the
    // draw lands), raises the aim, and fires through the real path until guard #1 is dead. All
    // ticks go through the walker (top-up on) — the tap is one tick_fire between aim ticks.
    {
        let mut armed = false;
        for _ in 0..120 {
            if w.scene.sim.as_ref().unwrap().gun.weapon == 0x1e { armed = true; break; }
            w.tick(nf_game::player::Input { switch_to: Some(0x1e), ..Default::default() }, false);
        }
        assert!(armed, "Bond never armed the 0x1e (weapon {:#x})", w.scene.sim.as_ref().unwrap().gun.weapon);
        // The aim raise: hold the aim button until the aim bit sets (the raise anim must finish
        // before a tap is a zero-spread round [M Player_SetWeaponAnimObj case 0xf]).
        for _ in 0..120 {
            if w.scene.sim.as_ref().unwrap().aiming { break; }
            w.tick(nf_game::player::Input { aiming: true, ..Default::default() }, false);
        }
    }
    let mut taps = 0u32;
    let killed = {
        let mut killed = false;
        let start = w.total_ticks;
        let mut last_tap = 0u32;
        while w.total_ticks - start < 7200 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let g = &sim.drones.list[di];
                if g.corpse || g.health <= 0.0 { killed = true; break; }
                if (w.total_ticks - start) % 600 == 0 {
                    eprintln!("[0c-comms] kill +{}: guard hp {:.1} corpse {} bond wpn {:#x} clip {} spare {} guard ({:.1},{:.1},{:.1}) bond ({:.1},{:.1},{:.1})",
                        w.total_ticks - start, g.health, g.corpse, sim.gun.weapon, sim.gun.clip, sim.gun.spare,
                        g.pos[0], g.pos[1], g.pos[2], sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
                }
                let bp = sim.bond.pos;
                let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
                let target = [g.pos[0], g.pos[1] - 0.6, g.pos[2]];
                let dx = target[0] - bp[0];
                let dz = target[2] - bp[2];
                let dy = target[1] - eye;
                let desired = dx.atan2(dz);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                let want_pitch = (dy / ((bp[0] - target[0]).powi(2) + (bp[2] - target[2]).powi(2)).sqrt().max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                let perr = want_pitch - sim.bond.pitch;
                let trigger = sim.gun.clip > 0 && sim.aiming && err.abs() < 0.05 && perr.abs() < 0.06 && w.total_ticks - last_tap > 12;
                if trigger { last_tap = w.total_ticks; taps += 1; }
                let input = nf_game::player::Input {
                    turn: err.clamp(-0.08, 0.08),
                    look: perr.clamp(-0.08, 0.08),
                    aiming: true,
                    ..Default::default()
                };
                if trigger { w.tick_fire(input); } else { w.tick(input, false); }
            }
            sample(&w, di, &mut samples);
        }
        killed
    };
    eprintln!("[0c-comms] kill leg: taps {taps}");
    assert!(killed, "Bond never killed guard #1 at the ring — the kill leg failed");
    // The post-kill window: the gate must come DOWN with its only qualifying guard dead.
    for _ in 0..600 {
        w.tick(nf_game::player::Input::default(), false);
        sample(&w, di, &mut samples);
    }

    // ---- the pins ----
    let g = &w.scene.sim.as_ref().unwrap().drones.list[di];
    eprintln!("[0c-comms] guard #1 after the window: alive {} corpse {} state {:#x}; samples {} (fight {})",
        g.health > 0.0, g.corpse, g.state, samples.len(), fight);
    let rise = samples[..fight].iter().position(|s| s.ch98 == 1);
    let first_rise = rise.expect("ch 0x98 never rose during the fight window — the InitComms chatter gate [M 0x139520:201] never fired");
    eprintln!("[0c-comms] 0x98 rose at sample {} (tick {}), guard state: alert {:.2} dist {:.2} qualified {}",
        first_rise, samples[first_rise].tick, samples[first_rise].alert, samples[first_rise].dist, samples[first_rise].g_qualified);
    // The alarm latch rides the same scan [M 0x139520:198-199].
    assert!(samples[..fight].iter().any(|s| s.ch97 == 1),
        "ch 0x97 never raised during the fight window [M 0x139520:198-199]");
    // The gate's value IS the scan's chatter latch, tick for tick (one-tick slack for the
    // HandleSoundAlerts pass that runs after the scan and can re-qualify a guard the same tick
    // [M 0x139520:207-218 vs :197-201]). On the old tree (no writer) the rise assert above is
    // the red; this equality pins the drop too — including the dead guard: a corpse (or a
    // PostDroneControl health-dead guard) whose stale alert/dist would qualify a skip-less scan
    // must not hold the gate up [M :166, 0x14a2f0:9, 0x14aff8:9-12].
    for i in 1..samples.len() {
        let s = &samples[i];
        let prev = &samples[i - 1];
        assert!(s.ch98 as usize == s.rule as usize || s.ch98 as usize == prev.rule as usize,
            "tick {}: ch 0x98 = {} but the scan law says {} (prev {}), guard alive {} qualified {} alert {:.2} dist {:.2}",
            s.tick, s.ch98, s.rule as u8, prev.rule as u8, s.g_alive, s.g_qualified, s.alert, s.dist);
    }
    // The dead-guard observable: after the kill, a tick where the corpse's stale alert/dist
    // would qualify a skip-less scan AND no living guard qualifies — the gate must read 0.
    // (Other yard guards alerted by the fight can legitimately hold the gate up; the pin only
    // binds when the scan's law itself says down.)
    let dead = samples.iter()
        .filter(|s| !s.g_alive && !s.rule)
        .find(|s| s.alert >= 1.0 && s.dist < 30.0);
    match dead {
        Some(s) => {
            assert_eq!(s.ch98, 0,
                "tick {}: the dead guard's stale state (alert {:.2} dist {:.2}) must not hold the chatter gate [M 0x139520:166]",
                s.tick, s.alert, s.dist);
            eprintln!("[0c-comms] the dead-guard pin held at tick {} (ch98 0 with alert {:.2} dist {:.2})", s.tick, s.alert, s.dist);
        }
        None => eprintln!("[0c-comms] no post-kill tick with the law down — the other yard guards held the ring; the skip is pinned by init_comms_scan_law"),
    }
}
