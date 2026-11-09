//! player-ceiling-tails: Bond's head stops at a ceiling the way the game's code says — the
//! jump-bump chain through the real viewer path (`nf_view::load` + the shared walker), walked
//! from the level start with the stick only (no teleports after the load). Opt-in via `NF_ISO`.
//!
//! Game evidence [M]:
//! - The ceiling contact flag: `Player_CollisionHandler` 0x1949a8's live path takes the last
//!   non-climb-object contact of the frame's HITDATA list and, when its position sits above the
//!   per-substate view offset BL+0x760, sets AnimObj flag 0x20 [M :155-165, asm 0x194cbc-0x194ce8
//!   — `lwc1 f0,0x24(sp)` the y of `contact − BL+0x760`, `c.olt.s` 0<y, `ori 0x20`].
//! - `Player_HandleJump` 0x1916b0 consumes it while rising: jump_state == 1 [M :67] and flag 0x20
//!   → jump_state = 2 and `Vec_Zero(BL+0x50)` [M :69-74] — the rise stops.
//! - The jump itself refuses under a roof: `Collide_LineOfSight` from the feet to feet+0.95
//!   [M 0x1916b0:171-178] — no jump start when the headroom is gone.
//! - [D] the ceiling itself: 07000001's office interior — the test walks from spawn #0
//!   (−2.78, 6.84, −12.40 [D the placement rows]) until an upward segment from the head hits
//!   real level geometry within reach, then jumps. The sim is deterministic, so the found spot
//!   is the same every run; the found height is asserted to exist (the [D] proof).
//!
//! The port maps the chain: `head_bump` from the capsule's downward push (the contact above),
//! the rising bump arm at physics_tick's tail (jump 1 + head_bump → vel 0 + jump 2, [M
//! Player_HandleJump 0x1916b0:69-74]), and the headroom refusal in the jump start.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;
use nf_game::player::Input;
use nf_game::world::V3;
fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The ceiling above Bond's head, if any: an upward segment from pos + 1.0 (the chest) over 2.6
/// units, against the level's own cel.
fn ceiling_above(w: &Walker) -> Option<f32> {
    let sim = w.scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    let from: V3 = [p[0], p[1] + 1.0, p[2]];
    let to: V3 = [p[0], p[1] + 3.6, p[2]];
    sim.world.segment_hit_scoped(sim.bond.cel, from, to).map(|(t, _)| from[1] + t * (to[1] - from[1]))
}

/// Walk forward from the spawn with the stick until a low ceiling sits within reach of the rise
/// (the capsule top = root + 0.825 [M Player_Collision 0x194df0 default arm]; the launch rise is
/// ~0.78 [M Player_HandleJump 0x1916b0:184-191]), then stop.
fn walk_to_a_low_ceiling(w: &mut Walker) -> f32 {
    let input = Input { forward: 1.0, ..Default::default() };
    for i in 0..300 {
        w.tick(input, false);
        if let Some(h) = ceiling_above(w) {
            let rise_room = h - w.scene.sim.as_ref().unwrap().bond.pos[1];
            if rise_room < 2.0 && rise_room > 1.15 {
                eprintln!("ceiling spot at tick {i}: root y {:.3}, ceiling bottom {:.3}", w.scene.sim.as_ref().unwrap().bond.pos[1], h);
                return h;
            }
        }
    }
    panic!("no low ceiling within the walk lane on 07000001 (the [D] spot was not found)");
}

/// Jump under the found ceiling: the rise stops at the face — jump 1 (rising) until the contact
/// above reads, then jump 2 with BL+0x50 zeroed and the head parked at the ceiling [M
/// Player_HandleJump 0x1916b0:69-74 via the 0x20 flag of 0x1949a8:155-165].
#[test]
fn the_head_stops_at_the_ceiling_of_the_office() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    w.wait_enabled(6000, "the intro tail");
    w.wait(30);
    let h = walk_to_a_low_ceiling(&mut w);
    let sim = w.scene.sim.as_mut().unwrap();
    let root0 = sim.bond.pos[1];
    // Jump: the real input path (the press rides HandleJump [M 0x1916b0]).
    let mut bumped = false;
    let mut seen_rise = false;
    for _ in 0..3 { w.tick(Input { jump: true, ..Default::default() }, false); }
    for _ in 0..60 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.jump == 1 && sim.bond.vel[1] > 0.0 { seen_rise = true; }
        if sim.bond.jump == 2 {
            bumped = true;
            break;
        }
        if sim.bond.grounded && sim.bond.jump == 0 && seen_rise { break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let apex = sim.bond.pos[1];
    assert!(bumped, "the head never bumped (jump {}, vel {:?}, root {:.3} → {:.3}, ceiling {:.3})",
        sim.bond.jump, sim.bond.vel, root0, apex, h);
    assert!(seen_rise, "the bump came without a rise");
    assert_eq!(sim.bond.vel, [0.0; 3], "the bump arm zeroes BL+0x50 [M Player_HandleJump 0x1916b0:69-74]");
    // The head parked at the face: the capsule top (root + 0.825) stops just under the ceiling
    // bottom the up-ray measured.
    let top = apex + 0.825;
    assert!(top <= h + 0.35, "the head pierced the ceiling: top {top:.3} vs face {h:.3}");
    assert!(h - top < 0.6, "the head never reached the face: top {top:.3} vs face {h:.3} — the ceiling was too far to pin");
}
