//! player-ceiling-tails: the dead player's collide-slot bookkeeping (the "dead-state" tail of
//! `Player_CollisionHandler` 0x1949a8) through the real viewer path (`nf_view::load` +
//! `Sim::step_bond`). Opt-in via `NF_ISO`.
//!
//! What the game does after death, per frame [M]:
//! - `Player_Update` 0x193600's dead branch (state obj+0xF4 2/3) runs ONLY `AnimObjectUpdate` +
//!   `Player_HandleDeath` [M 0x193600:316-319] — no movement, no `Player_Collision` rebuild. The
//!   port froze the whole body for this (death-animation-sp pinned the freeze; the pre-fix port
//!   fell −3.06 units over ~55 dead ticks).
//! - BUT the dead player's class (0x12) keeps `Player_CollisionHandler` 0x1949a8 as its collide
//!   slot [M obj-tag-player.md:18], and `Collide_Update` 0x1ea398:55-69 dispatches every object's
//!   collide slot unconditionally every frame. So the dead body still runs, per tick:
//!   `BL+0x95F = 0xFF` BEFORE the freeze-gate return [M :21-24, asm 0x1949e4 precedes the 0x94a
//!   gate], `Player_FeetOnPoint(dead)` → the grounded flag 8 [M :76-77 + 0x194188:176-185; the
//!   dead arg has no consumer — a3 is clobbered by Mat_GetUp in the prologue [M disasm
//!   0x194188]], the grounded history [M :78-82], the fall timer [M :97-131], the 0x7000012
//!   void-fall latch [M :104-108], and the ceiling arm [M :168-175: grounded && jump_state != 1
//!   → BL+0x14 = 0 [M 0x194d0c] + `Vec_Zero(BL+0x50)` [M 0x194d10 → 0x1e2d58]]. The per-hit list
//!   walk (:136-165) never runs — obj+0xD0 is freed at death [M obj-tag-player.md:51].
//!
//! The kill rides the real damage API the death suite uses (`pain_generic`, the
//! `Player_HandlePain` port), the same one camera_death_crouch_live drives.

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

/// A dead grounded body: the collide-slot bookkeeping keeps running while the body freezes.
/// The icon context clears [M 0x1949a8:21-24], the feet ray still grounds [M :76-82],
/// and the ceiling arm zeroes the world velocity [M :168-175].
#[test]
fn dead_body_keeps_the_collide_slot_bookkeeping() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, "07000005.bin");
    let sim = scene.sim.as_mut().unwrap();
    // A stale third-icon context (the shape the arm loop writes, [M 0x196e80:38-53]) — the spawn
    // spot has no icon nearby, so nothing re-arms over the clear.
    sim.bond.icon_ctx = 6;
    kill(sim);
    sim.step_bond(Input::default(), false, false, false);
    assert!(sim.drones.death.active, "the death sequence started [M 0x190810]");
    assert_eq!(sim.bond.icon_ctx, 0xFF,
        "the dead tick clears BL+0x95F [M Player_CollisionHandler 0x1949a8:21-24] (got {})", sim.bond.icon_ctx);
    let pos = sim.bond.pos;
    assert!(sim.bond.grounded, "the spawn spot is ground; the dead feet ray grounds [M 0x194188:176-185]");
    assert_eq!(sim.bond.grounded_history & 8, 8,
        "the grounded history takes flag 8 while dead [M 0x1949a8:78-82]");
    assert_eq!(sim.bond.vel, [0.0; 3],
        "the ceiling arm zeroes BL+0x50 on a grounded non-rising dead body [M 0x1949a8:168-175]");
    for i in 0..60 {
        sim.step_bond(Input::default(), false, false, false);
        assert_eq!(sim.bond.icon_ctx, 0xFF, "tick {i}: the clear re-runs every dead tick [M :21-24]");
        let p = sim.bond.pos;
        let d = [(p[0] - pos[0]).abs(), (p[1] - pos[1]).abs(), (p[2] - pos[2]).abs()];
        assert!(d[0] < 1e-4 && d[1] < 1e-4 && d[2] < 1e-4,
            "tick {i}: the dead body must not move [M Player_Update 0x193600:316-319] (drift {d:?})");
        assert_eq!(sim.bond.vel, [0.0; 3], "tick {i}: the ceiling arm holds BL+0x50 at zero [M :168-175]");
    }
}

/// A dead airborne body: the update-slot freeze holds (no falling — the pre-fix port dropped
/// −3.06 over ~55 ticks), the grounded history reads 0 and the fall timer keeps counting
/// [M 0x1949a8:97-103] while the frozen y velocity stays negative.
#[test]
fn dead_airborne_body_freezes_and_keeps_counting_the_fall_timer() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, "07000005.bin");
    let sim = scene.sim.as_mut().unwrap();
    // Jump from the spawn ground through the real input path, kill at the rise's apex.
    let mut apex = false;
    for _ in 0..40 {
        sim.step_bond(Input { jump: true, ..Default::default() }, false, false, false);
        if sim.bond.jump == 1 && sim.bond.vel[1] <= 0.0 { apex = true; break; }
    }
    assert!(apex, "the jump never left the ground (jump {}, vel {:?})", sim.bond.jump, sim.bond.vel);
    assert!(!sim.bond.grounded, "airborne at the apex");
    kill(sim);
    sim.step_bond(Input::default(), false, false, false);
    assert!(sim.drones.death.active, "the death sequence started mid-air");
    assert_eq!(sim.bond.grounded_history, 0, "airborne dead: the history reads 0 [M :78-82]");
    // The kill's own tick ran the alive physics once (the health check sees 0 in the drones
    // phase, after the player's move) — the freeze is measured from the first dead tick on.
    let pos = sim.bond.pos;
    let t0 = sim.bond.fall_timer;
    for i in 0..60 {
        sim.step_bond(Input::default(), false, false, false);
        let p = sim.bond.pos;
        let d = [(p[0] - pos[0]).abs(), (p[1] - pos[1]).abs(), (p[2] - pos[2]).abs()];
        assert!(d[0] < 1e-4 && d[1] < 1e-4 && d[2] < 1e-4,
            "tick {i}: the dead body must not fall [M 0x193600:316-319 — no obj+0x30 writer] (drift {d:?})");
        assert_eq!(sim.bond.grounded_history, 0, "tick {i}: still airborne [M :78-82]");
        assert!(sim.bond.fall_timer >= t0 + 1, "tick {i}: the fall timer keeps counting while dead-airborne with a negative frozen y velocity [M 0x1949a8:97-103]");
    }
}
