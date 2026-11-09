//! player-dead-landing-pain: a dead Bond's landing through the real viewer path
//! (`nf_view::load` + `Sim::step_bond`), opt-in via `NF_ISO`.
//!
//! The game's dead body keeps `Player_CollisionHandler` 0x1949a8 as its collide slot
//! [M obj-tag-player.md:18], and its landing arm calls `Player_HandlePain(timer − 60, type 6)`
//! when the fall timer is over 60 [M 0x1949a8:117-123, the `jal` at 0x194bd4]. The pain
//! pipeline's own dead gate drops it: `Player_HandlePain` 0x1902c8 reads health BL+0x894
//! [M 0x190318], compares `c.le.s` against 0.0 [M 0x190324] and returns at 0x19032c — before
//! its FIRST write (BL+0x8bc = 1.0 at 0x190378). So a dead landing produces no health change,
//! no armour math, no +0x967 direction bits, no +0x968 flash, no `Rand_Rand(4)` grunt draw,
//! no `Sound_Play3D`, no rumble. `Player_Hurt` 0x196060 (the only other damage entry) adds no
//! gate of its own — everything funnels through `HandlePain`. A dead body's health is exactly
//! 0.0 (`Player_SetHealth` 0x1960b8 clamps <0 to 0), so the gate always fires.
//!
//! The port emits no `fall_damage` from the dead tick (`Bond::dead_contacts_tick`), so the
//! `pain_generic` call in scene.rs never runs for a dead body and no side effect lands. This
//! test pins that equality through real input: Bond jumps (real input), is killed on the
//! descent just above the ground — inside the dead feet-ray band (dy − 0.1 ≤ cc
//! [M 0x194188 + the band in `dead_contacts_tick`], cc = ROOT_HEIGHT 1.0347 → ≈1.13 m) — so
//! the dead body lands (the grounded history takes flag 8 [M 0x1949a8:78-82]) with the fall
//! timer armed, and the pin is that NOTHING pain-shaped happens on that dead landing.
//!
//! The kill rides the real damage API the death suite uses (`pain_generic`, the
//! `Player_HandlePain` port), the same one `dead_contacts_live` drives.

use nf_game::player::Input;
fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load(iso: &str, level: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, level, false).expect("load");
    let idle = Input::default();
    for _ in 0..600 { if !scene.sim.as_ref().unwrap().disabled { break; } scene.sim.as_mut().unwrap().step_bond(idle, false, false, false); }
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(idle, false, false, false); }
    assert!(!scene.sim.as_ref().unwrap().disabled, "the intro never released Bond");
    scene
}

fn kill(sim: &mut nf_view::scene::Sim) {
    let (h, r) = (&mut sim.drones.health, &mut sim.drones.rng);
    h.pain_generic(r, 500.0, 1, 0x0700_0005, 0);
}

/// A dead body killed just above the ground lands dead, and the landing produces no pain:
/// no fall damage, no health change, no direction-bit or flash write, no movement —
/// the game's `Player_HandlePain` dead gate [M 0x190318-0x19032c] shape.
#[test]
fn dead_landing_produces_no_pain() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, "07000005.bin");
    let sim = scene.sim.as_mut().unwrap();
    let floor_y = sim.bond.pos[1];
    // Real input: hop from the spawn ground and kill on the descent in the LAST centimetres —
    // the dead feet-ray band is measured from the root (dy ≤ cc + 0.1 [M the band in
    // `dead_contacts_tick`], i.e. only ~0.1 m above the resting root), a 1-2-tick window per
    // hop, so the hops repeat until the descent passes through it. The frozen body then
    // grounds on the first dead tick and the game's landing arm runs with the fall timer armed.
    let mut killed = false;
    for _hop in 0..8 {
        for _ in 0..60 {
            sim.step_bond(Input { jump: true, ..Default::default() }, false, false, false);
            let h = sim.bond.pos[1] - floor_y;
            // The alive ground snap fires inside this same band (the root snaps to the floor
            // within the last ~0.1 m), so the window is "falling and inside the band" —
            // grounded or a hair above it; both freeze inside the dead feet-ray band.
            if sim.bond.vel[1] < 0.0 && h < 0.1 {
                kill(sim);
                killed = true;
                break;
            }
        }
        if killed { break; }
        // Back to the ground (idle) before the next hop — the jump input re-arms from rest.
        for _ in 0..30 {
            sim.step_bond(Input::default(), false, false, false);
            if sim.bond.grounded && sim.bond.vel[1] <= 0.0 { break; }
        }
    }
    assert!(killed, "the descent never passed through the dead-feet band (pos y {:?}, vel {:?}, grounded {})",
        sim.bond.pos[1], sim.bond.vel, sim.bond.grounded);
    sim.step_bond(Input::default(), false, false, false);
    assert!(sim.drones.death.active, "the death sequence started mid-fall");
    let (dir0, flash0) = (sim.drones.health.dir_bits, sim.drones.health.flash);
    let vel0 = sim.bond.vel;
    let pos = sim.bond.pos;
    // The kill's own tick ran the alive physics once (the harness's shape in dead_contacts_live);
    // the dead landing is measured from the first dead tick on.
    let mut landed = false;
    for i in 0..90 {
        sim.step_bond(Input::default(), false, false, false);
        assert!(sim.bond.fall_damage.is_none(),
            "tick {i}: the dead landing must not emit fall damage — the game's HandlePain dead gate returns at 0x19032c before any write [M 0x190318-0x190378]");
        assert_eq!(sim.drones.health.health, 0.0,
            "tick {i}: the dead body's health is exactly 0.0 (SetHealth clamps <0 to 0 [M 0x1960b8]) and the landing changes nothing");
        assert_eq!(sim.drones.health.dir_bits, dir0,
            "tick {i}: the dead landing writes no +0x967 direction bits [M the gate returns before the 0x967 write 0x1902c8:121-125]");
        assert!(sim.drones.health.flash <= flash0 + 1e-6,
            "tick {i}: the dead landing adds no +0x968 flash [M the gate returns before the 0x968 write 0x1902c8:126-130; the pane's decay may only lower it]");
        let p = sim.bond.pos;
        let d = [(p[0] - pos[0]).abs(), (p[1] - pos[1]).abs(), (p[2] - pos[2]).abs()];
        assert!(d[0] < 1e-4 && d[1] < 1e-4 && d[2] < 1e-4,
            "tick {i}: the dead body must not move [M Player_Update 0x193600:316-319] (drift {d:?})");
        if sim.bond.grounded_history & 8 != 0 {
            landed = true;
            // The dead landing ran: the grounded arm holds the fall timer at 0 [M 0x1949a8:117-123
            // — under 60 the timer just clears; the observable is the same through the gate].
            // The frozen velocity stays the death tick's: the ceiling arm skips while the death
            // froze jump state 1 (killed mid-descent) [M :168-175 gates on jump != 1], and no
            // dead-tick writer moves BL+0x50 otherwise.
            assert_eq!(sim.bond.fall_timer, 0, "tick {i}: the dead landing's grounded arm clears the fall timer");
            assert_eq!(sim.bond.vel, vel0, "tick {i}: the dead landing writes no velocity — the frozen BL+0x50 holds [M :168-175's jump gate + no dead writer]");
        }
    }
    assert!(landed, "the dead body never grounded — the kill window must sit inside the dead feet-ray band (grounded_history {:?})",
        sim.bond.grounded_history);
}
