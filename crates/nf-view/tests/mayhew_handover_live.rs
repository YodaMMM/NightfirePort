//! mayhew-exit-walk: 07000005 (Mayhew A) — the walk to the exit before the mid cut-scene.
//!
//! The level's truck-exit chain [D docs/levels/07000005.md §3; the truck script 0x06000A22]:
//! SP #37 (mode 1) plays the truck script at level start; stream 8 creates the script TouchOnce
//! ch 0x5C at t 1882 and kills it at t 2312 (0x908) [D script decode; the sim trace: the trigger
//! appears between the t 2100/2400 samples] — a static plate whose world centre is
//! (98.49, 27.05, 106.36) (the gatehouse walkway edge above the arch road; the walked lane
//! there is y ≈ 25.9). When Bond touches it, SP #38 (script 0x060008C8, 143 ticks, camera true,
//! Bond skin 0x05000007 on anim 0x06000A24 [D the script's stream 1]) plays — the mid cut-scene
//! that hands Bond onto the waiting truck — and the truck rides on: the bed touch ch 0x71
//! (script stream 7, parented to the truck) parents Bond; the park plate ch 0x70 (stream 6);
//! the gate-guard ch 0x72 (placement #318); mux #316 = AND(0x70, 0x71, 0x72) → SP #320
//! "EndLevel" (0x06000A23, event 0x11 at t 300 [D]) → ch 0x1D → #118 loads 07000006.
//!
//! Mayhew's walk to the exit before the mid cut-scene was broken; the fault is in this chain: walked Bond must reach the gatehouse edge INSIDE the plate's
//! live window. The probe on master (2026-10-17): the real-input walker arrives at the edge
//! (99.69, 25.85, 106.98) at t ≈ 373, waits, ch 0x5C fires while the window is open, SP #38
//! re-places Bond on the truck bed and ch 0x71 follows the ride. There is NO Mayhew drone on
//! 07000005 (16 drones, all guards 0x0500002e/2f/30 [D nf-extract drones]) — "Mayhew" is the
//! level's own name (Mayhew A).
//!
//! Sister: Bond who never walks to the gatehouse never sees the hand-over — ch 0x5C stays
//! low through the whole window and the level does not exit.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The hand-off plate ch 0x5C is up (the truck script made it [D stream 8]) and not yet fired.
fn handoff_plate_up(w: &Walker) -> bool {
    let sim = w.scene.sim.as_ref().unwrap();
    !sim.drones.channels.on(0x5c) && sim.drones.triggers.iter().any(|t| u32::from(t.out) == 0x5c && !t.gone)
}

/// Stand at the gatehouse edge until the plate is up, stand on 120 more ticks, then jump in place
/// until ch 0x5C fires. The game's standing contact (the walk HITTEST segment, top A = pos + 0.275
/// [M Player_Collision 0x194df0 default arm]) sits 0.56 under the quad at y 27.05, outside r 0.55;
/// the reference trace stood inside the quad's footprint for 212 frames with ch 0x5C at 0, jumped
/// (pad 0x10) and ch 0x5C rose 8 frames later [L tests/traces/05-truck.trace frames 33244-33464].
/// Returns the fire tick.
fn stand_then_jump_onto_the_plate(w: &mut Walker) -> Option<u32> {
    for _ in 0..3600u32 {
        if handoff_plate_up(w) { break; }
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x5c) { return Some(w.total_ticks); }
    }
    for _ in 0..120u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x5c) {
            assert!(!nf_game::drone::TOUCH_LAW_LIVE,
                "standing under the plate fired ch 0x5C at t {}; the game's standing contact misses it [L 05-truck.trace]", w.total_ticks);
            return Some(w.total_ticks);
        }
    }
    for _ in 0..600u32 {
        let jump = w.scene.sim.as_ref().unwrap().bond.grounded;
        w.tick(nf_game::player::Input { jump, ..Default::default() }, false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x5c) { return Some(w.total_ticks); }
    }
    None
}

#[test]
fn the_walk_to_the_gatehouse_fires_the_handover_window_and_rides_out() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    // The cold start #26 (gate 0) [D]: (104.95, 25.12, 108.83) — the gatehouse road.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 100.0 && p[0] < 110.0 && p[2] > 100.0 && p[2] < 115.0,
            "the cold start spawns at the gatehouse road (#26 [D]), got {p:?}");
        assert!(!sim.disabled, "no intro blocks this level's start");
    }
    w.wait(200);
    // The real-input walk (the walker drives step_bond with Input only) to the gatehouse
    // walkway edge — INSIDE the ch-0x5C plate's XZ (centre (98.49, 27.05, 106.36) [D the
    // entity matrix], extent ±1.95/±1.75 [D the TouchplateBig glist box]) — while the truck
    // script's window is still ahead of us (the plate lives t 1882..2312 [D stream 8]).
    w.walk_nav_to([98.6, 26.0, 106.5], "to the gatehouse walkway edge", 1.2);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = sim.bond.pos;
        let d = ((b[0] - 98.49).powi(2) + (b[2] - 106.36).powi(2)).sqrt();
        assert!(d < 2.0, "Bond reaches the plate's XZ within the window (t {}), d = {d:.2} at {b:?}",
            w.total_ticks);
        assert!(w.total_ticks < 1882, "the walk arrives before the plate even exists (t {})",
            w.total_ticks);
        assert!(!sim.drones.channels.on(0x5c), "the plate is not up yet [D t 1882]");
    }
    // Wait at the edge through the window: the plate appears, Bond jumps into it, it fires, the
    // hand-over scene plays (Bond disabled during the cut-scene, then re-placed on the truck bed).
    let fired_at = stand_then_jump_onto_the_plate(&mut w);
    let fired_at = fired_at.expect("ch 0x5C fires when Bond jumps at the gatehouse edge [D stream 8's window; L 05-truck.trace]");
    eprintln!("[05] ch 0x5C fired at t {fired_at}");
    assert!(fired_at >= 1882 && fired_at <= 2400,
        "the fire lands inside the script's own window (t 1882..2312 [D]), got {fired_at}");
    // The hand-over + the ride: Bond ends up parented to the truck (ch 0x71); the game's ride
    // stance is crouched [D fix-truck's trace] — the walked Bond crouches once the hand-over
    // re-enables him — and the crouched ride parks on the ch-0x70 plate, the chain ends the
    // level into 07000006 [D mux #316 → SP #320 → ch 0x1D → #118].
    let mut rode = false;
    let mut crouched = false;
    let mut input = nf_game::player::Input::default();
    let mut exited = None;
    for _ in 0..6800u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let on_bed = sim.drones.channels.on(0x71);
        let west = sim.bond.pos[0] < 90.0;
        if !crouched && on_bed && west && !sim.bond.crouched {
            crouched = true;
            input.crouch = true;
        } else {
            input.crouch = false;
        }
        drop(sim);
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if on_bed && west { rode = true; }
        if let Some(l) = sim.drones.level_exit { exited = Some(l); break; }
    }
    assert!(rode, "the hand-over puts Bond on the truck bed (ch 0x71) and the ride carries him west of the gatehouse");
    assert_eq!(exited, Some(0x0700_0006),
        "the crouched ride parks on ch 0x70 and the chain ends the level into 07000006 [D docs/levels/07000005.md §3]");
}

/// Sister: Bond who stays at the spawn never fires the hand-over — the plate's window opens
/// and closes (t 1882..2312 [D]) with ch 0x5C low the whole time, and the level does not exit.
#[test]
fn the_handover_never_fires_for_bond_who_stays_at_the_spawn() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    for _ in 0..3000u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0x5c),
        "the ch-0x5C hand-over never fires without the walk [D the plate is player-mask 8]");
    assert!(sim.drones.level_exit.is_none(), "the truck chain does not end the level from the spawn");
}

/// Sister: the STANDING ride — the same real-input walk to the gatehouse, the hand-over, and
/// then the player NEVER crouches. The park plate ch 0x70 is a mask-8 player trigger [D the
/// truck script 0x06000A22 stream 6: Entity 0x0200087F, params (0x70, 8, 0…)] and the standing
/// rider's own game segment reaches it: B = pos+0.55−cc = pos−0.4847 over the bed (feet 37.28
/// [D block 0x2E of 0x020003be]), the plate at 37.7418 sitting 0.163 under B — inside r 0.55
/// [M Player_Collision 0x194df0's default arm; Trigger_Hit 0x1b1fc8:56-58's mask-8 arm credits
/// the class-3 contact; the truck's own class-0x1b contact [M SP_CreateScriptPlayer
/// 0x1db9a8:71] credits nothing]. A rider who stays standing is NOT stuck: 0x70 fires on the
/// ride and the chain ends the level into 07000006.
#[test]
fn the_standing_ride_still_parks_and_ends_the_level() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 100.0 && p[0] < 110.0 && p[2] > 100.0 && p[2] < 115.0,
            "the cold start spawns at the gatehouse road (#26 [D]), got {p:?}");
        assert!(!sim.disabled, "no intro blocks this level's start");
    }
    w.wait(200);
    w.walk_nav_to([98.6, 26.0, 106.5], "to the gatehouse walkway edge", 1.2);
    let fired_at = stand_then_jump_onto_the_plate(&mut w);
    let fired_at = fired_at.expect("ch 0x5C fires when Bond jumps at the gatehouse edge [D stream 8's window; L 05-truck.trace]");
    eprintln!("[05-standing] ch 0x5C fired at t {fired_at}");
    // The hand-over + the STANDING ride: no crouch input at all. The bed touch ch 0x71 parents
    // Bond and the ride carries him west; the standing rider's own segment reaches the park
    // plate (0.163 under B < r 0.55 [M]), so the chain completes and the level exits.
    let mut rode = false;
    let mut stood = true;
    let mut exited = None;
    let mut t70 = None;
    for _ in 0..6800u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let on_bed = sim.drones.channels.on(0x71);
        let west = sim.bond.pos[0] < 90.0;
        if sim.bond.crouched { stood = false; }
        if on_bed && west { rode = true; }
        drop(sim);
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if t70.is_none() && sim.drones.channels.on(0x70) { t70 = Some(w.total_ticks); }
        if let Some(l) = sim.drones.level_exit { exited = Some(l); break; }
    }
    assert!(stood, "the ride stays standing (no crouch input through the whole ride)");
    assert!(rode, "the hand-over puts Bond on the truck bed (ch 0x71) and the standing ride carries him west");
    assert!(t70.is_some(), "the standing rider's own contact fires the park plate ch 0x70 (mask 8 [D]; the segment reach [M])");
    assert_eq!(exited, Some(0x0700_0006),
        "the standing ride is not stuck: the chain ends the level into 07000006 [D mux #316 → SP #320 → ch 0x1D → #118]");
}
