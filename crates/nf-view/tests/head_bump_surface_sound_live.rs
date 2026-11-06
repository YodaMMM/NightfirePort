//! head-bump-surface-sound: the head-bump sound follows the contact's surface byte.
//!
//! [M `Player_HandleJump` 0x1916b0:194-198] the bump arm reads `*(obj+0xDC)+0x50` — the anim
//! record's CONTACT byte, not a level tag — and plays `0x5FB` when `byte - 0xd < 2` (0xD/0xE,
//! the two WATERs [M docs/fields/hitdata-spawner-ally.md +0x50]), else `0xA3`. The byte is
//! cleared every tick by `Player_CollisionHandler` 0x1949a8:29 [M disasm 0x194a18
//! `sb zero,80(v1)`] and written by the collision path's 0x60-byte HITDATA copies
//! (`Player_FeetOnPoint` 0x194188 tail [M disasm 0x1943cc-0x19440c], `Player_DealWithObjHit`
//! 0x194458:65-85) — it is the surface material of the contact, 0 when nothing was hit.
//!
//! The port captures the ceiling contact's raw surface byte where the downward push resolves
//! (`CollisionWorld::capsule_push_scoped`'s third return) and plays from it. The real 0xD/0xE
//! ceilings [D census over every 0700*.bin placement collision: 0xD appears NOWHERE; 0xE
//! down-facing only in 07000002 map 51 obj 19 and 07000016 map 63 obj 94 — the pond undersides]:
//! the 07000002 culvert runs under the pond (dry 0xA floor ~2.09-2.61, the pond's 0xE underside
//! at 4.33-4.34, gap 1.72-2.23) — the jump-bumpable water ceiling. The unit pins carry the arms
//! the disc census cannot reach with real geometry.
//!
//! Opt-in via NF_ISO for the real-level tests; the unit pins run anywhere.

use nf_game::player::{Bond, Input, LadderCtx};
use nf_game::world::CollisionWorld;
use nf_game::world::V3;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// A flat dry floor at y 0 with a flat ceiling `gap` above the Bond root, both one quad.
fn unit_world(ceil_mat: u8, root: f32, gap: f32) -> CollisionWorld {
    let ceil = root + gap;
    let mut tris: Vec<([V3; 3], u8, bool)> = Vec::new();
    // floor (up-facing), mat 0xA
    tris.push(([[-10.0, 0.0, -10.0], [10.0, 0.0, 10.0], [10.0, 0.0, -10.0]], 0xA, false));
    tris.push(([[-10.0, 0.0, -10.0], [-10.0, 0.0, 10.0], [10.0, 0.0, 10.0]], 0xA, false));
    // ceiling (down-facing), the material under test
    tris.push(([[-10.0, ceil, -10.0], [10.0, ceil, -10.0], [10.0, ceil, 10.0]], ceil_mat, false));
    tris.push(([[-10.0, ceil, -10.0], [10.0, ceil, 10.0], [-10.0, ceil, 10.0]], ceil_mat, false));
    CollisionWorld::from_triangles_flags(tris)
}

/// Settle, then hold jump; drain `sfx_out` every tick and return every sound id played.
fn jump_and_collect(world: &CollisionWorld, root: f32) -> Vec<u32> {
    let mut b = Bond::new([0.0, root, 0.0], 0.0);
    let ctx = LadderCtx::default();
    let mut ids = Vec::new();
    for _ in 0..12 { b.tick_ctx(world, &ctx, &Input::default()); ids.extend(b.sfx_out.drain(..).map(|(id, _)| id)); }
    for _ in 0..90 {
        b.tick_ctx(world, &ctx, &Input { jump: true, ..Default::default() });
        ids.extend(b.sfx_out.drain(..).map(|(id, _)| id));
        if b.jump == 0 && !ids.is_empty() { break; }
    }
    ids
}

/// The pond underside (raw 0xE) bump plays the 0x5FB arm: `byte - 0xd < 2` [M 0x1916b0:195-198].
#[test]
fn the_pond_underside_head_bump_plays_5fb() {
    let world = unit_world(0xE, 1.1, 1.5);
    let ids = jump_and_collect(&world, 1.1);
    assert!(ids.contains(&0xa3), "the launch 0xA3 played [M 0x1916b0:184]: {ids:?}");
    assert!(ids.contains(&0x5fb), "the 0xE ceiling bump must play 0x5FB (byte - 0xd < 2 [M 0x1916b0:196]): {ids:?}");
}

/// Material 0xD is the second WATER of the `- 0xd < 2` arm [M 0x1916b0:196]; the disc carries no
/// 0xD surface at all [D census] so this arm rides the unit pin.
#[test]
fn the_0xd_surface_head_bump_also_plays_5fb() {
    let world = unit_world(0xD, 1.1, 1.5);
    let ids = jump_and_collect(&world, 1.1);
    assert!(ids.contains(&0x5fb), "the 0xD ceiling bump must play 0x5FB: {ids:?}");
}

/// A dry ceiling (the 0x4 concrete of 07000002's spots) keeps the 0xA3 arm: the launch AND the
/// bump, and never 0x5FB.
#[test]
fn the_dry_ceiling_head_bump_keeps_a3() {
    let world = unit_world(0x4, 1.1, 1.5);
    let ids = jump_and_collect(&world, 1.1);
    assert!(!ids.contains(&0x5fb), "a dry ceiling must never play 0x5FB: {ids:?}");
    assert_eq!(ids.iter().filter(|&&x| x == 0xa3).count(), 2, "the launch 0xA3 and the bump 0xA3 [M 0x1916b0:184,198]: {ids:?}");
}

/// The real dry ceiling on a second level: 07000003's 0x4 concrete underside at 8.87 over the
/// 6.93 floor (gap 1.94) [D level data]. Same jump: the bump stays 0xA3, never 0x5FB. (The
/// real-level 0x5FB arm is unreachable on the disc: every 0xD/0xE ceiling is a pond underside
/// under a WATER cel — Bond swims there and `Player_HandleJump` runs only from the walk
/// [M Player_Update 0x193600 dispatch] — hence the unit pins above carry the water arms.)
#[test]
fn the_dry_concrete_ceiling_bump_keeps_a3_07000003() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..2000 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
    sim.bond.pos = [8.95, 7.07, -32.30]; // the 0x4 floor 6.93 + 0.14, under the 0x4 ceiling 8.87
    sim.bond.yaw = 0.0;
    for _ in 0..20 { sim.step_bond(Default::default(), false, false, false); }
    let mut ids = Vec::new();
    for _ in 0..120 {
        sim.step_bond(nf_game::player::Input { jump: true, ..Default::default() }, false, false, false);
        let active = sim.drones.audio.active_ids();
        if active.contains(&0xa3) || active.contains(&0x5fb) { ids = active; break; }
    }
    assert!(ids.contains(&0xa3), "the dry ceiling bump plays 0xA3 (launch or bump), got {ids:?}");
    assert!(!ids.contains(&0x5fb), "a dry ceiling must never play 0x5FB, got {ids:?}");
}
/// The real dry ceiling one walk away: 07000002's 0x4 concrete underside at 8.16 over the 6.32
/// floor (gap 1.84) [D level data]. Same jump: the bump stays 0xA3, never 0x5FB.
#[test]
fn the_dry_concrete_ceiling_bump_keeps_a3_07000002() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..2000 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
    sim.bond.pos = [-46.14, 6.46, -18.48]; // the 0x4 floor 6.32 + 0.14, under the 0x4 ceiling 8.16
    sim.bond.yaw = 0.0;
    for _ in 0..20 { sim.step_bond(Default::default(), false, false, false); }
    eprintln!("[dry] after settle: substate {} pos {:?} grounded {}", sim.bond.substate, sim.bond.pos, sim.bond.grounded);
    let mut ids = Vec::new();
    for _ in 0..120 {
        sim.step_bond(nf_game::player::Input { jump: true, ..Default::default() }, false, false, false);
        let active = sim.drones.audio.active_ids();
        if active.contains(&0xa3) || active.contains(&0x5fb) { ids = active; break; }
    }
    assert!(ids.contains(&0xa3), "the dry ceiling bump plays 0xA3 (launch or bump), got {ids:?}");
    assert!(!ids.contains(&0x5fb), "a dry ceiling must never play 0x5FB, got {ids:?}");
}
