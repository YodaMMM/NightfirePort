//! stand-idle-variants: `DroneAnim_SetStandIdleAnim` 0x13ca68 — which stand idle a drone plays.
//!
//! The game's pick [M 0x13ca68]: with the respect argument set and the hold clock `+0x59c` still
//! running it returns at once (:19-21); in Idle (SM state `+0x10c` == 4) the state is
//! `0x24 + Rand(3)` and the hold `(5 + Rand(5)) * FRAME_RATE_INT` (:31-37); in PartyGirl (0x2f)
//! with behaviour property 0x47 set the same with `Rand(4)` — 0x24..0x27 (:39-42); every other
//! state takes 0x24 with no hold (:27-28). A model `+0xD8 == 0x10` always asks 0x26 (:46-48); a
//! state outside 0x24..0x27 or one the anim set cannot play falls back to 0x24 (:49-68); the pick
//! always goes through `DroneAnim_CallAnim` (:70) and stamps the clock (:71).
//!
//! The party guests on 07000007 are the PartyGirl drones: 9 stand in state 0x2f at level start,
//! all kind 0xd, anim set 0xe (which plays 0x24..0x27) with property 0x47 set [D census over the
//! level's drones after 30 ticks]. Pre-fix the port stood every one of them on 0x24 for good.
//!
//! State ids are written raw so this file compiles — and fails — against the pre-fix master.
//! Opt-in via `NF_ISO`.

use nf_view::scene::{Scene, Sim};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn guests(sim: &Sim) -> Vec<usize> {
    sim.drones.list.iter().enumerate().filter(|(_, d)| d.present && d.state == 0x2f).map(|(i, _)| i).collect()
}

/// The idle state a drone is in or asked for.
fn idle_of(sim: &Sim, i: usize) -> u16 {
    let a = &sim.drones.list[i].anim;
    a.call.map_or(a.state, |c| c.0)
}

/// The party guests cycle through the four stand idles 0x24..0x27, each held 5..9 s
/// [M 0x13ca68:39-42 + :31-37]: over 40 s some guest is seen on a variant above 0x24, and no
/// guest changes its pick faster than the 5 s floor.
#[test]
fn party_guests_cycle_the_four_stand_idles() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    let g = guests(sim);
    assert!(g.len() >= 5, "07000007 starts with its party guests in PartyGirl 0x2f [D] — got {}", g.len());
    let mut seen = std::collections::BTreeSet::new();
    let mut last: Vec<(u16, u32)> = g.iter().map(|&i| (idle_of(sim, i), 0)).collect();
    let mut fastest = u32::MAX;
    for t in 1..=2400u32 {
        sim.step_bond(Default::default(), false, false, false);
        for (k, &i) in g.iter().enumerate() {
            if sim.drones.list[i].state != 0x2f { continue }
            let now = idle_of(sim, i);
            if (0x24..=0x27).contains(&now) { seen.insert(now); }
            if now != last[k].0 && (0x24..=0x27).contains(&now) && (0x24..=0x27).contains(&last[k].0) {
                if last[k].1 != 0 { fastest = fastest.min(t - last[k].1); }
                last[k] = (now, t);
            }
        }
    }
    assert!(seen.iter().any(|&s| s > 0x24),
        "PartyGirl with prop 0x47 picks 0x24 + Rand(4) [M 0x13ca68:39-42] — only {seen:x?} seen in 40 s");
    assert!(fastest >= 5 * 60,
        "each pick holds (5 + Rand(5)) s [M 0x13ca68:36-37, :71] — a guest changed after {fastest} ticks");
}

/// Sister: a model-0x10 drone always asks stand idle 0x26, whatever its state [M 0x13ca68:46-48];
/// the same guest with its own model takes the PartyGirl pick (any of 0x24..0x27).
#[test]
fn a_model_0x10_drone_always_asks_idle_0x26() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    let i = guests(sim)[0];
    let far = [sim.drones.list[i].pos[0] + 60.0, sim.drones.list[i].pos[1], sim.drones.list[i].pos[2]];
    sim.bond.pos = far;
    send_update(&mut scene, i, 0x10);
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.drones.list[i].state, 0x2f, "the guest stays in PartyGirl (no armed person near)");
    assert_eq!(idle_of(sim, i), 0x26, "model +0xD8 == 0x10 asks 0x26 [M 0x13ca68:46-48]");
}

/// One PartyGirl UPDATE with the hold clock expired and the model byte set to `kind`.
fn send_update(scene: &mut Scene, i: usize, kind: u16) {
    let sim = scene.sim.as_mut().unwrap();
    let difficulty = sim.drones.difficulty;
    let data = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty,
        ai_points: &sim.ai_points,
    };
    let frame = sim.drones.frame;
    let dr = &mut sim.drones.list[i];
    dr.kind = kind;
    dr.stand_until = 0;
    dr.send(&data, &mut sim.drones.rng, frame, 3);
}
