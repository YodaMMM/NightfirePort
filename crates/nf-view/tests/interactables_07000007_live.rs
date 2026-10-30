//! interactables-07: one live test per KIND of player-touchable placement on 07000007 (the
//! Exchange, part 3: the party) that `interact_07000007_live.rs` does not already pin. Driven the
//! way a player drives it (the walk-over), each asserting the game's effect with its sister. The
//! census with the type table lives in docs/levels/07000007.md §7; the end-to-end route is
//! walkreal_07000007_live.rs.
//!
//! Covered elsewhere (the census → pin map): the 0x20 breakables (vase/window/statue), the 0xF0
//! pickups (armour +50, the kind-6 medal-gated bonuses), the 0xDB doors (free #245 / locked-200
//! #253 / the story door #251) and the 0xE8 exit — all in interact_07000007_live.rs,
//! party_door_lock_07000007_live.rs and walk_07000007_live.rs. This file adds the remaining
//! player-touchable kinds: the 0xEA walk-over TouchOnce plates, the 0xEB Touch plate, and the
//! player side of the 0x10 drone-only mask.
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes. A teleport to reach a spot is
//! used only where the walk there is documented (the probe notes at each site); the operation
//! itself always runs through the real input/contact.

#[path = "common/walker_b2.rs"]
mod walker;

use nf_game::player::Input;
use nf_game::triggers::kind;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Load, skip the intro, hand the walker back.
fn load() -> Walker {
    let scene = nf_view::load(&iso(), "07000007.bin", false).expect("load 07000007");
    let mut w = Walker::new(scene);
    w.wait_enabled(4000, "the intro");
    w
}

/// KIND 0xEA walk-over (TouchOnce #500, out ch 169, mask 8 = player, lock ch 99 [D params
/// (0,169),(1,8),(7,99) — ch 99 is the mission-fail channel, off on a fresh load]): the plate
/// fires when Bond's capsule walks into its volume and the TouchOnce LATCHES and REMOVES itself
/// [M Trigger_Create 0x1b1720 the param read; Trigger_Hit 0x1b1fc8's mask-8 arm; Trigger_Update's
/// TouchOnce self-delete — triggers.rs `glist_0x10`/gone]. The take-side effect is the global
/// sound trigger #499 wired to the same channel (on 169 → sfx 1087, global [D #499 params
/// (0,169),(2,1087),(5,1); M SoundTrigger_Create 0x1b2628]).
///
/// The walk: from the fireplace-hall 0x2d start marker (91.59, 44.25, −27.98 [D placements]) four
/// metres south across the plate band (90.5..92.5, 43.2..44.1, −32.6..−31.5 [D world bounds]).
/// The teleport to the marker is the stand-in transit (the fireplace hall's approach from the
/// party start is unwalked — no route pins it); the walk over the plate is the real input.
#[test]
fn the_fireplace_walkover_touchonce_fires_ch169_once() {
    let mut w = load();
    let ch = |w: &Walker, c: u32| w.scene.sim.as_ref().unwrap().drones.channels.on(c);
    // Sister: the plate and its sound trigger exist with the disc wiring, and the channel is off.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let t = sim.drones.triggers.iter()
            .find(|t| t.kind == kind::TOUCH_ONCE && u32::from(t.out) == 169)
            .expect("the TouchOnce #500 exists");
        assert!(!t.gone, "the plate sits before the walk");
        assert_eq!(t.mask, 8, "the disc mask is the player bit [D #500 param 1]");
        assert!(sim.drones.sound_triggers.list.iter()
            .any(|s| s.on == 169 && s.id == 1087 && s.global),
            "the sound trigger #499 is wired to ch 169 → sfx 1087, global [D #499; M 0x1b2628]");
        assert!(!ch(&w, 169), "sister: ch 169 off before the walk");
    }
    // The stand-in transit: the 0x2d start marker, then the real walk south across the band
    // (bounded forward drive: the walk_to overshoots into the hall's far furniture and wedges).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [91.59, 44.25, -27.98];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI; // face −z, the plate band
    let mut fired = false;
    for _ in 0..300 {
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
        if ch(&w, 169) { fired = true; break; }
    }
    for _ in 0..60 { w.tick(Input::default(), false); }
    assert!(fired, "the walk-over fires ch 169 [D #500 out; M Trigger_Hit 0x1b1fc8 mask 8]");
    let gone = w.scene.sim.as_ref().unwrap().drones.triggers.iter()
        .find(|t| t.kind == kind::TOUCH_ONCE && u32::from(t.out) == 169)
        .map(|t| t.gone).unwrap_or(true);
    assert!(gone, "the TouchOnce latches and removes itself [M Trigger_Update's self-delete]");
}

/// KIND 0xEB Touch (#443 "Steve's Music Start", out ch 138, mask 8 = player [D params (0,138),
/// (1,8)]): the standing contact against the plate's own plane (world bounds (98.2,52.5,−87.6)..
/// (101.0,52.5,−87.1) [D]) fires the channel through the real trigger tick, and ch 138 is the
/// MusicTrigger #444's on-channel (ch_on 138 → Music_Event(13) [D #444 params (0,138),(2,13);
/// M MusicTrigger_Update 0x1b31a0 — worldobj.rs]). The port's music board slot is consumed inside
/// the frame; the audible beat is the audio layer's (a live check), so the pin is the channel and
/// the trigger's disc wiring.
///
/// The stand: the plate's plane rides 2.4 m above the terrace floor (50.1 [D probe]); its west
/// edge is reachable from the step top at (98.0, ≈50.98, −87.3), whose capsule contact fires it.
/// The step ascent is the documented stand-in: from the terrace the walker stops at its base
/// (97.4, −86.7) and a direct forward walk slides along the step's face and falls (the probe in
/// this task's report) — the teleport only reaches the spot; the touch is the game's contact path.
/// Sister: ch 138 off at the balcony stand away from the band.
#[test]
fn the_roof_music_touch_fires_ch138_and_the_music_trigger_is_wired() {
    let mut w = load();
    let ch = |w: &Walker, c: u32| w.scene.sim.as_ref().unwrap().drones.channels.on(c);
    // Sister + the disc wiring, before any touch.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.triggers.iter()
            .any(|t| t.kind == kind::TOUCH && u32::from(t.out) == 138 && t.mask == 8),
            "the Touch #443 exists with the disc mask [D #443 params (0,138),(1,8)]");
        let mt = sim.drones.worldobjs.music_triggers.iter()
            .find(|t| t.ch_on == 138).expect("the MusicTrigger #444 exists");
        assert_eq!(mt.event_id, 13, "the music trigger fires event 13 [D #444 param 2; M 0x1b31a0]");
        assert_eq!(mt.ch_flag, 150, "the music trigger's flag channel [D #444 param 1]");
        assert!(!ch(&w, 138), "sister: ch 138 off before the touch");
    }
    // The terrace stand away from the band keeps it off (the sister's walk half).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [95.49, 50.3, -85.24]; // the balcony 0x2d marker [D placements]
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    assert!(!ch(&w, 138), "the balcony stand is outside the band");
    // The stand-in transit onto the step top by the plate's west edge; the contact is real.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [98.0, 51.2, -87.3];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..90 {
        w.tick(Input::default(), false);
        if ch(&w, 138) { break; }
    }
    assert!(ch(&w, 138), "the standing contact fires ch 138 [D #443; M Trigger_Hit 0x1b1fc8 mask 8]");
}

/// The mask sister: TouchOnce #497 (out ch 9, mask 0x10 = DRONE-only [D params (0,9),(1,16)]) is
/// player-touchable terrain but never answers the player: `Trigger_Hit` 0x1b1fc8's 0x10 arm
/// returns false without a drone contact [M triggers.rs hit()'s `m == 0x10 && !drone_touch`]. The
/// drone half is the guests' own walk (the blocker #496 gate [D §7]); the player half is this pin.
/// The walk: the north-row lane (98.6, −55.8) → east across the plate band (100.0..101.0,
/// 43.3..44.2, −55.1..−57.0 [D]) — the lane is walkreal_07000007's proven hall route (its node at
/// (98.6, −55.8) → (105, −55.9)); the teleport to the node is the stand-in transit from the spawn.
#[test]
fn the_drone_mask_plate_refuses_the_player() {
    let mut w = load();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let t = sim.drones.triggers.iter()
            .find(|t| t.kind == kind::TOUCH_ONCE && u32::from(t.out) == 9)
            .expect("the TouchOnce #497 exists");
        assert_eq!(t.mask, 0x10, "the disc mask is the drone bit [D #497 param 1]");
        assert!(!sim.drones.channels.on(9), "sister: ch 9 off before the walk");
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [98.6, 43.6, -55.8];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::FRAC_PI_2; // face +x, the plate
    let mut crossed = false;
    for _ in 0..300 {
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
        if w.scene.sim.as_ref().unwrap().bond.pos[0] >= 99.6 { crossed = true; break; }
    }
    for _ in 0..90 { w.tick(Input::default(), false); }
    assert!(crossed, "Bond walked into the plate's band");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(9),
        "the drone-only plate never answers the player [D #497 mask 0x10; M Trigger_Hit 0x1b1fc8]");
}
